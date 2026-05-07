use crate::{
    emulated::badgevms::fs::paths::{BASE_DIRECTORY_ENV_VAR, base_directory},
    types::*,
};
use core::ffi::{CStr, c_void};
use std::{
    os::unix::process::CommandExt,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};

pub(crate) mod runtime;

use runtime::{
    MIN_STACK_SIZE, cancel_task, collect_command_arguments, current_task_pid, get_num_tasks_inner,
    lookup_device, register_task, resolve_host_executable, set_current_task_pid, task_exited,
    wait_for_child,
};

// Emulated stubs for BadgeVMS task, device, and misc functions.
//
// The rustdoc on the items in this file is derived from the upstream firmware implementation in
// `WHY2025/team-badge/firmware` at commit `a548d825a3295432d374939607feb552eb505210`
// (`Update espressif/eppp_link`). The goal is to document the exact implementation behavior of the
// current firmware, including details that are only visible in the `.c` files and not in the
// public headers.

type size_t = usize;

/// Create a new managed BadgeVMS process from an ELF path.
///
/// # Exact upstream behavior
///
/// In upstream this is just a wrapper around `run_task_path(path, stack_size, TASK_TYPE_ELF_PATH,
/// argc, argv)`.
///
/// `run_task_path` first checks that the request is an ELF-path launch, then validates `path` by
/// opening it with `why_open(path, O_RDONLY, 0)` and immediately closing it again. If the open
/// fails it logs a warning and returns `-1`.
///
/// The public header takes `stack_size` as `size_t`, but upstream narrows it to `uint16_t` when it
/// enters `run_task_path` / `run_task` and stores it in the Zeus command structure. Any high bits
/// are therefore discarded before Zeus later clamps the value up to `MIN_STACK_SIZE` (`16384`).
///
/// Argument handling is also implementation-defined rather than purely header-defined:
///
/// - If `argc == 0`, upstream synthesizes a temporary argument vector with `argc = 1` and
///   `argv[0] = strdup(path)`.
/// - If `argc != 0`, upstream keeps the caller's logical argument list as-is.
/// - In both cases `run_task` deep-copies the argument vector into one contiguous allocation sized
///   as `(argc + 1) * sizeof(char*) + sum(strlen(argv[i]) + 1)`.
/// - If that allocation fails, the call returns `-1` before Zeus is involved.
///
/// The caller then sends a `zeus_command_message_t` to `zeus_queue` and blocks indefinitely in
/// `ulTaskNotifyTakeIndexed(0, pdTRUE, portMAX_DELAY)` waiting for Zeus to reply with a PID or
/// `-1`.
///
/// # Zeus-side side effects
///
/// On the Zeus task, a successful launch:
///
/// - allocates a PID from the global PID pool (`1..=127` in the current firmware)
/// - clamps the already-narrowed stack size to at least `MIN_STACK_SIZE` (`16384`)
/// - allocates `task_info_t` in SPIRAM and creates a per-task `children` queue with capacity `10`
/// - creates a fresh `task_thread_t` rooted at `VADDR_TASK_START`, which means a fresh address
///   space / heap rather than a shared thread heap
/// - pre-opens file descriptors `0`, `1`, and `2` against device `"TT01"`
/// - allocates one khash-backed resource table per `task_resource_type_t`
/// - records the caller PID as the new task's parent PID
/// - creates a FreeRTOS task named `Task <pid>`, pinned to core `1`, at priority `5`
/// - stores `task_info_t` in TLS slot `1`, sets the application task tag to `0x12345678`, adds the
///   task to `process_table`, and increments `num_tasks`
///
/// The task type is normalized to `TASK_TYPE_ELF_PATH`, so process creation is path-based even if
/// the lower-level helpers also understand `TASK_TYPE_ELF`.
///
/// # Failure paths and subtleties
///
/// Zeus returns `-1` if PID allocation, `task_info_t` allocation, heap creation, or FreeRTOS task
/// creation fails.
///
/// There is also a subtle cleanup issue in current upstream: the Zeus error path frees the PID and
/// `task_info_t`, but it does not call `task_thread_destroy` if a `task_thread_t` had already been
/// attached. Late Zeus-side failures can therefore leak per-process heap state.
///
/// Another subtle interaction is with the default `TT01` handles: `device_get("TT01")` only logs
/// and returns `NULL` on lookup failure. That means process creation can still succeed while the
/// initial stdio file-handle slots contain null device pointers.
///
/// # Interactions with other features
///
/// Resource teardown is not synchronous with task exit. The `hades` task performs cleanup after
/// FreeRTOS deletion, so a parent only hears about child termination after the child has already
/// been removed from the process table and its task resources have started being reclaimed.
///
/// `application_launch` relies on this function and currently passes `stack_size = 0, argc = 0,
/// argv = NULL`, which means the actual stack size becomes the minimum stack size and `argv[0]` is
/// synthesized from the ELF path.
#[unsafe(no_mangle)]
pub extern "C" fn process_create(
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
    let Some((argv0, arguments)) = collect_command_arguments(path, argc, argv) else {
        return -1;
    };

    let mut command = Command::new(&host_path);
    command
        .arg0(&argv0)
        .args(arguments)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .env(BASE_DIRECTORY_ENV_VAR, base_directory());

    let Ok(child) = command.spawn() else {
        return -1;
    };

    let pid = register_task(Some(current_task_pid()));
    let child = Arc::new(Mutex::new(child));
    if thread::Builder::new()
        .name(format!("BadgeVMS process reaper {pid}"))
        .spawn({
            let child = Arc::clone(&child);
            move || {
                let mut child = child
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let _ = child.wait();
                task_exited(pid);
            }
        })
        .is_err()
    {
        let mut child = child
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _ = child.kill();
        let _ = child.wait();
        cancel_task(pid);
        return -1;
    }

    pid
}
/// Create a new managed BadgeVMS thread that shares the caller's address space.

/// # Exact upstream behavior

/// The public entry point packages the request into a `zeus_command_message_t`, sends it to Zeus,
/// and then blocks indefinitely waiting for Zeus to reply with a PID via task notification.

/// Unlike `process_create`, this path does not create a fresh `task_thread_t`. Zeus instead calls
/// `task_thread_ref(command.parent_task_info->thread)` and reuses the parent's thread/heap object.
/// That has several concrete consequences:

/// - the new thread shares the parent's mapped address space
/// - the new thread shares the parent's file-handle table
/// - the new thread shares the parent's resource tables
/// - thread/heap cleanup is reference-counted, not per-thread

/// `task_thread_ref` increments the parent's refcount unless it has already reached zero. If the
/// parent heap is already being torn down, `task_thread_ref` returns `NULL`, Zeus logs a warning,
/// and thread creation fails with `-1`.

/// Zeus still allocates a distinct PID, allocates a distinct `task_info_t`, creates a distinct
/// `children` queue, stores the caller PID as the parent PID, and spawns a FreeRTOS task named
/// `Task <pid>` pinned to core `1` at priority `5`.

/// The thread entry point is `generic_thread`, which calls `task_info->thread_entry(task_info->buffer)`.
/// In other words, the firmware passes `user_data` through the `buffer` field and does not adapt or
/// wrap it further. There is no null-check on `thread_entry`; a null function pointer would be
/// invoked directly.

/// If the entry function returns, the firmware logs that return and immediately calls
/// `vTaskDelete(NULL)`.

/// # Interactions with other features

/// Because threads share the parent's `task_thread_t`, they also participate in the same delayed
/// teardown path as the parent. If a parent task dies, Hades force-deletes all remaining children
/// whose recorded parent PID matches the dead task, regardless of whether those children are full
/// processes or shared-heap threads.
#[unsafe(no_mangle)]
pub extern "C" fn thread_create(
    thread_entry: ::core::option::Option<unsafe extern "C" fn(user_data: *mut ::core::ffi::c_void)>,
    user_data: *mut ::core::ffi::c_void,
    stack_size: u16,
) -> pid_t {
    let Some(thread_entry) = thread_entry else {
        return -1;
    };

    let parent_pid = current_task_pid();
    let pid = register_task(Some(parent_pid));
    let user_data = user_data as usize;
    let mut builder = thread::Builder::new().name(format!("Task {pid}"));
    if stack_size > 0 {
        builder = builder.stack_size(usize::from(stack_size).max(MIN_STACK_SIZE));
    }

    if builder
        .spawn(move || {
            set_current_task_pid(pid);
            unsafe { thread_entry(user_data as *mut c_void) };
            task_exited(pid);
        })
        .is_err()
    {
        cancel_task(pid);
        return -1;
    }

    pid
}
/// Wait for a child process or thread to be reaped by the firmware.
///
/// # Exact upstream behavior
///
/// Upstream simply reads from `get_task_info()->children`, which is a FreeRTOS queue created with a
/// capacity of `10` PIDs when the task was born.
///
/// The timeout behavior is more specific than the public header suggests:
///
/// - if `block == true`, the firmware uses `portMAX_DELAY` and ignores `timeout_msec`
/// - if `block == false`, it converts `timeout_msec` to ticks using integer division
///   (`timeout_msec / portTICK_PERIOD_MS`), so sub-tick values round down to zero
/// - if `xQueueReceive` fails, the function returns `-1`
/// - otherwise it returns the child PID read from the queue
///
/// # Producer-side details
///
/// Child termination is reported by the `hades` task, not by the exiting child directly.
/// `vTaskPreDeletionHook` sends the dead PID to `hades_queue`, then Hades:
///
/// - removes the child from `process_table`
/// - destroys or dereferences its shared `task_thread_t`
/// - frees `task_info_t`
/// - only then tries to enqueue the dead PID onto the parent's `children` queue
/// - force-deletes any surviving descendants of the dead task
/// - frees the PID back to the PID pool and decrements `num_tasks`
///
/// This means `wait` observes already-reaped children. By the time the parent receives the PID, the
/// child's resources are gone and the numeric PID is already eligible for reuse.
///
/// Hades sends to the parent queue with a zero timeout. If the queue is full, it only logs a
/// warning and drops the notification, so `wait` can miss child exits permanently under load.
///
/// # Assumptions not enforced by the API
///
/// The implementation assumes the caller has a valid `children` queue in its `task_info_t`. There
/// is no guard for kernel-context callers using the statically initialized `kernel_task`.
#[unsafe(no_mangle)]
pub extern "C" fn wait(block: bool, timeout_msec: u32) -> pid_t {
    wait_for_child(current_task_pid(), block, timeout_msec).unwrap_or(-1)
}
/// Abort the whole system immediately via `esp_system_abort`.
///
/// # Exact upstream behavior
///
/// The user-visible `die` symbol is backed by `why_die` in `wrapped_funcs.c`, which is marked
/// `IRAM_ATTR` and does exactly one thing: call `esp_system_abort(reason)`.
///
/// The `reason` pointer is forwarded unchanged. There is no logging, no task-local cleanup, no
/// attempt to notify parents, and no opportunity for Hades-based resource reclamation to complete
/// first. This is a system-wide abort path rather than a process-local termination path.
#[unsafe(no_mangle)]
pub extern "C" fn die(reason: *const ::core::ffi::c_char) {
    let _ = reason;
    std::process::abort();
}
/// Lower the current managed task's FreeRTOS priority to `TASK_PRIORITY_LOW` (`4`).
///
/// # Exact upstream behavior
///
/// Upstream calls `get_task_info()`, checks `eTaskGetState(task_info->handle) != eDeleted`, and if
/// the task is still alive calls `vTaskPrioritySet(task_info->handle, TASK_PRIORITY_LOW)`.
///
/// There is no nesting, token, or previous-priority bookkeeping. This is not "restore to the value
/// before the last priority change"; it is an unconditional write of priority `4` for the current
/// managed task, provided the task has not already reached the deleted state.
///
/// # Interaction with the compositor
///
/// The compositor can boost the task owning the frontmost fullscreen window to
/// `TASK_PRIORITY_FOREGROUND` (`6`) while it remains eligible for that boost. Calling
/// `task_priority_lower` overrides that current priority immediately.
#[unsafe(no_mangle)]
pub extern "C" fn task_priority_lower() {
    // Host threads/processes do not have a portable FreeRTOS-equivalent priority model.
}
/// Reset the current managed task's FreeRTOS priority to the normal task priority (`5`).
///
/// # Exact upstream behavior
///
/// Upstream calls `get_task_info()`, checks `eTaskGetState(task_info->handle) != eDeleted`, and if
/// the task is still alive calls `vTaskPrioritySet(task_info->handle, TASK_PRIORITY)`.
///
/// The public header says "Restore to previous priority", but the implementation does not remember
/// any previous value. It always writes `TASK_PRIORITY` (`5`). This can therefore undo unrelated
/// priority changes as well.
///
/// In particular, the compositor may temporarily boost a fullscreen foreground window's task to
/// `TASK_PRIORITY_FOREGROUND` (`6`). Calling `task_priority_restore` drops that task back to `5`
/// until the compositor raises it again.
#[unsafe(no_mangle)]
pub extern "C" fn task_priority_restore() {
    // Host threads/processes do not have a portable FreeRTOS-equivalent priority model.
}
/// Return the current number of live Zeus-managed tasks.
///
/// # Exact upstream behavior
///
/// This is a direct read of the global `num_tasks` counter in `task.c`.
///
/// The counter is incremented only after Zeus successfully creates a managed task and records it in
/// the process table, and it is decremented only after Hades has processed task death, reclaimed the
/// task's process-table entry, and freed the PID.
///
/// Consequences of that exact implementation:
///
/// - kernel tasks are not counted; the counter tracks Zeus-managed ELF tasks and threads only
/// - a task that has returned but has not yet been processed by Hades still contributes to the count
/// - once Hades finishes cleanup, the task stops contributing to the count even if a parent has not
///   yet consumed its PID through `wait`
#[unsafe(no_mangle)]
pub extern "C" fn get_num_tasks() -> u32 {
    get_num_tasks_inner()
}

/// Look up a registered global device by its exact string name.
///
/// # Exact upstream behavior
///
/// Upstream acquires `device_table_lock` with `xSemaphoreTake(..., portMAX_DELAY)`, performs a
/// khash string lookup, releases the mutex, and returns the raw `device_t *` stored in the global
/// device table.
///
/// Important observable details:
///
/// - lookup is by exact string key
/// - there is no cloning, refcounting, or ownership transfer; the returned pointer is the global
///   device-table entry
/// - if the device table mutex cannot be acquired, the firmware logs an error and calls `abort()`
/// - if the device name does not exist, the khash helper logs `"The device does not exist: <name>"`
///   and the function returns `NULL`
///
/// The null-return behavior is only obvious in the macro-based implementation, not in the public
/// header.
///
/// # Interaction with task creation
///
/// `task_thread_init` uses `device_get("TT01")` to prepopulate file descriptors `0`, `1`, and `2`
/// for every new process. Missing `TT01` therefore does not fail process creation immediately, but
/// it does leave those stdio slots pointing at null devices.
#[unsafe(no_mangle)]
pub extern "C" fn device_get(name: *const ::core::ffi::c_char) -> *mut device_t {
    if name.is_null() {
        return std::ptr::null_mut();
    }

    let name = unsafe { CStr::from_ptr(name) }
        .to_string_lossy()
        .into_owned();
    let Some(device) = lookup_device(&name) else {
        eprintln!("The device does not exist: {name}");
        return std::ptr::null_mut();
    };

    device
}
/// Convert the current PSRAM0 virtual mapping to a physical address.
///
/// # Exact upstream behavior
///
/// Upstream hard-codes `MMU_TARGET_PSRAM0`, obtains that target's MMU ID via
/// `why_mmu_hal_get_id_from_target`, calls `mmu_hal_vaddr_to_paddr`, ignores the returned target,
/// and returns the raw `paddr` output.
///
/// The implementation has no validation or error reporting. It does not check whether `vaddr` is
/// actually mapped, whether it belongs to PSRAM0, or whether the returned target matches the assumed
/// one.
///
/// # Interaction with task address spaces
///
/// BadgeVMS remaps and unmaps managed task address spaces on task switches. Because this function
/// consults the MMU state directly, the result depends on the mappings active for the *currently
/// running* task at the moment of the call. A value obtained in one task context should therefore be
/// treated as context-sensitive rather than a globally stable translation.
#[unsafe(no_mangle)]
pub extern "C" fn vaddr_to_paddr(vaddr: u32) -> u32 {
    vaddr
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emulated::badgevms::{
        fs::paths::set_base_directory_for_tests, misc::runtime::lock_global_test_runtime,
    };
    use std::{
        ffi::{CString, OsString},
        fs,
        os::unix::{fs::PermissionsExt, process::ExitStatusExt},
        path::{Path, PathBuf},
        process::Command,
        thread,
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
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
                "why2025-badge-sys-{test_name}-{}-{unique_suffix}",
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
            // These tests are executed with `--test-threads=1`, so process-wide env mutation is
            // serialized for the duration of the guard.
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
        assert!(!device_get(c"TT01".as_ptr()).is_null());
        assert!(device_get(c"DOES_NOT_EXIST".as_ptr()).is_null());
    }

    #[test]
    fn thread_create_reaps_before_wait_consumes_pid() {
        let _test_lock = lock_global_test_runtime();
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
        const TEST_NAME: &str =
            "emulated::badgevms::misc::tests::process_create_reaps_before_wait_consumes_pid";

        let _test_lock = lock_global_test_runtime();

        if std::env::var_os(ENV_NAME).is_some() {
            return;
        }

        let temporary_directory = TemporaryTestDirectory::new("misc-process-create");
        let _base_directory =
            set_base_directory_for_tests(temporary_directory.path().to_path_buf());
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
        const TEST_NAME: &str = "emulated::badgevms::misc::tests::die_aborts_current_process";

        let _test_lock = lock_global_test_runtime();

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
