use crate::{runtime as crate_runtime, types::*};
use core::ffi::CStr;

mod paths;
pub(crate) mod runtime;

use runtime::{
    cancel_task, collect_command_arguments, collect_command_environment, current_task_pid,
    get_num_tasks_inner, kill_and_reap_host_process, lookup_device, register_task,
    resolve_host_executable, spawn_managed_thread, spawn_process, spawn_process_reaper,
    wait_for_child,
};

type size_t = usize;

#[unsafe(no_mangle)]
extern "C" fn process_create(
    path: *const ::core::ffi::c_char,
    stack_size: size_t,
    argc: ::core::ffi::c_int,
    argv: *mut *mut ::core::ffi::c_char,
) -> pid_t {
    let _ = stack_size;

    if path.is_null() {
        return -1;
    }

    let path = unsafe { CStr::from_ptr(path) };
    if path.to_bytes().is_empty() {
        return -1;
    }

    let Some(host_path) = resolve_host_executable(path) else {
        return -1;
    };
    let Some(mut arguments) = collect_command_arguments(path, argc, argv) else {
        return -1;
    };
    let Some(mut environment) = collect_command_environment() else {
        return -1;
    };

    let pid = register_task(Some(current_task_pid()));
    let Ok(host_pid) = spawn_process(
        host_path.as_c_str(),
        &mut arguments,
        &mut environment,
    ) else {
        cancel_task(pid);
        return -1;
    };

    if !spawn_process_reaper(host_pid, pid) {
        kill_and_reap_host_process(host_pid);
        cancel_task(pid);
        return -1;
    }

    pid
}

#[unsafe(no_mangle)]
extern "C" fn thread_create(
    thread_entry: ::core::option::Option<unsafe extern "C" fn(user_data: *mut ::core::ffi::c_void)>,
    user_data: *mut ::core::ffi::c_void,
    stack_size: u16,
) -> pid_t {
    let Some(thread_entry) = thread_entry else {
        return -1;
    };

    let pid = register_task(Some(current_task_pid()));
    if !spawn_managed_thread(pid, thread_entry, user_data, stack_size) {
        cancel_task(pid);
        return -1;
    }

    pid
}

#[unsafe(no_mangle)]
extern "C" fn wait(block: bool, timeout_msec: u32) -> pid_t {
    wait_for_child(current_task_pid(), block, timeout_msec).unwrap_or(-1)
}

#[unsafe(no_mangle)]
extern "C" fn die(reason: *const ::core::ffi::c_char) {
    let _ = reason;
    crate_runtime::abort_process()
}

#[unsafe(no_mangle)]
extern "C" fn task_priority_lower() {}

#[unsafe(no_mangle)]
extern "C" fn task_priority_restore() {}

#[unsafe(no_mangle)]
extern "C" fn get_num_tasks() -> u32 {
    get_num_tasks_inner()
}

#[unsafe(no_mangle)]
extern "C" fn device_get(name: *const ::core::ffi::c_char) -> *mut device_t {
    if name.is_null() {
        return core::ptr::null_mut();
    }

    let name = unsafe { CStr::from_ptr(name) };
    let Some(device) = lookup_device(name) else {
        crate_runtime::write_stderr(b"The device does not exist: ");
        crate_runtime::write_stderr(name.to_bytes());
        crate_runtime::write_stderr(b"\n");
        return core::ptr::null_mut();
    };

    device
}

#[unsafe(no_mangle)]
extern "C" fn vaddr_to_paddr(vaddr: u32) -> u32 {
    vaddr
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::{
        paths::set_base_directory_for_tests,
        runtime::{lock_global_test_runtime, reset_runtime_for_tests},
    };
    use std::{
        ffi::c_void,
        format,
        ffi::{CString, OsString},
        fs,
        os::unix::{fs::PermissionsExt, process::ExitStatusExt},
        path::{Path, PathBuf},
        process::Command,
        thread,
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
        vec,
        vec::Vec,
    };

    struct TemporaryTestDirectory {
        path: PathBuf,
    }

    impl TemporaryTestDirectory {
        fn new(test_name: &str) -> Self {
            let unique_suffix = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after the unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "why2025-badge-emu-abi-{test_name}-{}-{unique_suffix}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&path);
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TemporaryTestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    struct TemporaryEnvVar {
        key: &'static str,
        previous: Option<OsString>,
    }

    impl TemporaryEnvVar {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var_os(key);
            unsafe { std::env::set_var(key, value) };
            Self { key, previous }
        }
    }

    impl Drop for TemporaryEnvVar {
        fn drop(&mut self) {
            if let Some(previous) = self.previous.take() {
                unsafe { std::env::set_var(self.key, previous) };
            } else {
                unsafe { std::env::remove_var(self.key) };
            }
        }
    }

    fn install_current_test_binary(base_directory: &Path, host_filename: &str) -> CString {
        let source = std::env::current_exe().expect("current test binary path");
        let destination_directory = base_directory.join("APP");
        fs::create_dir_all(&destination_directory).expect("create APP device directory");

        let destination = destination_directory.join(host_filename);
        fs::copy(&source, &destination).expect("copy current test binary into emulated filesystem");

        let mut permissions = fs::metadata(&destination)
            .expect("destination metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&destination, permissions).expect("set executable permissions");

        CString::new(format!("APP:{host_filename}"))
            .expect("badge path should not contain interior nulls")
    }

    fn build_process_argv(
        path: &CStr,
        test_name: &str,
    ) -> (Vec<CString>, Vec<*mut ::core::ffi::c_char>) {
        let argv_owned = vec![
            CString::new(path.to_bytes()).expect("badge path should not contain interior nulls"),
            CString::new("--exact").expect("literal argument should not contain interior nulls"),
            CString::new(test_name).expect("test name should not contain interior nulls"),
            CString::new("--test-threads=1")
                .expect("literal argument should not contain interior nulls"),
        ];
        let argv = argv_owned
            .iter()
            .map(|argument| argument.as_ptr().cast_mut())
            .collect();
        (argv_owned, argv)
    }

    fn wait_for_task_count(expected: u32) {
        let start = Instant::now();
        while get_num_tasks() != expected {
            assert!(
                start.elapsed() < Duration::from_secs(5),
                "timed out waiting for task count {expected}, got {}",
                get_num_tasks()
            );
            thread::sleep(Duration::from_millis(10));
        }
    }

    unsafe extern "C" fn test_thread_entry(_user_data: *mut c_void) {}

    #[test]
    fn device_get_returns_tt01_and_rejects_unknown_devices() {
        let _test_lock = lock_global_test_runtime();
        reset_runtime_for_tests();

        assert!(!device_get(c"TT01".as_ptr()).is_null());
        assert!(device_get(c"DOES_NOT_EXIST".as_ptr()).is_null());
    }

    #[test]
    fn thread_create_reaps_before_wait_consumes_pid() {
        let _test_lock = lock_global_test_runtime();
        reset_runtime_for_tests();

        let base_count = get_num_tasks();
        let pid = thread_create(Some(test_thread_entry), std::ptr::null_mut(), 0);

        assert!(pid > 0);
        wait_for_task_count(base_count);
        assert_eq!(wait(true, 0), pid);
        assert_eq!(get_num_tasks(), base_count);
    }

    #[test]
    fn process_create_reaps_before_wait_consumes_pid() {
        const ENV_NAME: &str = "WHY2025_MISC_PROCESS_CREATE_CHILD";
        const TEST_NAME: &str = "misc::tests::process_create_reaps_before_wait_consumes_pid";

        let _test_lock = lock_global_test_runtime();
        reset_runtime_for_tests();

        if std::env::var_os(ENV_NAME).is_some() {
            return;
        }

        let temporary_directory = TemporaryTestDirectory::new("misc-process-create");
        let _base_directory = set_base_directory_for_tests(temporary_directory.path());
        let badge_path =
            install_current_test_binary(temporary_directory.path(), "misc-process-child");
        let (_argv_owned, mut argv) = build_process_argv(badge_path.as_c_str(), TEST_NAME);
        let _child_env = TemporaryEnvVar::set(ENV_NAME, "1");

        let base_count = get_num_tasks();
        let pid = process_create(
            badge_path.as_ptr(),
            0,
            argv.len() as ::core::ffi::c_int,
            argv.as_mut_ptr(),
        );

        assert!(pid > 0);
        wait_for_task_count(base_count);
        assert_eq!(wait(true, 0), pid);
        assert_eq!(get_num_tasks(), base_count);
    }

    #[test]
    #[cfg(unix)]
    fn die_aborts_current_process() {
        const ENV_NAME: &str = "WHY2025_MISC_DIE_TEST";
        const TEST_NAME: &str = "misc::tests::die_aborts_current_process";

        let _test_lock = lock_global_test_runtime();
        reset_runtime_for_tests();

        if std::env::var_os(ENV_NAME).is_some() {
            die(c"test abort".as_ptr());
        }

        let output = Command::new(std::env::current_exe().expect("current test binary path"))
            .arg("--exact")
            .arg(TEST_NAME)
            .env(ENV_NAME, "1")
            .output()
            .expect("spawn child test process");

        assert!(!output.status.success());
        assert_eq!(output.status.signal(), Some(libc::SIGABRT));
    }
}