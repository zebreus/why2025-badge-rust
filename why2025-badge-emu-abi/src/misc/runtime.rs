use super::paths::{ParsedPath, base_directory_cstring, base_directory_env_var_name};
use crate::{ota::abort_task_owned_ota_session, runtime as crate_runtime, types::*};
use alloc::{
    boxed::Box,
    collections::{BTreeMap, VecDeque},
    ffi::CString,
    vec::Vec,
};
use core::{
    cell::UnsafeCell,
    ffi::{CStr, c_char, c_int, c_void},
    mem::{self, MaybeUninit},
    ops::{Deref, DerefMut},
    ptr,
};

const ROOT_TASK_PID: pid_t = 1;
const CHILD_QUEUE_CAPACITY: usize = 10;
pub(crate) const MIN_STACK_SIZE: usize = 16 * 1024;

struct PthreadKey(UnsafeCell<libc::pthread_key_t>);
struct PthreadOnce(UnsafeCell<libc::pthread_once_t>);
struct RuntimeCell(UnsafeCell<*mut MiscRuntime>);

unsafe impl Sync for PthreadKey {}
unsafe impl Sync for PthreadOnce {}
unsafe impl Sync for RuntimeCell {}

static CURRENT_TASK_PID_KEY: PthreadKey = PthreadKey(UnsafeCell::new(0));
static CURRENT_TASK_PID_KEY_ONCE: PthreadOnce =
    PthreadOnce(UnsafeCell::new(libc::PTHREAD_ONCE_INIT));
static MISC_RUNTIME: RuntimeCell = RuntimeCell(UnsafeCell::new(ptr::null_mut()));
static MISC_RUNTIME_ONCE: PthreadOnce = PthreadOnce(UnsafeCell::new(libc::PTHREAD_ONCE_INIT));

#[derive(Debug, Default)]
struct TaskState {
    parent_pid: Option<pid_t>,
    children: VecDeque<pid_t>,
    ota_sessions: Vec<usize>,
}

#[derive(Debug)]
struct MiscRuntimeState {
    next_pid: pid_t,
    recycled_pids: Vec<pid_t>,
    num_tasks: u32,
    tasks: BTreeMap<pid_t, TaskState>,
    devices: BTreeMap<Vec<u8>, Box<device_t>>,
}

impl Default for MiscRuntimeState {
    fn default() -> Self {
        let mut tasks = BTreeMap::new();
        tasks.insert(ROOT_TASK_PID, TaskState::default());

        let mut devices = BTreeMap::new();
        devices.insert(b"TT01".to_vec(), make_console_device());

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

struct PthreadMutex {
    inner: UnsafeCell<libc::pthread_mutex_t>,
}

struct PthreadCondvar {
    inner: UnsafeCell<libc::pthread_cond_t>,
}

unsafe impl Sync for PthreadMutex {}
unsafe impl Sync for PthreadCondvar {}

impl PthreadMutex {
    fn new() -> Self {
        let mut mutex = MaybeUninit::<libc::pthread_mutex_t>::uninit();
        let rc = unsafe { libc::pthread_mutex_init(mutex.as_mut_ptr(), ptr::null()) };
        if rc != 0 {
            crate_runtime::abort_with_message(
                b"why2025-badge-emu-abi failed to initialize misc mutex\n",
            )
        }

        Self {
            inner: UnsafeCell::new(unsafe { mutex.assume_init() }),
        }
    }

    fn lock(&self) {
        let rc = unsafe { libc::pthread_mutex_lock(self.inner.get()) };
        if rc != 0 {
            crate_runtime::abort_with_message(
                b"why2025-badge-emu-abi failed to lock misc mutex\n",
            )
        }
    }

    fn unlock(&self) {
        let rc = unsafe { libc::pthread_mutex_unlock(self.inner.get()) };
        if rc != 0 {
            crate_runtime::abort_with_message(
                b"why2025-badge-emu-abi failed to unlock misc mutex\n",
            )
        }
    }
}

impl PthreadCondvar {
    fn new() -> Self {
        let mut condvar = MaybeUninit::<libc::pthread_cond_t>::uninit();
        let rc = unsafe { libc::pthread_cond_init(condvar.as_mut_ptr(), ptr::null()) };
        if rc != 0 {
            crate_runtime::abort_with_message(
                b"why2025-badge-emu-abi failed to initialize misc condvar\n",
            )
        }

        Self {
            inner: UnsafeCell::new(unsafe { condvar.assume_init() }),
        }
    }

    fn wait(&self, mutex: &PthreadMutex) {
        let rc = unsafe { libc::pthread_cond_wait(self.inner.get(), mutex.inner.get()) };
        if rc != 0 {
            crate_runtime::abort_with_message(
                b"why2025-badge-emu-abi failed to wait on misc condvar\n",
            )
        }
    }

    fn wait_until(&self, mutex: &PthreadMutex, deadline: &libc::timespec) -> c_int {
        unsafe { libc::pthread_cond_timedwait(self.inner.get(), mutex.inner.get(), deadline) }
    }

    fn broadcast(&self) {
        let rc = unsafe { libc::pthread_cond_broadcast(self.inner.get()) };
        if rc != 0 {
            crate_runtime::abort_with_message(
                b"why2025-badge-emu-abi failed to broadcast misc condvar\n",
            )
        }
    }
}

struct MiscRuntime {
    state_lock: PthreadMutex,
    child_events: PthreadCondvar,
    state: UnsafeCell<MiscRuntimeState>,
}

unsafe impl Sync for MiscRuntime {}

impl MiscRuntime {
    fn new() -> Self {
        Self {
            state_lock: PthreadMutex::new(),
            child_events: PthreadCondvar::new(),
            state: UnsafeCell::new(MiscRuntimeState::default()),
        }
    }

    fn lock_state(&self) -> StateGuard<'_> {
        self.state_lock.lock();
        StateGuard { runtime: self }
    }
}

struct StateGuard<'a> {
    runtime: &'a MiscRuntime,
}

impl Deref for StateGuard<'_> {
    type Target = MiscRuntimeState;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.runtime.state.get() }
    }
}

impl DerefMut for StateGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.runtime.state.get() }
    }
}

impl Drop for StateGuard<'_> {
    fn drop(&mut self) {
        self.runtime.state_lock.unlock();
    }
}

pub(crate) struct CStringArray {
    _owned: Vec<CString>,
    pointers: Vec<*mut c_char>,
}

impl CStringArray {
    fn new(owned: Vec<CString>) -> Self {
        let mut pointers = owned
            .iter()
            .map(|value| value.as_ptr().cast_mut())
            .collect::<Vec<_>>();
        pointers.push(ptr::null_mut());
        Self {
            _owned: owned,
            pointers,
        }
    }

    fn as_ptr(&mut self) -> *mut *mut c_char {
        self.pointers.as_mut_ptr()
    }
}

struct ManagedThreadStart {
    badge_pid: pid_t,
    thread_entry: unsafe extern "C" fn(*mut c_void),
    user_data: *mut c_void,
}

struct ProcessReaperStart {
    host_pid: libc::pid_t,
    badge_pid: pid_t,
}

unsafe extern "C" fn free_current_task_pid_slot(value: *mut c_void) {
    if !value.is_null() {
        unsafe { libc::free(value) };
    }
}

extern "C" fn init_current_task_pid_key() {
    let rc = unsafe {
        libc::pthread_key_create(
            CURRENT_TASK_PID_KEY.0.get(),
            Some(free_current_task_pid_slot),
        )
    };
    if rc != 0 {
        crate_runtime::abort_with_message(
            b"why2025-badge-emu-abi failed to initialize misc TLS\n",
        )
    }
}

extern "C" fn init_misc_runtime() {
    let runtime = Box::new(MiscRuntime::new());
    unsafe {
        *MISC_RUNTIME.0.get() = Box::into_raw(runtime);
    }
}

fn current_task_pid_key() -> libc::pthread_key_t {
    let rc = unsafe { libc::pthread_once(CURRENT_TASK_PID_KEY_ONCE.0.get(), init_current_task_pid_key) };
    if rc != 0 {
        crate_runtime::abort_with_message(
            b"why2025-badge-emu-abi failed to run misc TLS init\n",
        )
    }

    unsafe { *CURRENT_TASK_PID_KEY.0.get() }
}

fn misc_runtime() -> &'static MiscRuntime {
    let rc = unsafe { libc::pthread_once(MISC_RUNTIME_ONCE.0.get(), init_misc_runtime) };
    if rc != 0 {
        crate_runtime::abort_with_message(
            b"why2025-badge-emu-abi failed to initialize misc runtime\n",
        )
    }

    let runtime = unsafe { *MISC_RUNTIME.0.get() };
    if runtime.is_null() {
        crate_runtime::abort_with_message(
            b"why2025-badge-emu-abi misc runtime was not initialized\n",
        )
    }

    unsafe { &*runtime }
}

fn current_task_pid_slot() -> *mut pid_t {
    let key = current_task_pid_key();
    let existing = unsafe { libc::pthread_getspecific(key) }.cast::<pid_t>();
    if !existing.is_null() {
        return existing;
    }

    let slot = unsafe { libc::malloc(mem::size_of::<pid_t>().max(1)) }.cast::<pid_t>();
    if slot.is_null() {
        crate_runtime::abort_with_message(
            b"why2025-badge-emu-abi failed to allocate misc TLS\n",
        )
    }

    unsafe {
        *slot = 0;
    }

    let rc = unsafe { libc::pthread_setspecific(key, slot.cast::<c_void>()) };
    if rc != 0 {
        unsafe { libc::free(slot.cast::<c_void>()) };
        crate_runtime::abort_with_message(
            b"why2025-badge-emu-abi failed to install misc TLS\n",
        )
    }

    slot
}

fn build_absolute_timeout(timeout_msec: u32) -> libc::timespec {
    let mut deadline = MaybeUninit::<libc::timespec>::uninit();
    let rc = unsafe { libc::clock_gettime(libc::CLOCK_REALTIME, deadline.as_mut_ptr()) };
    if rc != 0 {
        crate_runtime::abort_with_message(
            b"why2025-badge-emu-abi failed to read realtime clock\n",
        )
    }

    let mut deadline = unsafe { deadline.assume_init() };
    deadline.tv_sec += (timeout_msec / 1000) as libc::time_t;
    deadline.tv_nsec += ((timeout_msec % 1000) as libc::c_long) * 1_000_000;
    if deadline.tv_nsec >= 1_000_000_000 {
        deadline.tv_sec += 1;
        deadline.tv_nsec -= 1_000_000_000;
    }
    deadline
}

fn create_detached_pthread(
    stack_size: Option<usize>,
    start_routine: extern "C" fn(*mut c_void) -> *mut c_void,
    arg: *mut c_void,
) -> c_int {
    let mut attr = MaybeUninit::<libc::pthread_attr_t>::uninit();
    let init_rc = unsafe { libc::pthread_attr_init(attr.as_mut_ptr()) };
    if init_rc != 0 {
        return init_rc;
    }

    let mut attr = unsafe { attr.assume_init() };
    let detach_rc = unsafe {
        libc::pthread_attr_setdetachstate(&mut attr, libc::PTHREAD_CREATE_DETACHED)
    };
    if detach_rc != 0 {
        unsafe { libc::pthread_attr_destroy(&mut attr) };
        return detach_rc;
    }

    if let Some(stack_size) = stack_size {
        let stack_rc = unsafe { libc::pthread_attr_setstacksize(&mut attr, stack_size) };
        if stack_rc != 0 {
            unsafe { libc::pthread_attr_destroy(&mut attr) };
            return stack_rc;
        }
    }

    let mut thread = MaybeUninit::<libc::pthread_t>::uninit();
    let create_rc = unsafe { libc::pthread_create(thread.as_mut_ptr(), &attr, start_routine, arg) };
    unsafe { libc::pthread_attr_destroy(&mut attr) };
    create_rc
}

extern "C" fn managed_thread_main(arg: *mut c_void) -> *mut c_void {
    let start = unsafe { Box::from_raw(arg.cast::<ManagedThreadStart>()) };
    set_current_task_pid(start.badge_pid);
    unsafe { (start.thread_entry)(start.user_data) };
    task_exited(start.badge_pid);
    ptr::null_mut()
}

extern "C" fn process_reaper_main(arg: *mut c_void) -> *mut c_void {
    let start = unsafe { Box::from_raw(arg.cast::<ProcessReaperStart>()) };

    loop {
        let waited = unsafe { libc::waitpid(start.host_pid, ptr::null_mut(), 0) };
        if waited == start.host_pid {
            break;
        }

        if waited == -1 {
            let host_errno = unsafe { *libc::__errno_location() };
            if host_errno == libc::EINTR {
                continue;
            }
            break;
        }
    }

    task_exited(start.badge_pid);
    ptr::null_mut()
}

pub(crate) fn current_task_pid() -> pid_t {
    let slot = current_task_pid_slot();
    unsafe {
        if *slot == 0 {
            *slot = ROOT_TASK_PID;
        }
        *slot
    }
}

pub(crate) fn current_managed_task_pid() -> Option<pid_t> {
    let slot = current_task_pid_slot();
    unsafe {
        match *slot {
            0 => None,
            pid => Some(pid),
        }
    }
}

pub(crate) fn set_current_task_pid(pid: pid_t) {
    unsafe {
        *current_task_pid_slot() = pid;
    }
}

pub(crate) fn register_task(parent_pid: Option<pid_t>) -> pid_t {
    let runtime = misc_runtime();
    let mut state = runtime.lock_state();
    let pid = state.allocate_pid();
    state.tasks.insert(
        pid,
        TaskState {
            parent_pid,
            children: VecDeque::new(),
            ota_sessions: Vec::new(),
        },
    );
    state.num_tasks = state.num_tasks.saturating_add(1);
    pid
}

pub(crate) fn cancel_task(pid: pid_t) {
    let ota_sessions = {
        let runtime = misc_runtime();
        let mut state = runtime.lock_state();
        let Some(task) = state.tasks.remove(&pid) else {
            return;
        };

        state.num_tasks = state.num_tasks.saturating_sub(1);
        state.recycled_pids.push(pid);
        task.ota_sessions
    };

    cleanup_ota_sessions(ota_sessions);
}

pub(crate) fn task_exited(pid: pid_t) {
    let ota_sessions = {
        let runtime = misc_runtime();
        let mut state = runtime.lock_state();
        let Some(task) = state.tasks.remove(&pid) else {
            return;
        };

        state.num_tasks = state.num_tasks.saturating_sub(1);
        state.recycled_pids.push(pid);

        if let Some(parent_pid) = task.parent_pid {
            if let Some(parent) = state.tasks.get_mut(&parent_pid) {
                if parent.children.len() < CHILD_QUEUE_CAPACITY {
                    parent.children.push_back(pid);
                }
            }
        }

        runtime.child_events.broadcast();
        task.ota_sessions
    };

    cleanup_ota_sessions(ota_sessions);
}

fn cleanup_ota_sessions(ota_sessions: Vec<usize>) {
    for handle in ota_sessions {
        abort_task_owned_ota_session(handle as ota_handle_t);
    }
}

pub(crate) fn register_ota_session(owner_pid: Option<pid_t>, handle: ota_handle_t) {
    if handle.is_null() {
        return;
    }

    let Some(owner_pid) = owner_pid else {
        return;
    };

    let runtime = misc_runtime();
    let mut state = runtime.lock_state();
    let Some(task) = state.tasks.get_mut(&owner_pid) else {
        return;
    };

    let handle = handle as usize;
    if !task.ota_sessions.contains(&handle) {
        task.ota_sessions.push(handle);
    }
}

pub(crate) fn release_ota_session(owner_pid: Option<pid_t>, handle: ota_handle_t) {
    if handle.is_null() {
        return;
    }

    let Some(owner_pid) = owner_pid else {
        return;
    };

    let runtime = misc_runtime();
    let mut state = runtime.lock_state();
    let Some(task) = state.tasks.get_mut(&owner_pid) else {
        return;
    };

    let handle = handle as usize;
    task.ota_sessions.retain(|owned_handle| *owned_handle != handle);
}

pub(crate) fn wait_for_child(parent_pid: pid_t, block: bool, timeout_msec: u32) -> Option<pid_t> {
    let runtime = misc_runtime();
    let mut state = runtime.lock_state();

    if block {
        loop {
            if let Some(child_pid) = state
                .tasks
                .get_mut(&parent_pid)
                .and_then(|parent| parent.children.pop_front())
            {
                return Some(child_pid);
            }

            if !state.tasks.contains_key(&parent_pid) {
                return None;
            }

            runtime.child_events.wait(&runtime.state_lock);
        }
    }

    if let Some(child_pid) = state
        .tasks
        .get_mut(&parent_pid)
        .and_then(|parent| parent.children.pop_front())
    {
        return Some(child_pid);
    }

    if timeout_msec == 0 || !state.tasks.contains_key(&parent_pid) {
        return None;
    }

    let deadline = build_absolute_timeout(timeout_msec);
    loop {
        let wait_rc = runtime.child_events.wait_until(&runtime.state_lock, &deadline);
        if wait_rc == libc::ETIMEDOUT {
            return None;
        }
        if wait_rc != 0 {
            crate_runtime::abort_with_message(
                b"why2025-badge-emu-abi failed to timed-wait on misc condvar\n",
            )
        }

        if let Some(child_pid) = state
            .tasks
            .get_mut(&parent_pid)
            .and_then(|parent| parent.children.pop_front())
        {
            return Some(child_pid);
        }

        if !state.tasks.contains_key(&parent_pid) {
            return None;
        }
    }
}

pub(crate) fn get_num_tasks_inner() -> u32 {
    misc_runtime().lock_state().num_tasks
}

pub(crate) fn lookup_device(name: &CStr) -> Option<*mut device_t> {
    let mut state = misc_runtime().lock_state();
    let key = name.to_bytes().to_vec();
    state
        .devices
        .get_mut(&key)
        .map(|device| &mut **device as *mut device_t)
}

pub(crate) fn resolve_host_executable(path: &CStr) -> Option<CString> {
    let host_path = ParsedPath::new(path)?.to_host_file()?;
    let mut stat = MaybeUninit::<libc::stat>::uninit();
    if unsafe { libc::stat(host_path.as_ptr(), stat.as_mut_ptr()) } != 0 {
        return None;
    }

    let stat = unsafe { stat.assume_init() };
    if stat.st_mode & libc::S_IFMT != libc::S_IFREG {
        return None;
    }

    Some(host_path)
}

pub(crate) fn collect_command_arguments(
    path: &CStr,
    argc: c_int,
    argv: *mut *mut c_char,
) -> Option<CStringArray> {
    if argc < 0 {
        return None;
    }

    if argc == 0 {
        let mut arguments = Vec::with_capacity(1);
        arguments.push(CString::new(path.to_bytes()).ok()?);
        return Some(CStringArray::new(arguments));
    }

    if argv.is_null() {
        return None;
    }

    let mut arguments = Vec::with_capacity(argc as usize);
    for index in 0..argc as usize {
        let argument_ptr = unsafe { *argv.add(index) };
        if argument_ptr.is_null() {
            return None;
        }

        let argument = unsafe { CStr::from_ptr(argument_ptr) };
        arguments.push(CString::new(argument.to_bytes()).ok()?);
    }

    Some(CStringArray::new(arguments))
}

pub(crate) fn collect_command_environment() -> Option<CStringArray> {
    let environ = unsafe {
        crate_runtime::resolve_next_object_value::<*mut *mut c_char>(b"environ\0")
    };
    let mut environment = Vec::new();

    if !environ.is_null() {
        let mut index = 0usize;
        loop {
            let entry_ptr = unsafe { *environ.add(index) };
            if entry_ptr.is_null() {
                break;
            }

            let entry = unsafe { CStr::from_ptr(entry_ptr) };
            if !entry
                .to_bytes()
                .starts_with(base_directory_env_var_name())
                || entry
                    .to_bytes()
                    .get(base_directory_env_var_name().len())
                    != Some(&b'=')
            {
                environment.push(CString::new(entry.to_bytes()).ok()?);
            }

            index += 1;
        }
    }

    let base_directory = base_directory_cstring()?;
    let mut override_entry = Vec::with_capacity(
        base_directory_env_var_name().len() + 1 + base_directory.as_bytes().len(),
    );
    override_entry.extend_from_slice(base_directory_env_var_name());
    override_entry.push(b'=');
    override_entry.extend_from_slice(base_directory.as_bytes());
    environment.push(CString::new(override_entry).ok()?);

    Some(CStringArray::new(environment))
}

pub(crate) fn spawn_process(
    host_path: &CStr,
    arguments: &mut CStringArray,
    environment: &mut CStringArray,
) -> Result<libc::pid_t, c_int> {
    let mut child_pid = 0;
    let rc = unsafe {
        libc::posix_spawn(
            &mut child_pid,
            host_path.as_ptr(),
            ptr::null(),
            ptr::null(),
            arguments.as_ptr(),
            environment.as_ptr(),
        )
    };
    if rc != 0 {
        return Err(rc);
    }

    Ok(child_pid)
}

pub(crate) fn spawn_process_reaper(host_pid: libc::pid_t, badge_pid: pid_t) -> bool {
    let start = Box::new(ProcessReaperStart { host_pid, badge_pid });
    let start_ptr = Box::into_raw(start);
    let rc = create_detached_pthread(None, process_reaper_main, start_ptr.cast());
    if rc == 0 {
        return true;
    }

    unsafe {
        drop(Box::from_raw(start_ptr));
    }
    false
}

pub(crate) fn kill_and_reap_host_process(host_pid: libc::pid_t) {
    unsafe {
        let _ = libc::kill(host_pid, libc::SIGKILL);
    }

    loop {
        let waited = unsafe { libc::waitpid(host_pid, ptr::null_mut(), 0) };
        if waited == host_pid {
            return;
        }

        if waited == -1 {
            let host_errno = unsafe { *libc::__errno_location() };
            if host_errno == libc::EINTR {
                continue;
            }
            return;
        }
    }
}

pub(crate) fn spawn_managed_thread(
    pid: pid_t,
    thread_entry: unsafe extern "C" fn(*mut c_void),
    user_data: *mut c_void,
    stack_size: u16,
) -> bool {
    let start = Box::new(ManagedThreadStart {
        badge_pid: pid,
        thread_entry,
        user_data,
    });
    let stack_size = if stack_size > 0 {
        Some(usize::from(stack_size).max(MIN_STACK_SIZE))
    } else {
        None
    };

    let start_ptr = Box::into_raw(start);
    let rc = create_detached_pthread(stack_size, managed_thread_main, start_ptr.cast());
    if rc == 0 {
        return true;
    }

    unsafe {
        drop(Box::from_raw(start_ptr));
    }
    false
}

unsafe extern "C" fn console_open(
    _dev: *mut c_void,
    _path: *mut path_t,
    _flags: c_int,
    _mode: mode_t,
) -> c_int {
    0
}

unsafe extern "C" fn console_close(_dev: *mut c_void, _fd: c_int) -> c_int {
    0
}

unsafe extern "C" fn console_write(
    _dev: *mut c_void,
    fd: c_int,
    buf: *const c_void,
    count: usize,
) -> isize {
    if buf.is_null() {
        return -1;
    }

    let written = unsafe {
        libc::syscall(
            libc::SYS_write,
            fd,
            buf,
            count,
        )
    };
    if written < 0 {
        -1
    } else {
        written as isize
    }
}

unsafe extern "C" fn console_read(
    _dev: *mut c_void,
    fd: c_int,
    buf: *mut c_void,
    count: usize,
) -> isize {
    if buf.is_null() {
        return -1;
    }

    let read = unsafe {
        libc::syscall(
            libc::SYS_read,
            fd,
            buf,
            count,
        )
    };
    if read < 0 {
        -1
    } else {
        read as isize
    }
}

unsafe extern "C" fn console_lseek(
    _dev: *mut c_void,
    _fd: c_int,
    _offset: off_t,
    _whence: c_int,
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

#[cfg(test)]
use std::sync::{LazyLock, Mutex, MutexGuard};

#[cfg(test)]
static GLOBAL_TEST_RUNTIME_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[cfg(test)]
pub(crate) type GlobalTestRuntimeGuard = MutexGuard<'static, ()>;

#[cfg(test)]
pub(crate) fn lock_global_test_runtime() -> GlobalTestRuntimeGuard {
    GLOBAL_TEST_RUNTIME_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
pub(crate) fn reset_runtime_for_tests() {
    let runtime = misc_runtime();
    {
        let mut state = runtime.lock_state();
        *state = MiscRuntimeState::default();
    }
    unsafe {
        *current_task_pid_slot() = 0;
    }
}

#[cfg(test)]
pub(crate) fn ota_session_count_for_task(pid: pid_t) -> usize {
    let state = misc_runtime().lock_state();
    state
        .tasks
        .get(&pid)
        .map(|task| task.ota_sessions.len())
        .unwrap_or(0)
}