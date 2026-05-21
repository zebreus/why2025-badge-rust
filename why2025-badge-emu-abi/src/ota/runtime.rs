use crate::{
    host_forward,
    misc::runtime::{current_managed_task_pid, register_ota_session, release_ota_session},
    types::*,
};
use alloc::{
    ffi::CString,
    format,
    string::String,
    vec::Vec,
};
use core::{
    ffi::{c_char, c_int, c_void, CStr},
    mem::MaybeUninit,
    ptr,
    str,
};
use serde::{Deserialize, Serialize};
use spin::{Lazy, Mutex};

#[cfg(test)]
use std::{fs, path::PathBuf};

const OTA_BASE_DIRECTORY: &str = ".why2025-badge/data/ota";
const OTA_ROOT_OVERRIDE_ENV: &[u8] = b"WHY2025_EMULATOR_OTA_ROOT\0";
const HOME_ENV_VAR: &[u8] = b"HOME\0";
const STATE_FILE_NAME: &str = "state.json";
const TEMP_STATE_FILE_NAME: &str = "state.tmp";
const SLOTS_DIRECTORY_NAME: &str = "slots";
const STAGING_FILE_NAME: &str = "staging.bin";
const DEFAULT_RUNNING_VERSION: &str = concat!("emulator-", env!("CARGO_PKG_VERSION"));
const ESP_APP_DESC_MAGIC_WORD: u32 = 0xABCD5432;
const ESP_APP_DESC_ALIGNMENT: usize = 4;
const ESP_APP_DESC_SEARCH_LIMIT: usize = 4096;
const ESP_APP_DESC_VERSION_OFFSET: usize = 16;
const ESP_APP_DESC_VERSION_LENGTH: usize = 32;

static OTA_SESSION_HANDLE: u8 = 0;
static OTA_RUNTIME: Lazy<Mutex<OtaRuntime>> = Lazy::new(|| Mutex::new(OtaRuntime::default()));

#[cfg(test)]
static TEST_OTA_ROOT_DIRECTORY: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct SlotState {
    image_path: String,
    version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedOtaState {
    running: SlotState,
    configured: SlotState,
    last_invalid: Option<SlotState>,
    force_next_validation_failure: bool,
}

#[derive(Debug)]
struct FileDescriptor(c_int);

impl Drop for FileDescriptor {
    fn drop(&mut self) {
        unsafe {
            let _ = host_forward::close(self.0);
        }
    }
}

#[derive(Debug)]
struct StagingSession {
    fd: FileDescriptor,
    path: String,
}

#[derive(Debug)]
struct OtaRuntime {
    initialized: bool,
    open: bool,
    error: bool,
    running: SlotState,
    configured: SlotState,
    last_invalid: Option<SlotState>,
    force_next_validation_failure: bool,
    staging: Option<StagingSession>,
}

impl Default for OtaRuntime {
    fn default() -> Self {
        let placeholder = SlotState {
            image_path: String::new(),
            version: String::from(DEFAULT_RUNNING_VERSION),
        };

        Self {
            initialized: false,
            open: false,
            error: false,
            running: placeholder.clone(),
            configured: placeholder,
            last_invalid: None,
            force_next_validation_failure: false,
            staging: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WriteFileError {
    Open(c_int),
    Write(c_int),
}

fn ota_runtime() -> spin::MutexGuard<'static, OtaRuntime> {
    let mut runtime = OTA_RUNTIME.lock();
    initialize_runtime(&mut runtime);
    runtime
}

fn initialize_runtime(runtime: &mut OtaRuntime) {
    if runtime.initialized {
        return;
    }

    let root = ota_root_directory();
    let mut state = load_persisted_state(&root);

    let _ = create_dir_all(&slots_directory(&root));
    let _ = remove_file(&staging_file_path(&root));

    if state.configured != state.running {
        let previous = state.running.clone();
        let candidate = state.configured.clone();

        if state.force_next_validation_failure {
            state.last_invalid = Some(candidate);
            state.running = previous.clone();
            state.configured = previous;
            state.force_next_validation_failure = false;
        } else {
            state.running = candidate.clone();
            state.configured = candidate;
        }

        let _ = save_persisted_state(&root, &state);
    }

    runtime.initialized = true;
    runtime.open = false;
    runtime.error = false;
    runtime.running = state.running;
    runtime.configured = state.configured;
    runtime.last_invalid = state.last_invalid;
    runtime.force_next_validation_failure = state.force_next_validation_failure;
    runtime.staging = None;
}

fn current_errno() -> c_int {
    unsafe { *libc::__errno_location() }
}

fn host_env_string(name: &[u8]) -> Option<String> {
    let value = unsafe { host_forward::getenv(name.as_ptr().cast::<c_char>()) };
    if value.is_null() {
        return None;
    }

    Some(unsafe { CStr::from_ptr(value) }.to_string_lossy().into_owned())
}

fn join_path(base: &str, tail: &str) -> String {
    if base.is_empty() {
        return String::from(tail);
    }
    if tail.is_empty() {
        return String::from(base);
    }
    if base.ends_with('/') {
        format!("{base}{tail}")
    } else {
        format!("{base}/{tail}")
    }
}

fn ota_root_directory() -> String {
    #[cfg(test)]
    if let Some(root) = TEST_OTA_ROOT_DIRECTORY.lock().clone() {
        return root.to_string_lossy().into_owned();
    }

    if let Some(root) = host_env_string(OTA_ROOT_OVERRIDE_ENV)
        && !root.is_empty()
    {
        return root;
    }

    if let Some(home_directory) = host_env_string(HOME_ENV_VAR)
        && !home_directory.is_empty()
    {
        return join_path(&home_directory, OTA_BASE_DIRECTORY);
    }

    String::from(OTA_BASE_DIRECTORY)
}

fn slots_directory(root: &str) -> String {
    join_path(root, SLOTS_DIRECTORY_NAME)
}

fn state_file_path(root: &str) -> String {
    join_path(root, STATE_FILE_NAME)
}

fn temporary_state_file_path(root: &str) -> String {
    join_path(root, TEMP_STATE_FILE_NAME)
}

fn staging_file_path(root: &str) -> String {
    join_path(root, STAGING_FILE_NAME)
}

fn default_slot_state(root: &str) -> SlotState {
    SlotState {
        image_path: join_path(&slots_directory(root), "emulator-running.bin"),
        version: String::from(DEFAULT_RUNNING_VERSION),
    }
}

fn default_persisted_state(root: &str) -> PersistedOtaState {
    let running = default_slot_state(root);

    PersistedOtaState {
        configured: running.clone(),
        running,
        last_invalid: None,
        force_next_validation_failure: false,
    }
}

fn to_c_string(value: &str) -> Result<CString, c_int> {
    CString::new(value).map_err(|_| libc::EINVAL)
}

fn create_dir_all(path: &str) -> Result<(), c_int> {
    if path.is_empty() {
        return Ok(());
    }

    let mut current = String::new();
    if path.starts_with('/') {
        current.push('/');
    }

    for component in path.split('/').filter(|component| !component.is_empty()) {
        if !current.is_empty() && !current.ends_with('/') {
            current.push('/');
        }
        current.push_str(component);

        let current_path = to_c_string(&current)?;
        let result = unsafe { host_forward::mkdir(current_path.as_ptr(), 0o755 as mode_t) };
        if result != 0 {
            let errno = current_errno();
            if errno != libc::EEXIST {
                return Err(errno);
            }
        }
    }

    Ok(())
}

fn open_read_only(path: &str) -> Result<FileDescriptor, c_int> {
    let path = to_c_string(path)?;
    let fd = unsafe { host_forward::open(path.as_ptr(), libc::O_RDONLY) };
    if fd < 0 {
        return Err(current_errno());
    }

    Ok(FileDescriptor(fd))
}

fn open_write_truncate(path: &str, mode: mode_t) -> Result<FileDescriptor, c_int> {
    let path = to_c_string(path)?;
    let fd = unsafe {
        host_forward::open(
            path.as_ptr(),
            libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
            mode,
        )
    };
    if fd < 0 {
        return Err(current_errno());
    }

    Ok(FileDescriptor(fd))
}

fn read_file_bytes(path: &str) -> Result<Vec<u8>, c_int> {
    let fd = open_read_only(path)?;
    let mut content = Vec::new();
    let mut chunk = [0u8; 4096];

    loop {
        let read = unsafe { host_forward::read(fd.0, chunk.as_mut_ptr().cast(), chunk.len()) };
        if read < 0 {
            return Err(current_errno());
        }
        if read == 0 {
            break;
        }

        content.extend_from_slice(&chunk[..read as usize]);
    }

    Ok(content)
}

fn write_all_fd(fd: c_int, content: &[u8]) -> Result<(), c_int> {
    let mut written = 0usize;
    while written < content.len() {
        let count = unsafe {
            host_forward::write(
                fd,
                content[written..].as_ptr().cast(),
                content.len() - written,
            )
        };
        if count <= 0 {
            let errno = current_errno();
            return Err(if errno == 0 { libc::EIO } else { errno });
        }

        written += count as usize;
    }

    Ok(())
}

fn write_file_bytes(path: &str, content: &[u8]) -> Result<(), WriteFileError> {
    let fd = open_write_truncate(path, 0o644 as mode_t).map_err(WriteFileError::Open)?;
    write_all_fd(fd.0, content).map_err(WriteFileError::Write)
}

fn remove_file(path: &str) -> Result<(), c_int> {
    let path = to_c_string(path)?;
    let result = unsafe { host_forward::remove(path.as_ptr()) };
    if result == 0 {
        return Ok(());
    }

    Err(current_errno())
}

fn rename_file(old_path: &str, new_path: &str) -> Result<(), c_int> {
    let old_path = to_c_string(old_path)?;
    let new_path = to_c_string(new_path)?;
    let result = unsafe { host_forward::rename(old_path.as_ptr(), new_path.as_ptr()) };
    if result == 0 {
        return Ok(());
    }

    Err(current_errno())
}

fn load_persisted_state(root: &str) -> PersistedOtaState {
    let default_state = default_persisted_state(root);
    let path = state_file_path(root);

    let content = match read_file_bytes(&path) {
        Ok(content) => content,
        Err(errno) if errno == libc::ENOENT => {
            let _ = save_persisted_state(root, &default_state);
            return default_state;
        }
        Err(_) => return default_state,
    };

    serde_json::from_slice(&content).unwrap_or(default_state)
}

fn save_persisted_state(root: &str, state: &PersistedOtaState) -> Result<(), c_int> {
    create_dir_all(root)?;
    create_dir_all(&slots_directory(root))?;

    let serialized = serde_json::to_vec_pretty(state).map_err(|_| libc::EINVAL)?;
    let temporary_path = temporary_state_file_path(root);
    write_file_bytes(&temporary_path, &serialized).map_err(|error| match error {
        WriteFileError::Open(errno) | WriteFileError::Write(errno) => errno,
    })?;
    rename_file(&temporary_path, &state_file_path(root))
}

fn persist_runtime(runtime: &OtaRuntime) -> Result<(), c_int> {
    save_persisted_state(
        &ota_root_directory(),
        &PersistedOtaState {
            running: runtime.running.clone(),
            configured: runtime.configured.clone(),
            last_invalid: runtime.last_invalid.clone(),
            force_next_validation_failure: runtime.force_next_validation_failure,
        },
    )
}

fn session_handle() -> ota_handle_t {
    (&OTA_SESSION_HANDLE as *const u8)
        .cast_mut()
        .cast::<ota_session_t>()
}

fn is_valid_handle(session: ota_handle_t) -> bool {
    !session.is_null()
        && ptr::eq(
            session.cast::<u8>().cast_const(),
            &OTA_SESSION_HANDLE as *const u8,
        )
}

fn allocate_c_string(value: &str) -> *mut c_char {
    let Some(allocation_size) = value.len().checked_add(1) else {
        return ptr::null_mut();
    };

    let buffer = unsafe { libc::malloc(allocation_size) }.cast::<c_char>();
    if buffer.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        ptr::copy_nonoverlapping(value.as_ptr().cast::<c_char>(), buffer, value.len());
        *buffer.add(value.len()) = 0;
    }

    buffer
}

fn next_slot_image_path(root: &str) -> String {
    let unique = {
        let mut timestamp = MaybeUninit::<libc::timespec>::uninit();
        if unsafe { libc::clock_gettime(libc::CLOCK_REALTIME, timestamp.as_mut_ptr()) } == 0 {
            let timestamp = unsafe { timestamp.assume_init() };
            ((timestamp.tv_sec as u128) * 1_000_000_000u128) + (timestamp.tv_nsec as u128)
        } else {
            0
        }
    };

    join_path(&slots_directory(root), &format!("image-{unique}.bin"))
}

fn is_valid_version_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'+')
}

fn extract_esp_app_version_from_bytes(image: &[u8]) -> Result<String, c_int> {
    let magic = ESP_APP_DESC_MAGIC_WORD.to_le_bytes();
    let search_limit = image.len().min(ESP_APP_DESC_SEARCH_LIMIT);
    if search_limit < ESP_APP_DESC_VERSION_OFFSET + ESP_APP_DESC_VERSION_LENGTH {
        return Err(libc::EINVAL);
    }

    for start in 0..=search_limit.saturating_sub(magic.len()) {
        if start % ESP_APP_DESC_ALIGNMENT != 0 {
            continue;
        }
        if image[start..start + magic.len()] != magic {
            continue;
        }

        let version_start = start + ESP_APP_DESC_VERSION_OFFSET;
        let version_end = version_start + ESP_APP_DESC_VERSION_LENGTH;
        if version_end > search_limit {
            continue;
        }

        let raw_version = &image[version_start..version_end];
        let Some(null_terminator) = raw_version.iter().position(|&byte| byte == 0) else {
            continue;
        };
        if null_terminator == 0 {
            continue;
        }

        let candidate = &raw_version[..null_terminator];
        if !candidate.iter().all(|byte| is_valid_version_byte(*byte)) {
            continue;
        }
        if !candidate.iter().any(u8::is_ascii_digit) {
            continue;
        }

        let version = str::from_utf8(candidate).map_err(|_| libc::EINVAL)?;
        return Ok(String::from(version));
    }

    Err(libc::EINVAL)
}

fn extract_esp_app_version(image_path: &str) -> Result<String, c_int> {
    extract_esp_app_version_from_bytes(&read_file_bytes(image_path)?)
}

pub(crate) fn abort_task_owned_ota_session(session: ota_handle_t) {
    let _ = ota_session_abort_inner(session);
}

pub(crate) fn ota_session_open_inner() -> ota_handle_t {
    let mut runtime = ota_runtime();
    if runtime.open {
        return ptr::null_mut();
    }

    let root = ota_root_directory();
    if create_dir_all(&slots_directory(&root)).is_err() {
        runtime.open = false;
        runtime.error = true;
        runtime.staging = None;
        return ptr::null_mut();
    }

    let staging_path = staging_file_path(&root);
    match open_write_truncate(&staging_path, 0o644 as mode_t) {
        Ok(fd) => {
            runtime.open = true;
            runtime.error = false;
            runtime.staging = Some(StagingSession {
                fd,
                path: staging_path,
            });
            let session = session_handle();
            register_ota_session(current_managed_task_pid(), session);
            session
        }
        Err(_) => {
            runtime.open = false;
            runtime.error = true;
            runtime.staging = None;
            ptr::null_mut()
        }
    }
}

pub(crate) fn ota_write_inner(
    session: ota_handle_t,
    buffer: *mut c_void,
    block_size: c_int,
) -> bool {
    if !is_valid_handle(session) {
        return false;
    }

    let mut runtime = ota_runtime();
    if runtime.error {
        return false;
    }

    if block_size < 0 || (buffer.is_null() && block_size > 0) {
        drop(runtime);
        return ota_session_abort_inner(session);
    }

    let Some(staging) = runtime.staging.as_mut() else {
        return false;
    };

    let block = unsafe {
        core::slice::from_raw_parts(
            buffer.cast::<u8>(),
            usize::try_from(block_size).unwrap_or_default(),
        )
    };

    if write_all_fd(staging.fd.0, block).is_err() {
        drop(runtime);
        return ota_session_abort_inner(session);
    }

    true
}

pub(crate) fn ota_session_commit_inner(session: ota_handle_t) -> bool {
    if !is_valid_handle(session) {
        return false;
    }

    let mut runtime = ota_runtime();
    if runtime.error {
        return false;
    }

    let Some(staging) = runtime.staging.as_ref() else {
        return false;
    };

    if unsafe { libc::fsync(staging.fd.0) } != 0 {
        return false;
    }

    let root = ota_root_directory();
    let staged_image_path = staging.path.clone();
    let version = match extract_esp_app_version(&staged_image_path) {
        Ok(version) => version,
        Err(_) => return false,
    };

    let slot_path = next_slot_image_path(&root);
    if rename_file(&staged_image_path, &slot_path).is_err() {
        return false;
    }

    runtime.configured = SlotState {
        image_path: slot_path,
        version,
    };
    release_ota_session(current_managed_task_pid(), session);
    runtime.open = false;
    runtime.error = false;
    runtime.staging = None;

    persist_runtime(&runtime).is_ok()
}

pub(crate) fn ota_session_abort_inner(session: ota_handle_t) -> bool {
    if !is_valid_handle(session) {
        return false;
    }

    let mut runtime = ota_runtime();
    let staging_path = runtime.staging.as_ref().map(|staging| staging.path.clone());

    release_ota_session(current_managed_task_pid(), session);
    runtime.staging = None;
    runtime.open = false;
    runtime.error = true;

    if let Some(staging_path) = staging_path {
        let _ = remove_file(&staging_path);
    }

    true
}

pub(crate) fn ota_get_running_version_inner(version: *mut *mut c_char) -> bool {
    if version.is_null() {
        return false;
    }

    let runtime = ota_runtime();
    let allocated_version = allocate_c_string(&runtime.running.version);
    if allocated_version.is_null() {
        return false;
    }

    unsafe {
        *version = allocated_version;
    }

    true
}

pub(crate) fn ota_get_invalid_version_inner(version: *mut *mut c_char) -> bool {
    if version.is_null() {
        return false;
    }

    let runtime = ota_runtime();
    let Some(last_invalid) = runtime.last_invalid.as_ref() else {
        return false;
    };

    let allocated_version = allocate_c_string(&last_invalid.version);
    if allocated_version.is_null() {
        return false;
    }

    unsafe {
        *version = allocated_version;
    }

    true
}

#[cfg(test)]
fn set_ota_root_directory_for_tests(root: impl Into<PathBuf>) -> TestOtaRootGuard {
    let mut value = TEST_OTA_ROOT_DIRECTORY.lock();
    let previous = value.replace(root.into());
    reset_ota_runtime_for_tests();
    TestOtaRootGuard { previous }
}

#[cfg(test)]
struct TestOtaRootGuard {
    previous: Option<PathBuf>,
}

#[cfg(test)]
impl Drop for TestOtaRootGuard {
    fn drop(&mut self) {
        reset_ota_runtime_for_tests();
        let mut value = TEST_OTA_ROOT_DIRECTORY.lock();
        *value = self.previous.take();
    }
}

#[cfg(test)]
fn reset_ota_runtime_for_tests() {
    let mut runtime = OTA_RUNTIME.lock();
    *runtime = OtaRuntime::default();
}

#[cfg(test)]
#[derive(Debug, PartialEq, Eq)]
struct OtaRuntimeSnapshot {
    open: bool,
    error: bool,
    running_version: String,
    configured_version: String,
    last_invalid_version: Option<String>,
}

#[cfg(test)]
fn ota_runtime_snapshot() -> OtaRuntimeSnapshot {
    let runtime = ota_runtime();
    OtaRuntimeSnapshot {
        open: runtime.open,
        error: runtime.error,
        running_version: runtime.running.version.clone(),
        configured_version: runtime.configured.version.clone(),
        last_invalid_version: runtime
            .last_invalid
            .as_ref()
            .map(|slot| slot.version.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::misc::runtime::{
        GlobalTestRuntimeGuard, current_task_pid, lock_global_test_runtime,
        ota_session_count_for_task, register_ota_session, register_task,
        reset_runtime_for_tests, set_current_task_pid, task_exited,
    };
    use crate::ota::{
        ota_get_invalid_version, ota_get_running_version, ota_session_abort,
        ota_session_commit, ota_session_open, ota_write,
    };
    use std::vec;

    struct TestOtaDirectory {
        root: PathBuf,
        _lock: GlobalTestRuntimeGuard,
        _guard: TestOtaRootGuard,
    }

    impl TestOtaDirectory {
        fn new() -> Self {
            let lock = lock_global_test_runtime();
            let suffix = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let root = std::env::temp_dir()
                .join(format!("why2025-ota-test-{}-{suffix}", std::process::id()));
            let guard = set_ota_root_directory_for_tests(root.clone());

            Self {
                root,
                _lock: lock,
                _guard: guard,
            }
        }

        fn create_image(&self, version: &str) -> Vec<u8> {
            let mut image = vec![0xE9, 0x03, 0x02, 0x00, 0, 0, 0, 0];
            image.resize(96, 0);
            image.extend_from_slice(&ESP_APP_DESC_MAGIC_WORD.to_le_bytes());
            image.extend_from_slice(&0u32.to_le_bytes());
            image.extend_from_slice(&0u32.to_le_bytes());
            image.extend_from_slice(&0u32.to_le_bytes());
            let mut version_bytes = [0u8; ESP_APP_DESC_VERSION_LENGTH];
            version_bytes[..version.len()].copy_from_slice(version.as_bytes());
            image.extend_from_slice(&version_bytes);
            image
        }

        fn create_invalid_image_without_descriptor(&self) -> Vec<u8> {
            let mut image = vec![0xE9, 0x03, 0x02, 0x00, 0, 0, 0, 0];
            image.resize(256, 0xA5);
            image
        }

        fn create_image_with_invalid_version_bytes(&self) -> Vec<u8> {
            let mut image = self.create_image("1.0.0");
            let version_start = 96 + ESP_APP_DESC_VERSION_OFFSET;
            image[version_start] = 0x1F;
            image
        }

        fn create_image_beyond_search_window(&self, version: &str) -> Vec<u8> {
            let mut image = vec![0x00; ESP_APP_DESC_SEARCH_LIMIT + 8];
            image.extend_from_slice(&self.create_image(version));
            image
        }

        fn corrupt_state_file(&self, content: &[u8]) {
            fs::create_dir_all(&self.root).unwrap();
            fs::write(self.root.join(STATE_FILE_NAME), content).unwrap();
        }

        fn set_force_next_validation_failure(&self, enabled: bool) {
            let mut runtime = ota_runtime();
            runtime.force_next_validation_failure = enabled;
            persist_runtime(&runtime).unwrap();
        }
    }

    impl Drop for TestOtaDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
            reset_runtime_for_tests();
        }
    }

    #[test]
    fn parser_rejects_images_without_app_descriptor() {
        let test_directory = TestOtaDirectory::new();
        let image = test_directory.create_invalid_image_without_descriptor();
        assert!(extract_esp_app_version_from_bytes(&image).is_err());
    }

    #[test]
    fn parser_rejects_invalid_version_bytes() {
        let test_directory = TestOtaDirectory::new();
        let image = test_directory.create_image_with_invalid_version_bytes();
        assert!(extract_esp_app_version_from_bytes(&image).is_err());
    }

    #[test]
    fn parser_ignores_descriptors_beyond_search_window() {
        let test_directory = TestOtaDirectory::new();
        let image = test_directory.create_image_beyond_search_window("9.9.9");
        assert!(extract_esp_app_version_from_bytes(&image).is_err());
    }

    #[test]
    fn committed_image_becomes_running_after_restart() {
        let test_directory = TestOtaDirectory::new();
        let session = ota_session_open();
        assert!(!session.is_null());

        let image = test_directory.create_image("1.2.3");
        assert!(ota_write(
            session,
            image.as_ptr().cast::<c_void>().cast_mut(),
            image.len() as i32,
        ));
        assert!(ota_session_commit(session));

        let mut running = ptr::null_mut();
        assert!(ota_get_running_version(&mut running));
        let running_ptr = running;
        let running = unsafe { CStr::from_ptr(running) }
            .to_string_lossy()
            .into_owned();
        unsafe { libc::free(running_ptr.cast::<c_void>()) };
        assert_eq!(running, DEFAULT_RUNNING_VERSION);

        reset_ota_runtime_for_tests();

        let mut running = ptr::null_mut();
        assert!(ota_get_running_version(&mut running));
        let running_ptr = running;
        let running = unsafe { CStr::from_ptr(running) }
            .to_string_lossy()
            .into_owned();
        unsafe { libc::free(running_ptr.cast::<c_void>()) };
        assert_eq!(running, "1.2.3");
    }

    #[test]
    fn forced_validation_failure_rolls_back_and_records_invalid_version() {
        let test_directory = TestOtaDirectory::new();
        let session = ota_session_open();
        assert!(!session.is_null());

        let image = test_directory.create_image("2.0.0");
        assert!(ota_write(
            session,
            image.as_ptr().cast::<c_void>().cast_mut(),
            image.len() as i32,
        ));
        assert!(ota_session_commit(session));

        test_directory.set_force_next_validation_failure(true);
        reset_ota_runtime_for_tests();

        let mut running = ptr::null_mut();
        assert!(ota_get_running_version(&mut running));
        let running_ptr = running;
        let running = unsafe { CStr::from_ptr(running) }
            .to_string_lossy()
            .into_owned();
        unsafe { libc::free(running_ptr.cast::<c_void>()) };
        assert_eq!(running, DEFAULT_RUNNING_VERSION);

        let mut invalid = ptr::null_mut();
        assert!(ota_get_invalid_version(&mut invalid));
        let invalid_ptr = invalid;
        let invalid = unsafe { CStr::from_ptr(invalid) }
            .to_string_lossy()
            .into_owned();
        unsafe { libc::free(invalid_ptr.cast::<c_void>()) };
        assert_eq!(invalid, "2.0.0");
    }

    #[test]
    fn commit_failure_keeps_session_open_until_abort() {
        let test_directory = TestOtaDirectory::new();
        let session = ota_session_open();
        assert!(!session.is_null());

        let image = test_directory.create_invalid_image_without_descriptor();
        assert!(ota_write(
            session,
            image.as_ptr().cast::<c_void>().cast_mut(),
            image.len() as i32,
        ));
        assert!(!ota_session_commit(session));
        assert_eq!(
            ota_runtime_snapshot(),
            OtaRuntimeSnapshot {
                open: true,
                error: false,
                running_version: String::from(DEFAULT_RUNNING_VERSION),
                configured_version: String::from(DEFAULT_RUNNING_VERSION),
                last_invalid_version: None,
            }
        );
        assert!(ota_session_open().is_null());

        assert!(ota_session_abort(session));
        assert_eq!(
            ota_runtime_snapshot(),
            OtaRuntimeSnapshot {
                open: false,
                error: true,
                running_version: String::from(DEFAULT_RUNNING_VERSION),
                configured_version: String::from(DEFAULT_RUNNING_VERSION),
                last_invalid_version: None,
            }
        );
        assert!(!ota_session_open().is_null());
    }

    #[test]
    fn version_getters_return_independent_allocations() {
        let test_directory = TestOtaDirectory::new();
        let session = ota_session_open();
        assert!(!session.is_null());

        let image = test_directory.create_image("4.5.6");
        assert!(ota_write(
            session,
            image.as_ptr().cast::<c_void>().cast_mut(),
            image.len() as i32,
        ));
        assert!(ota_session_commit(session));
        reset_ota_runtime_for_tests();

        let mut first = ptr::null_mut();
        let mut second = ptr::null_mut();
        assert!(ota_get_running_version(&mut first));
        assert!(ota_get_running_version(&mut second));
        assert_ne!(first, second);

        let first_value = unsafe { CStr::from_ptr(first) }
            .to_string_lossy()
            .into_owned();
        let second_value = unsafe { CStr::from_ptr(second) }
            .to_string_lossy()
            .into_owned();
        unsafe {
            libc::free(first.cast::<c_void>());
            libc::free(second.cast::<c_void>());
        }

        assert_eq!(first_value, "4.5.6");
        assert_eq!(second_value, "4.5.6");
    }

    #[test]
    fn corrupted_persisted_state_falls_back_to_default() {
        let test_directory = TestOtaDirectory::new();
        test_directory.corrupt_state_file(br#"{ definitely not valid json"#);
        reset_ota_runtime_for_tests();

        let mut running = ptr::null_mut();
        assert!(ota_get_running_version(&mut running));
        let running_ptr = running;
        let running = unsafe { CStr::from_ptr(running) }
            .to_string_lossy()
            .into_owned();
        unsafe { libc::free(running_ptr.cast::<c_void>()) };
        assert_eq!(running, DEFAULT_RUNNING_VERSION);
    }

    #[test]
    fn task_exit_aborts_owned_ota_session() {
        let _test_directory = TestOtaDirectory::new();
        let parent_pid = current_task_pid();
        let pid = register_task(Some(parent_pid));
        set_current_task_pid(pid);

        let session = ota_session_open();
        assert!(!session.is_null());
        assert!(ota_runtime_snapshot().open);

        task_exited(pid);
        set_current_task_pid(parent_pid);

        assert_eq!(
            ota_runtime_snapshot(),
            OtaRuntimeSnapshot {
                open: false,
                error: true,
                running_version: String::from(DEFAULT_RUNNING_VERSION),
                configured_version: String::from(DEFAULT_RUNNING_VERSION),
                last_invalid_version: None,
            }
        );
        assert!(!ota_session_open().is_null());
    }

    #[test]
    fn kernel_context_session_is_not_resource_tracked() {
        let _test_directory = TestOtaDirectory::new();
        reset_runtime_for_tests();
        set_current_task_pid(0);

        let session = ota_session_open();
        assert!(!session.is_null());
        assert_eq!(ota_session_count_for_task(1), 0);

        assert!(ota_session_abort(session));
    }

    #[test]
    fn commit_only_releases_current_task_ownership() {
        let test_directory = TestOtaDirectory::new();
        reset_runtime_for_tests();

        let owner_pid = register_task(None);
        let other_pid = register_task(None);
        set_current_task_pid(owner_pid);

        let session = ota_session_open();
        assert!(!session.is_null());
        assert_eq!(ota_session_count_for_task(owner_pid), 1);

        register_ota_session(Some(other_pid), session);
        assert_eq!(ota_session_count_for_task(other_pid), 1);

        let image = test_directory.create_image("7.8.9");
        assert!(ota_write(
            session,
            image.as_ptr().cast::<c_void>().cast_mut(),
            image.len() as i32,
        ));
        assert!(ota_session_commit(session));

        assert_eq!(ota_session_count_for_task(owner_pid), 0);
        assert_eq!(ota_session_count_for_task(other_pid), 1);

        set_current_task_pid(0);
        reset_runtime_for_tests();
    }
}