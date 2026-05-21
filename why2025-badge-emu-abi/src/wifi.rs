use crate::{host_forward, runtime, types::*};
use alloc::{
    boxed::Box,
    ffi::CString,
    string::String,
    vec::Vec,
};
use core::{
    ffi::{c_char, c_int, CStr},
    ptr,
    time::Duration,
};
use serde::{Deserialize, Serialize};
use spin::{Lazy, Mutex};

const BASE_DIRECTORY: &str = ".why2025-badge/data";
const BASE_DIRECTORY_ENV_VAR: &[u8] = b"WHY2025_BADGE_EMULATED_BASE_DIRECTORY\0";
const HOME_ENV_VAR: &[u8] = b"HOME\0";
const WIFI_DIRECTORY_NAME: &str = "wifi";
const STATE_FILE_NAME: &str = "state.json";
const TEMP_STATE_FILE_NAME: &str = "state.tmp";
const ZERO_MAC_ADDRESS: &str = "00:00:00:00:00:00";
const DEFAULT_SSID: &str = "WHY2025-open";
const DEFAULT_PASSWORD: &str = "";
const MAX_SCAN_RESULTS: usize = 20;
const MAX_CONFIG_SSID_BYTES: usize = 31;
const MAX_CONFIG_PASSWORD_BYTES: usize = 63;
const MAX_STATION_SSID_BYTES: usize = 32;
const MIN_SCAN_INTERVAL: Duration = Duration::from_secs(10);
const MIN_SCAN_INTERVAL_EMPTY: Duration = Duration::from_secs(1);

static WIFI_RUNTIME: Lazy<Mutex<WifiRuntime>> = Lazy::new(|| Mutex::new(WifiRuntime::default()));

#[cfg(test)]
static TEST_BASE_DIRECTORY: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
#[cfg(test)]
static TEST_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct PersistedCredentials {
    ssid: String,
    password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct PersistedWifiFixture {
    ssid: String,
    bssid: [u8; 6],
    primary: c_int,
    secondary: c_int,
    rssi: i8,
    authmode: u32,
    wps: bool,
    password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct PersistedWifiState {
    credentials: PersistedCredentials,
    fixtures: Vec<PersistedWifiFixture>,
    mac_address: Option<String>,
}

#[derive(Debug, Clone)]
struct HostWifiStation {
    bssid: mac_address_t,
    ssid: [c_char; MAX_STATION_SSID_BYTES + 1],
    primary: c_int,
    secondary: c_int,
    rssi: i8,
    authmode: wifi_auth_mode_t,
    wps: bool,
}

#[derive(Debug)]
struct WifiRuntime {
    initialized: bool,
    connection_status: wifi_connection_status_t,
    connection_status_want: wifi_connection_status_t,
    current_station: Option<HostWifiStation>,
    scan_results: Vec<HostWifiStation>,
    last_scan_at: Option<Duration>,
    persisted: PersistedWifiState,
    cached_mac_address: Option<CString>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PersistStateError {
    StoreOpen(c_int),
    Write(c_int),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WriteFileError {
    Open(c_int),
    Write(c_int),
}

impl Default for WifiRuntime {
    fn default() -> Self {
        Self {
            initialized: false,
            connection_status: wifi_connection_status_t::WIFI_DISCONNECTED,
            connection_status_want: wifi_connection_status_t::WIFI_DISCONNECTED,
            current_station: None,
            scan_results: Vec::new(),
            last_scan_at: None,
            persisted: default_persisted_state(),
            cached_mac_address: None,
        }
    }
}

impl HostWifiStation {
    fn from_fixture(fixture: &PersistedWifiFixture) -> Self {
        Self {
            bssid: fixture.bssid,
            ssid: encode_ssid(&fixture.ssid),
            primary: fixture.primary,
            secondary: fixture.secondary,
            rssi: fixture.rssi,
            authmode: authmode_from_raw(fixture.authmode),
            wps: fixture.wps,
        }
    }
}

#[cfg(not(test))]
struct FileDescriptor(c_int);

#[cfg(not(test))]
impl Drop for FileDescriptor {
    fn drop(&mut self) {
        unsafe {
            let _ = host_forward::close(self.0);
        }
    }
}

struct DirectoryHandle(*mut DIR);

impl Drop for DirectoryHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = host_forward::closedir(self.0);
        }
    }
}

fn wifi_runtime() -> spin::MutexGuard<'static, WifiRuntime> {
    let mut runtime = WIFI_RUNTIME.lock();
    initialize_runtime(&mut runtime);
    runtime
}

fn initialize_runtime(runtime: &mut WifiRuntime) {
    if runtime.initialized {
        return;
    }

    runtime.persisted = load_persisted_state(&wifi_root_directory());
    runtime.initialized = true;
}

fn current_errno() -> c_int {
    unsafe { *runtime::__errno() }
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
        alloc::format!("{base}{tail}")
    } else {
        alloc::format!("{base}/{tail}")
    }
}

fn base_directory() -> String {
    #[cfg(test)]
    if let Some(base_directory) = TEST_BASE_DIRECTORY.lock().clone() {
        return base_directory;
    }

    if let Some(base_directory) = host_env_string(BASE_DIRECTORY_ENV_VAR)
        && !base_directory.is_empty()
    {
        return base_directory;
    }

    if let Some(home_directory) = host_env_string(HOME_ENV_VAR)
        && !home_directory.is_empty()
    {
        return join_path(&home_directory, BASE_DIRECTORY);
    }

    String::from(BASE_DIRECTORY)
}

fn wifi_root_directory() -> String {
    join_path(&base_directory(), WIFI_DIRECTORY_NAME)
}

fn state_file_path(root: &str) -> String {
    join_path(root, STATE_FILE_NAME)
}

fn temporary_state_file_path(root: &str) -> String {
    join_path(root, TEMP_STATE_FILE_NAME)
}

fn to_c_string(value: &str) -> Result<CString, c_int> {
    CString::new(value).map_err(|_| libc::EINVAL)
}

#[cfg(not(test))]
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

#[cfg(test)]
fn create_dir_all(path: &str) -> Result<(), c_int> {
    std::fs::create_dir_all(path).map_err(|error| error.raw_os_error().unwrap_or(libc::EIO))
}

#[cfg(not(test))]
fn read_file_bytes(path: &str) -> Result<Vec<u8>, c_int> {
    let path = to_c_string(path)?;
    let fd = unsafe { host_forward::open(path.as_ptr(), libc::O_RDONLY) };
    if fd < 0 {
        return Err(current_errno());
    }
    let fd = FileDescriptor(fd);

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

#[cfg(test)]
fn read_file_bytes(path: &str) -> Result<Vec<u8>, c_int> {
    std::fs::read(path).map_err(|error| error.raw_os_error().unwrap_or(libc::EIO))
}

fn read_file_string(path: &str) -> Option<String> {
    read_file_bytes(path)
        .ok()
        .map(|content| String::from_utf8_lossy(&content).into_owned())
}

#[cfg(not(test))]
fn write_file_bytes(path: &str, content: &[u8]) -> Result<(), WriteFileError> {
    let path = to_c_string(path).map_err(WriteFileError::Open)?;
    let fd = unsafe {
        host_forward::open(
            path.as_ptr(),
            libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC,
            0o644 as mode_t,
        )
    };
    if fd < 0 {
        return Err(WriteFileError::Open(current_errno()));
    }
    let fd = FileDescriptor(fd);

    let mut written = 0;
    while written < content.len() {
        let count = unsafe {
            host_forward::write(
                fd.0,
                content[written..].as_ptr().cast(),
                content.len() - written,
            )
        };
        if count <= 0 {
            let errno = current_errno();
            return Err(WriteFileError::Write(if errno == 0 {
                libc::EIO
            } else {
                errno
            }));
        }
        written += count as usize;
    }

    Ok(())
}

#[cfg(test)]
fn write_file_bytes(path: &str, content: &[u8]) -> Result<(), WriteFileError> {
    std::fs::write(path, content).map_err(|error| {
        let errno = error.raw_os_error().unwrap_or(libc::EIO);
        match error.kind() {
            std::io::ErrorKind::NotFound
            | std::io::ErrorKind::PermissionDenied
            | std::io::ErrorKind::IsADirectory => WriteFileError::Open(errno),
            _ => WriteFileError::Write(errno),
        }
    })
}

fn default_persisted_state() -> PersistedWifiState {
    PersistedWifiState {
        credentials: PersistedCredentials {
            ssid: String::from(DEFAULT_SSID),
            password: String::from(DEFAULT_PASSWORD),
        },
        fixtures: alloc::vec![
            PersistedWifiFixture {
                ssid: String::from(DEFAULT_SSID),
                bssid: [0x57, 0x48, 0x59, 0x25, 0x00, 0x01],
                primary: 1,
                secondary: 0,
                rssi: -42,
                authmode: wifi_auth_mode_t::WIFI_AUTH_OPEN as u32,
                wps: false,
                password: String::new(),
            },
            PersistedWifiFixture {
                ssid: String::from("WHY2025-secure"),
                bssid: [0x57, 0x48, 0x59, 0x25, 0x00, 0x02],
                primary: 6,
                secondary: 0,
                rssi: -55,
                authmode: wifi_auth_mode_t::WIFI_AUTH_WPA2_PSK as u32,
                wps: true,
                password: String::from("why2025"),
            },
            PersistedWifiFixture {
                ssid: String::from("WHY2025-mesh"),
                bssid: [0x57, 0x48, 0x59, 0x25, 0x00, 0x03],
                primary: 11,
                secondary: 0,
                rssi: -61,
                authmode: wifi_auth_mode_t::WIFI_AUTH_WPA2_WPA3_PSK as u32,
                wps: false,
                password: String::from("mesh-password"),
            },
        ],
        mac_address: None,
    }
}

fn load_persisted_state(root: &str) -> PersistedWifiState {
    let default_state = default_persisted_state();
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

fn save_persisted_state(root: &str, state: &PersistedWifiState) -> Result<(), PersistStateError> {
    create_dir_all(root).map_err(PersistStateError::StoreOpen)?;

    let content =
        serde_json::to_vec_pretty(state).map_err(|_| PersistStateError::Write(libc::EINVAL))?;
    let temporary_path = temporary_state_file_path(root);
    write_file_bytes(&temporary_path, &content).map_err(|error| match error {
        WriteFileError::Open(errno) => PersistStateError::StoreOpen(errno),
        WriteFileError::Write(errno) => PersistStateError::Write(errno),
    })?;

    let temporary_path =
        to_c_string(&temporary_path).map_err(PersistStateError::Write)?;
    let final_path =
        to_c_string(&state_file_path(root)).map_err(PersistStateError::Write)?;
    let result = unsafe { host_forward::rename(temporary_path.as_ptr(), final_path.as_ptr()) };
    if result != 0 {
        return Err(PersistStateError::Write(current_errno()));
    }

    Ok(())
}

fn persist_runtime(state: &PersistedWifiState) -> Result<(), PersistStateError> {
    save_persisted_state(&wifi_root_directory(), state)
}

fn reload_persisted_state(runtime: &mut WifiRuntime) {
    runtime.persisted = load_persisted_state(&wifi_root_directory());
}

fn encode_ssid(ssid: &str) -> [c_char; MAX_STATION_SSID_BYTES + 1] {
    let mut encoded = [0; MAX_STATION_SSID_BYTES + 1];
    let bytes = ssid.as_bytes();
    let len = bytes.len().min(MAX_STATION_SSID_BYTES);

    for (destination, source) in encoded.iter_mut().zip(bytes.iter()).take(len) {
        *destination = *source as c_char;
    }

    encoded
}

fn authmode_from_raw(raw: u32) -> wifi_auth_mode_t {
    match raw {
        x if x == wifi_auth_mode_t::WIFI_AUTH_NONE as u32 => wifi_auth_mode_t::WIFI_AUTH_NONE,
        x if x == wifi_auth_mode_t::WIFI_AUTH_OPEN as u32 => wifi_auth_mode_t::WIFI_AUTH_OPEN,
        x if x == wifi_auth_mode_t::WIFI_AUTH_WEP as u32 => wifi_auth_mode_t::WIFI_AUTH_WEP,
        x if x == wifi_auth_mode_t::WIFI_AUTH_WPA_PSK as u32 => wifi_auth_mode_t::WIFI_AUTH_WPA_PSK,
        x if x == wifi_auth_mode_t::WIFI_AUTH_WPA2_PSK as u32 => {
            wifi_auth_mode_t::WIFI_AUTH_WPA2_PSK
        }
        x if x == wifi_auth_mode_t::WIFI_AUTH_WPA_WPA2_PSK as u32 => {
            wifi_auth_mode_t::WIFI_AUTH_WPA_WPA2_PSK
        }
        x if x == wifi_auth_mode_t::WIFI_AUTH_WPA2_ENTERPRISE as u32 => {
            wifi_auth_mode_t::WIFI_AUTH_WPA2_ENTERPRISE
        }
        x if x == wifi_auth_mode_t::WIFI_AUTH_WPA3_PSK as u32 => {
            wifi_auth_mode_t::WIFI_AUTH_WPA3_PSK
        }
        x if x == wifi_auth_mode_t::WIFI_AUTH_WPA2_WPA3_PSK as u32 => {
            wifi_auth_mode_t::WIFI_AUTH_WPA2_WPA3_PSK
        }
        x if x == wifi_auth_mode_t::WIFI_AUTH_WAPI_PSK as u32 => {
            wifi_auth_mode_t::WIFI_AUTH_WAPI_PSK
        }
        x if x == wifi_auth_mode_t::WIFI_AUTH_OWE as u32 => wifi_auth_mode_t::WIFI_AUTH_OWE,
        x if x == wifi_auth_mode_t::WIFI_AUTH_WPA3_ENT_192 as u32 => {
            wifi_auth_mode_t::WIFI_AUTH_WPA3_ENT_192
        }
        x if x == wifi_auth_mode_t::WIFI_AUTH_DPP as u32 => wifi_auth_mode_t::WIFI_AUTH_DPP,
        x if x == wifi_auth_mode_t::WIFI_AUTH_WPA3_ENTERPRISE as u32 => {
            wifi_auth_mode_t::WIFI_AUTH_WPA3_ENTERPRISE
        }
        x if x == wifi_auth_mode_t::WIFI_AUTH_WPA2_WPA3_ENTERPRISE as u32 => {
            wifi_auth_mode_t::WIFI_AUTH_WPA2_WPA3_ENTERPRISE
        }
        x if x == wifi_auth_mode_t::WIFI_AUTH_WPA_ENTERPRISE as u32 => {
            wifi_auth_mode_t::WIFI_AUTH_WPA_ENTERPRISE
        }
        _ => wifi_auth_mode_t::WIFI_AUTH_NONE,
    }
}

fn read_truncated_string(value: *const c_char, max_bytes: usize) -> Option<String> {
    if value.is_null() {
        return None;
    }

    let bytes = unsafe { CStr::from_ptr(value) }.to_bytes();
    Some(String::from_utf8_lossy(&bytes[..bytes.len().min(max_bytes)]).into_owned())
}

fn monotonic_now() -> Option<Duration> {
    let mut timestamp = timespec {
        tv_sec: 0,
        tv_nsec: 0,
        __bindgen_padding_0: [0; 4],
    };
    let result = unsafe {
        host_forward::clock_gettime(libc::CLOCK_MONOTONIC as clockid_t, &mut timestamp)
    };

    if result != 0 || timestamp.tv_sec < 0 || timestamp.tv_nsec < 0 {
        return None;
    }

    Some(Duration::new(
        timestamp.tv_sec as u64,
        timestamp.tv_nsec as u32,
    ))
}

fn refresh_scan_results(runtime: &mut WifiRuntime) {
    if let Some(now) = monotonic_now() {
        let minimum_interval = if runtime.scan_results.is_empty() {
            MIN_SCAN_INTERVAL_EMPTY
        } else {
            MIN_SCAN_INTERVAL
        };

        if let Some(last_scan_at) = runtime.last_scan_at
            && now.saturating_sub(last_scan_at) < minimum_interval
        {
            return;
        }

        runtime.last_scan_at = Some(now);
    } else {
        runtime.last_scan_at = None;
    }

    reload_persisted_state(runtime);
    runtime.scan_results = runtime
        .persisted
        .fixtures
        .iter()
        .take(MAX_SCAN_RESULTS)
        .map(HostWifiStation::from_fixture)
        .collect();
}

fn box_station(station: &HostWifiStation) -> wifi_station_handle {
    Box::into_raw(Box::new(station.clone())).cast::<wifi_station>()
}

unsafe fn station_ref<'a>(station: wifi_station_handle) -> Option<&'a HostWifiStation> {
    unsafe { station.cast::<HostWifiStation>().as_ref() }
}

fn format_mac_address(bytes: [u8; 6]) -> String {
    alloc::format!(
        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]
    )
}

fn normalize_mac_address(value: &str) -> Option<String> {
    let mut bytes = [0u8; 6];
    let mut segments = value.trim().split(':');

    for byte in &mut bytes {
        let segment = segments.next()?;
        if segment.len() != 2 {
            return None;
        }
        *byte = u8::from_str_radix(segment, 16).ok()?;
    }

    if segments.next().is_some() {
        return None;
    }

    Some(format_mac_address(bytes))
}

fn discover_host_mac_address() -> Option<String> {
    let directory = unsafe { host_forward::opendir(c"/sys/class/net".as_ptr()) };
    if directory.is_null() {
        return None;
    }
    let _directory = DirectoryHandle(directory);

    loop {
        let entry = unsafe { host_forward::readdir(directory) };
        if entry.is_null() {
            return None;
        }

        let interface_name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        if matches!(interface_name.as_str(), "." | ".." | "lo") {
            continue;
        }

        let address_path = alloc::format!("/sys/class/net/{interface_name}/address");
        let Some(address) = read_file_string(&address_path) else {
            continue;
        };

        if let Some(mac_address) = normalize_mac_address(&address)
            && mac_address != ZERO_MAC_ADDRESS
        {
            return Some(mac_address);
        }
    }
}

fn wifi_get_connection_status_inner() -> wifi_connection_status_t {
    wifi_runtime().connection_status
}

fn wifi_get_connection_station_inner() -> wifi_station_handle {
    let runtime = wifi_runtime();
    if runtime.connection_status != wifi_connection_status_t::WIFI_CONNECTED {
        return ptr::null_mut();
    }

    runtime
        .current_station
        .as_ref()
        .map_or(ptr::null_mut(), box_station)
}

fn wifi_connect_inner() -> wifi_connection_status_t {
    let mut runtime = wifi_runtime();
    if runtime.connection_status == wifi_connection_status_t::WIFI_CONNECTED {
        return wifi_connection_status_t::WIFI_CONNECTED;
    }

    reload_persisted_state(&mut runtime);
    runtime.connection_status_want = wifi_connection_status_t::WIFI_CONNECTED;

    let credentials = runtime.persisted.credentials.clone();
    let fixture = runtime
        .persisted
        .fixtures
        .iter()
        .find(|fixture| fixture.ssid == credentials.ssid)
        .cloned();

    let outcome = match fixture {
        Some(fixture)
            if matches!(
                authmode_from_raw(fixture.authmode),
                wifi_auth_mode_t::WIFI_AUTH_OPEN | wifi_auth_mode_t::WIFI_AUTH_NONE
            ) || fixture.password == credentials.password =>
        {
            runtime.current_station = Some(HostWifiStation::from_fixture(&fixture));
            wifi_connection_status_t::WIFI_CONNECTED
        }
        Some(_) => {
            runtime.current_station = None;
            wifi_connection_status_t::WIFI_ERROR_WRONG_CREDENTIALS
        }
        None => {
            runtime.current_station = None;
            wifi_connection_status_t::WIFI_ERROR
        }
    };

    runtime.connection_status = outcome;
    outcome
}

fn wifi_disconnect_inner() -> wifi_connection_status_t {
    let mut runtime = wifi_runtime();
    runtime.connection_status_want = wifi_connection_status_t::WIFI_DISCONNECTED;

    if runtime.connection_status == wifi_connection_status_t::WIFI_DISCONNECTED {
        return wifi_connection_status_t::WIFI_DISCONNECTED;
    }

    if runtime.connection_status != wifi_connection_status_t::WIFI_CONNECTED {
        return runtime.connection_status;
    }

    runtime.connection_status = wifi_connection_status_t::WIFI_DISCONNECTED;
    runtime.current_station = None;
    wifi_connection_status_t::WIFI_DISCONNECTED
}

fn wifi_scan_free_station_inner(station: wifi_station_handle) {
    if station.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(station.cast::<HostWifiStation>()));
    }
}

fn wifi_scan_get_num_results_inner() -> c_int {
    let mut runtime = wifi_runtime();
    refresh_scan_results(&mut runtime);
    runtime.scan_results.len() as c_int
}

fn wifi_scan_get_result_inner(num: c_int) -> wifi_station_handle {
    if num < 0 {
        return ptr::null_mut();
    }

    let runtime = wifi_runtime();
    runtime
        .scan_results
        .get(num as usize)
        .map_or(ptr::null_mut(), box_station)
}

fn wifi_station_get_ssid_inner(station: wifi_station_handle) -> *const c_char {
    unsafe { station_ref(station) }
        .map(|station| station.ssid.as_ptr())
        .unwrap_or(ptr::null())
}

fn wifi_station_get_bssid_inner(station: wifi_station_handle) -> *mut mac_address_t {
    unsafe { station_ref(station) }
        .map(|station| ptr::from_ref(&station.bssid).cast_mut())
        .unwrap_or(ptr::null_mut())
}

fn wifi_station_get_primary_channel_inner(station: wifi_station_handle) -> c_int {
    unsafe { station_ref(station) }
        .map(|station| station.primary)
        .unwrap_or_default()
}

fn wifi_station_get_secondary_channel_inner(station: wifi_station_handle) -> c_int {
    unsafe { station_ref(station) }
        .map(|station| station.secondary)
        .unwrap_or_default()
}

fn wifi_station_get_rssi_inner(station: wifi_station_handle) -> c_int {
    unsafe { station_ref(station) }
        .map(|station| station.rssi as c_int)
        .unwrap_or_default()
}

fn wifi_station_get_mode_inner(station: wifi_station_handle) -> wifi_auth_mode_t {
    unsafe { station_ref(station) }
        .map(|station| station.authmode)
        .unwrap_or(wifi_auth_mode_t::WIFI_AUTH_NONE)
}

fn wifi_station_wps_inner(station: wifi_station_handle) -> bool {
    unsafe { station_ref(station) }
        .map(|station| station.wps)
        .unwrap_or(false)
}

fn wifi_set_connection_parameters_inner(ssid: *const c_char, password: *const c_char) -> bool {
    let Some(ssid) = read_truncated_string(ssid, MAX_CONFIG_SSID_BYTES) else {
        return false;
    };
    let Some(password) = read_truncated_string(password, MAX_CONFIG_PASSWORD_BYTES) else {
        return false;
    };

    let mut runtime = wifi_runtime();
    let mut updated_state = runtime.persisted.clone();
    updated_state.credentials.ssid = ssid;
    updated_state.credentials.password = password;

    match persist_runtime(&updated_state) {
        Ok(()) => {
            runtime.persisted = updated_state;
            true
        }
        Err(PersistStateError::StoreOpen(_)) => true,
        Err(PersistStateError::Write(_)) => false,
    }
}

fn get_mac_address_inner() -> *const c_char {
    let mut runtime = wifi_runtime();
    if runtime.cached_mac_address.is_none() {
        reload_persisted_state(&mut runtime);
        let mac_address = runtime
            .persisted
            .mac_address
            .as_deref()
            .and_then(normalize_mac_address)
            .or_else(discover_host_mac_address)
            .unwrap_or_else(|| String::from(ZERO_MAC_ADDRESS));

        runtime.cached_mac_address = Some(
            CString::new(mac_address)
                .unwrap_or_else(|_| CString::new(ZERO_MAC_ADDRESS).expect("valid MAC fallback")),
        );
    }

    runtime
        .cached_mac_address
        .as_ref()
        .map(|mac_address| mac_address.as_ptr())
        .unwrap_or(ptr::null())
}

#[unsafe(no_mangle)]
pub extern "C" fn wifi_get_status() -> wifi_status_t {
    wifi_status_t::WIFI_ENABLED
}

#[unsafe(no_mangle)]
pub extern "C" fn wifi_get_connection_status() -> wifi_connection_status_t {
    wifi_get_connection_status_inner()
}

#[unsafe(no_mangle)]
pub extern "C" fn wifi_get_connection_station() -> wifi_station_handle {
    wifi_get_connection_station_inner()
}

#[unsafe(no_mangle)]
pub extern "C" fn wifi_connect() -> wifi_connection_status_t {
    wifi_connect_inner()
}

#[unsafe(no_mangle)]
pub extern "C" fn wifi_disconnect() -> wifi_connection_status_t {
    wifi_disconnect_inner()
}

#[unsafe(no_mangle)]
pub extern "C" fn wifi_scan_free_station(station: wifi_station_handle) {
    wifi_scan_free_station_inner(station)
}

#[unsafe(no_mangle)]
pub extern "C" fn wifi_scan_get_num_results() -> c_int {
    wifi_scan_get_num_results_inner()
}

#[unsafe(no_mangle)]
pub extern "C" fn wifi_scan_get_result(num: c_int) -> wifi_station_handle {
    wifi_scan_get_result_inner(num)
}

#[unsafe(no_mangle)]
pub extern "C" fn wifi_station_get_ssid(station: wifi_station_handle) -> *const c_char {
    wifi_station_get_ssid_inner(station)
}

#[unsafe(no_mangle)]
pub extern "C" fn wifi_station_get_bssid(station: wifi_station_handle) -> *mut mac_address_t {
    wifi_station_get_bssid_inner(station)
}

#[unsafe(no_mangle)]
pub extern "C" fn wifi_station_get_primary_channel(station: wifi_station_handle) -> c_int {
    wifi_station_get_primary_channel_inner(station)
}

#[unsafe(no_mangle)]
pub extern "C" fn wifi_station_get_secondary_channel(station: wifi_station_handle) -> c_int {
    wifi_station_get_secondary_channel_inner(station)
}

#[unsafe(no_mangle)]
pub extern "C" fn wifi_station_get_rssi(station: wifi_station_handle) -> c_int {
    wifi_station_get_rssi_inner(station)
}

#[unsafe(no_mangle)]
pub extern "C" fn wifi_station_get_mode(station: wifi_station_handle) -> wifi_auth_mode_t {
    wifi_station_get_mode_inner(station)
}

#[unsafe(no_mangle)]
pub extern "C" fn wifi_station_wps(station: wifi_station_handle) -> bool {
    wifi_station_wps_inner(station)
}

#[unsafe(no_mangle)]
pub extern "C" fn wifi_set_connection_parameters(
    ssid: *const c_char,
    password: *const c_char,
) -> bool {
    wifi_set_connection_parameters_inner(ssid, password)
}

#[unsafe(no_mangle)]
pub extern "C" fn get_mac_address() -> *const c_char {
    get_mac_address_inner()
}

#[cfg(test)]
fn reset_wifi_runtime_for_tests() {
    *WIFI_RUNTIME.lock() = WifiRuntime::default();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        ffi::{CStr, CString},
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    struct TestWifiDirectory {
        root: PathBuf,
        _lock: spin::MutexGuard<'static, ()>,
    }

    impl TestWifiDirectory {
        fn new() -> Self {
            let lock = TEST_LOCK.lock();
            let suffix = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let root = std::env::temp_dir()
                .join(alloc::format!("why2025-wifi-test-{}-{suffix}", std::process::id()));

            let _ = fs::remove_dir_all(&root);
            *TEST_BASE_DIRECTORY.lock() = Some(root.to_string_lossy().into_owned());
            reset_wifi_runtime_for_tests();

            Self { root, _lock: lock }
        }

        fn set_blocked_base_directory(&self) -> PathBuf {
            let blocked_base = self.root.join("blocked-base");
            fs::create_dir_all(&self.root).unwrap();
            fs::write(&blocked_base, b"not a directory").unwrap();
            *TEST_BASE_DIRECTORY.lock() = Some(blocked_base.to_string_lossy().into_owned());
            reset_wifi_runtime_for_tests();
            blocked_base
        }

        fn state_file(&self) -> PathBuf {
            self.root.join(WIFI_DIRECTORY_NAME).join(STATE_FILE_NAME)
        }

        fn write_state(&self, state: &PersistedWifiState) {
            fs::create_dir_all(self.root.join(WIFI_DIRECTORY_NAME)).unwrap();
            fs::write(
                self.state_file(),
                serde_json::to_vec_pretty(state).expect("serializable Wi-Fi state"),
            )
            .unwrap();
            reset_wifi_runtime_for_tests();
        }

        fn overwrite_state_without_reset(&self, state: &PersistedWifiState) {
            fs::create_dir_all(self.root.join(WIFI_DIRECTORY_NAME)).unwrap();
            fs::write(
                self.state_file(),
                serde_json::to_vec_pretty(state).expect("serializable Wi-Fi state"),
            )
            .unwrap();
        }

        fn read_state(&self) -> PersistedWifiState {
            serde_json::from_slice(&fs::read(self.state_file()).unwrap()).unwrap()
        }
    }

    impl Drop for TestWifiDirectory {
        fn drop(&mut self) {
            *TEST_BASE_DIRECTORY.lock() = None;
            reset_wifi_runtime_for_tests();
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn fixture(ssid: &str, authmode: wifi_auth_mode_t, password: &str) -> PersistedWifiFixture {
        PersistedWifiFixture {
            ssid: String::from(ssid),
            bssid: [0x10, 0x20, 0x30, 0x40, 0x50, 0x60],
            primary: 1,
            secondary: 0,
            rssi: -50,
            authmode: authmode as u32,
            wps: false,
            password: String::from(password),
        }
    }

    #[test]
    fn wifi_state_alloc_serde_round_trip() {
        let state = PersistedWifiState {
            credentials: PersistedCredentials {
                ssid: String::from(DEFAULT_SSID),
                password: String::from(DEFAULT_PASSWORD),
            },
            fixtures: alloc::vec![fixture(DEFAULT_SSID, wifi_auth_mode_t::WIFI_AUTH_OPEN, "")],
            mac_address: Some(String::from("AA:BB:CC:DD:EE:FF")),
        };

        let encoded = serde_json::to_vec(&state).expect("serialize wifi state");
        let decoded: PersistedWifiState =
            serde_json::from_slice(&encoded).expect("deserialize wifi state");

        assert_eq!(decoded, state);
    }

    #[test]
    fn credentials_persist_and_default_network_connects() {
        let directory = TestWifiDirectory::new();
        let ssid = CString::new(DEFAULT_SSID).unwrap();
        let password = CString::new(DEFAULT_PASSWORD).unwrap();

        assert!(wifi_set_connection_parameters(ssid.as_ptr(), password.as_ptr()));

        let state = directory.read_state();
        assert_eq!(state.credentials.ssid, DEFAULT_SSID);
        assert_eq!(state.credentials.password, DEFAULT_PASSWORD);

        assert_eq!(wifi_connect(), wifi_connection_status_t::WIFI_CONNECTED);
        assert_eq!(
            wifi_get_connection_status(),
            wifi_connection_status_t::WIFI_CONNECTED
        );

        let station = wifi_get_connection_station();
        assert!(!station.is_null());
        let connected_ssid = unsafe { CStr::from_ptr(wifi_station_get_ssid(station)) };
        assert_eq!(connected_ssid.to_str().unwrap(), DEFAULT_SSID);
        assert_eq!(
            wifi_station_get_mode(station),
            wifi_auth_mode_t::WIFI_AUTH_OPEN
        );
        wifi_scan_free_station(station);
    }

    #[test]
    fn wrong_password_surfaces_wrong_credentials_and_disconnect_preserves_error() {
        let _directory = TestWifiDirectory::new();
        let ssid = CString::new("WHY2025-secure").unwrap();
        let password = CString::new("not-the-password").unwrap();

        assert!(wifi_set_connection_parameters(ssid.as_ptr(), password.as_ptr()));
        assert_eq!(
            wifi_connect(),
            wifi_connection_status_t::WIFI_ERROR_WRONG_CREDENTIALS
        );
        assert!(wifi_get_connection_station().is_null());
        assert_eq!(
            wifi_disconnect(),
            wifi_connection_status_t::WIFI_ERROR_WRONG_CREDENTIALS
        );
    }

    #[test]
    fn set_connection_parameters_keeps_upstream_true_result_when_store_open_fails() {
        let directory = TestWifiDirectory::new();
        let blocked_base = directory.set_blocked_base_directory();
        let ssid = CString::new("WHY2025-secure").unwrap();
        let password = CString::new("not-the-password").unwrap();

        assert!(wifi_set_connection_parameters(ssid.as_ptr(), password.as_ptr()));
        assert!(!blocked_base.join(WIFI_DIRECTORY_NAME).join(STATE_FILE_NAME).exists());

        assert_eq!(wifi_connect(), wifi_connection_status_t::WIFI_CONNECTED);

        let station = wifi_get_connection_station();
        assert!(!station.is_null());
        let connected_ssid = unsafe { CStr::from_ptr(wifi_station_get_ssid(station)) };
        assert_eq!(connected_ssid.to_str().unwrap(), DEFAULT_SSID);
        wifi_scan_free_station(station);
    }

    #[test]
    fn scan_cache_throttles_and_truncates_visible_results() {
        let directory = TestWifiDirectory::new();
        let state = PersistedWifiState {
            credentials: PersistedCredentials {
                ssid: String::from(DEFAULT_SSID),
                password: String::from(DEFAULT_PASSWORD),
            },
            fixtures: (0..25)
                .map(|index| PersistedWifiFixture {
                    ssid: alloc::format!("scan-{index}"),
                    bssid: [0, 1, 2, 3, 4, index as u8],
                    primary: index,
                    secondary: 0,
                    rssi: -30,
                    authmode: wifi_auth_mode_t::WIFI_AUTH_OPEN as u32,
                    wps: false,
                    password: String::new(),
                })
                .collect(),
            mac_address: None,
        };
        directory.write_state(&state);

        assert_eq!(wifi_scan_get_num_results(), 20);

        let emptied_state = PersistedWifiState {
            fixtures: Vec::new(),
            ..state.clone()
        };
        directory.overwrite_state_without_reset(&emptied_state);
        assert_eq!(wifi_scan_get_num_results(), 20);

        {
            let mut runtime = wifi_runtime();
            let now = monotonic_now().unwrap_or(Duration::from_secs(0));
            runtime.last_scan_at = Some(now.saturating_sub(MIN_SCAN_INTERVAL + Duration::from_secs(1)));
        }

        assert_eq!(wifi_scan_get_num_results(), 0);
    }

    #[test]
    fn mac_address_is_cached_after_first_lookup() {
        let directory = TestWifiDirectory::new();
        let state = PersistedWifiState {
            credentials: PersistedCredentials {
                ssid: String::from(DEFAULT_SSID),
                password: String::from(DEFAULT_PASSWORD),
            },
            fixtures: alloc::vec![fixture(DEFAULT_SSID, wifi_auth_mode_t::WIFI_AUTH_OPEN, "")],
            mac_address: Some(String::from("AA:BB:CC:DD:EE:FF")),
        };
        directory.write_state(&state);

        let first = get_mac_address();
        let first_value = String::from(unsafe { CStr::from_ptr(first) }.to_str().unwrap());
        assert_eq!(first_value, "AA:BB:CC:DD:EE:FF");

        directory.overwrite_state_without_reset(&PersistedWifiState {
            mac_address: Some(String::from("11:22:33:44:55:66")),
            ..state
        });

        let second = get_mac_address();
        let second_value = String::from(unsafe { CStr::from_ptr(second) }.to_str().unwrap());
        assert_eq!(first, second);
        assert_eq!(second_value, "AA:BB:CC:DD:EE:FF");
    }
}