//! Internal host-side runtime for the emulated BadgeVMS misc/task surface.
//!
//! # Design Principles
//!
//! - Keep the public ABI in `misc.rs`. This module owns only the host-side bookkeeping and helper
//!   code needed to implement that ABI.
//! - Preserve behavior that guest code can actually observe, rather than mirroring every firmware
//!   implementation detail. In practice that means synthetic PIDs, per-parent child queues, task
//!   count timing, and exact-name device lookup matter more than reproducing Zeus/Hades literally.
//! - Use host-native execution primitives where isolation matters. `process_create` is backed by a
//!   real host child process so `die()` can abort only the guest process, while `thread_create`
//!   maps to a host thread inside the current process.
//! - Keep runtime state process-local. Each host process gets its own task table, child queue
//!   storage, and device registry, which matches the process-boundary expectations of the emulator
//!   better than trying to share mutable state across spawned guests.
//! - Reap before `wait()`. Task counts are decremented and child notifications are queued when the
//!   host-side task is observed to exit, so `wait()` consumes an already-reaped child PID, matching
//!   the firmware's externally visible timing.
//! - Make emulator-only simplifications explicit. Priority changes, MMU behavior, and other kernel
//!   details are modeled elsewhere as no-ops or identity mappings instead of being hidden behind
//!   fake fidelity in this runtime.
//! - Keep the helpers narrow and testable. Path resolution, command-line synthesis, task tracking,
//!   and the small built-in device registry live here so `misc.rs` can remain a thin exported shim.

use crate::{
    emulated::badgevms::fs::paths::ParsedPath,
    types::*,
};
use core::ffi::{CStr, c_char, c_void};
use std::{
    cell::Cell,
    collections::{HashMap, VecDeque},
    ffi::OsString,
    io::{Read, Write},
    os::unix::ffi::OsStringExt,
    path::PathBuf,
    sync::{Condvar, LazyLock, Mutex, MutexGuard},
    time::Duration,
};

const ROOT_TASK_PID: pid_t = 1;
const CHILD_QUEUE_CAPACITY: usize = 10;
pub(crate) const MIN_STACK_SIZE: usize = 16 * 1024;

thread_local! {
    pub(crate) static CURRENT_TASK_PID: Cell<pid_t> = const { Cell::new(0) };
}

#[derive(Debug, Default)]
struct TaskState {
    parent_pid: Option<pid_t>,
    children: VecDeque<pid_t>,
}

#[derive(Debug)]
struct MiscRuntimeState {
    next_pid: pid_t,
    recycled_pids: Vec<pid_t>,
    num_tasks: u32,
    tasks: HashMap<pid_t, TaskState>,
    devices: HashMap<String, Box<device_t>>,
}

impl Default for MiscRuntimeState {
    fn default() -> Self {
        let mut tasks = HashMap::new();
        tasks.insert(ROOT_TASK_PID, TaskState::default());

        let mut devices = HashMap::new();
        devices.insert(String::from("TT01"), make_console_device());

        Self {
            next_pid: ROOT_TASK_PID + 1,
            recycled_pids: Vec::new(),
            num_tasks: 1,
            tasks,
            devices,
        }
    }
}

impl MiscRuntimeState {
    fn allocate_pid(&mut self) -> pid_t {
        if let Some(pid) = self.recycled_pids.pop() {
            return pid;
        }

        let mut pid = if self.next_pid <= ROOT_TASK_PID {
            ROOT_TASK_PID + 1
        } else {
            self.next_pid
        };

        loop {
            if !self.tasks.contains_key(&pid) {
                self.next_pid = pid.saturating_add(1);
                if self.next_pid <= ROOT_TASK_PID {
                    self.next_pid = ROOT_TASK_PID + 1;
                }
                return pid;
            }

            pid = pid.saturating_add(1);
            if pid <= ROOT_TASK_PID {
                pid = ROOT_TASK_PID + 1;
            }
        }
    }
}

struct MiscRuntime {
    state: Mutex<MiscRuntimeState>,
    child_events: Condvar,
}

impl MiscRuntime {
    fn new() -> Self {
        Self {
            state: Mutex::new(MiscRuntimeState::default()),
            child_events: Condvar::new(),
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, MiscRuntimeState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn register_task(&self, parent_pid: Option<pid_t>) -> pid_t {
        let mut state = self.lock_state();
        let pid = state.allocate_pid();
        state.tasks.insert(
            pid,
            TaskState {
                parent_pid,
                children: VecDeque::new(),
            },
        );
        state.num_tasks = state.num_tasks.saturating_add(1);
        pid
    }

    fn cancel_task(&self, pid: pid_t) {
        let mut state = self.lock_state();
        if state.tasks.remove(&pid).is_some() {
            state.num_tasks = state.num_tasks.saturating_sub(1);
            state.recycled_pids.push(pid);
        }
    }

    fn task_exited(&self, pid: pid_t) {
        let mut state = self.lock_state();
        let Some(task) = state.tasks.remove(&pid) else {
            return;
        };

        state.num_tasks = state.num_tasks.saturating_sub(1);
        state.recycled_pids.push(pid);

        if let Some(parent_pid) = task.parent_pid
            && let Some(parent) = state.tasks.get_mut(&parent_pid)
        {
            if parent.children.len() < CHILD_QUEUE_CAPACITY {
                parent.children.push_back(pid);
            } else {
                eprintln!("Dropping child exit notification for pid {pid}");
            }
        }

        self.child_events.notify_all();
    }

    fn wait_for_child(&self, parent_pid: pid_t, block: bool, timeout_msec: u32) -> Option<pid_t> {
        let mut state = self.lock_state();

        if block {
            loop {
                let parent = state.tasks.get_mut(&parent_pid)?;
                if let Some(pid) = parent.children.pop_front() {
                    return Some(pid);
                }
                state = self
                    .child_events
                    .wait(state)
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
            }
        }

        let timeout = Duration::from_millis(timeout_msec as u64);
        loop {
            let parent = state.tasks.get_mut(&parent_pid)?;
            if let Some(pid) = parent.children.pop_front() {
                return Some(pid);
            }
            if timeout.is_zero() {
                return None;
            }

            let (next_state, wait_result) = self
                .child_events
                .wait_timeout(state, timeout)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if wait_result.timed_out() {
                return None;
            }
            state = next_state;
        }
    }
}

static MISC_RUNTIME: LazyLock<MiscRuntime> = LazyLock::new(MiscRuntime::new);

pub(crate) fn current_task_pid() -> pid_t {
    CURRENT_TASK_PID.with(|current_pid| {
        let pid = current_pid.get();
        if pid != 0 {
            return pid;
        }

        current_pid.set(ROOT_TASK_PID);
        ROOT_TASK_PID
    })
}

pub(crate) fn set_current_task_pid(pid: pid_t) {
    CURRENT_TASK_PID.with(|current_pid| current_pid.set(pid));
}

pub(crate) fn register_task(parent_pid: Option<pid_t>) -> pid_t {
    MISC_RUNTIME.register_task(parent_pid)
}

pub(crate) fn cancel_task(pid: pid_t) {
    MISC_RUNTIME.cancel_task(pid);
}

pub(crate) fn task_exited(pid: pid_t) {
    MISC_RUNTIME.task_exited(pid);
}

pub(crate) fn wait_for_child(parent_pid: pid_t, block: bool, timeout_msec: u32) -> Option<pid_t> {
    MISC_RUNTIME.wait_for_child(parent_pid, block, timeout_msec)
}

pub(crate) fn get_num_tasks_inner() -> u32 {
    MISC_RUNTIME.lock_state().num_tasks
}

pub(crate) fn lookup_device(name: &str) -> Option<*mut device_t> {
    let mut state = MISC_RUNTIME.lock_state();
    state
        .devices
        .get_mut(name)
        .map(|device| &mut **device as *mut device_t)
}

pub(crate) fn resolve_host_executable(path: &CStr) -> Option<PathBuf> {
    let parsed_path = ParsedPath::new(path).ok()?;
    let host_path = parsed_path.to_host_file();
    if host_path.is_file() {
        Some(host_path)
    } else {
        None
    }
}

pub(crate) fn collect_command_arguments(
    path: &CStr,
    argc: ::core::ffi::c_int,
    argv: *mut *mut c_char,
) -> Option<(OsString, Vec<OsString>)> {
    if argc < 0 {
        return None;
    }

    if argc == 0 {
        return Some((OsString::from_vec(path.to_bytes().to_vec()), Vec::new()));
    }

    if argv.is_null() {
        return None;
    }

    let mut collected_args = Vec::with_capacity(argc as usize);
    for index in 0..argc as usize {
        let argument_ptr = unsafe { *argv.add(index) };
        if argument_ptr.is_null() {
            return None;
        }

        let argument = unsafe { CStr::from_ptr(argument_ptr) };
        collected_args.push(OsString::from_vec(argument.to_bytes().to_vec()));
    }

    let mut collected_args = collected_args.into_iter();
    let argv0 = collected_args.next()?;
    Some((argv0, collected_args.collect()))
}

unsafe extern "C" fn console_open(
    _dev: *mut c_void,
    _path: *mut path_t,
    _flags: ::core::ffi::c_int,
    _mode: mode_t,
) -> ::core::ffi::c_int {
    0
}

unsafe extern "C" fn console_close(
    _dev: *mut c_void,
    _fd: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    0
}

unsafe extern "C" fn console_write(
    _dev: *mut c_void,
    fd: ::core::ffi::c_int,
    buf: *const c_void,
    count: usize,
) -> isize {
    if buf.is_null() {
        return -1;
    }

    let bytes = unsafe { std::slice::from_raw_parts(buf.cast::<u8>(), count) };
    let write_result = if fd == 2 {
        let mut stderr = std::io::stderr().lock();
        stderr.write_all(bytes).and_then(|()| stderr.flush())
    } else {
        let mut stdout = std::io::stdout().lock();
        stdout.write_all(bytes).and_then(|()| stdout.flush())
    };

    if write_result.is_ok() {
        count as isize
    } else {
        -1
    }
}

unsafe extern "C" fn console_read(
    _dev: *mut c_void,
    _fd: ::core::ffi::c_int,
    buf: *mut c_void,
    count: usize,
) -> isize {
    if buf.is_null() {
        return -1;
    }

    let bytes = unsafe { std::slice::from_raw_parts_mut(buf.cast::<u8>(), count) };
    match std::io::stdin().read(bytes) {
        Ok(read) => read as isize,
        Err(_) => -1,
    }
}

unsafe extern "C" fn console_lseek(
    _dev: *mut c_void,
    _fd: ::core::ffi::c_int,
    _offset: off_t,
    _whence: ::core::ffi::c_int,
) -> isize {
    -1
}

unsafe extern "C" fn console_destroy(_dev: *mut c_void) {}

fn make_console_device() -> Box<device_t> {
    Box::new(device_t {
        type_: device_type_t::DEVICE_TYPE_BLOCK,
        _open: Some(console_open),
        _close: Some(console_close),
        _write: Some(console_write),
        _read: Some(console_read),
        _lseek: Some(console_lseek),
        _destroy: Some(console_destroy),
    })
}