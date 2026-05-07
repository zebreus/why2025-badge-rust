use crate::{emulated::badgevms::fs::paths::base_directory, types::*};
use core::ffi::{CStr, c_char, c_int};
use serde::{Deserialize, Serialize};
use std::{
    ffi::CString,
    fs, io,
    path::{Path, PathBuf},
    ptr,
    sync::{LazyLock, Mutex, MutexGuard},
    time::{Duration, Instant},
};

#[cfg(test)]
use crate::emulated::badgevms::fs::paths::set_base_directory_for_tests;
#[cfg(test)]
use std::time::{SystemTime, UNIX_EPOCH};

const WIFI_DIRECTORY_NAME: &str = "wifi";
const STATE_FILE_NAME: &str = "state.json";
const ZERO_MAC_ADDRESS: &str = "00:00:00:00:00:00";
const DEFAULT_SSID: &str = "WHY2025-open";
const DEFAULT_PASSWORD: &str = "";
const MAX_SCAN_RESULTS: usize = 20;
const MAX_CONFIG_SSID_BYTES: usize = 31;
const MAX_CONFIG_PASSWORD_BYTES: usize = 63;
const MAX_STATION_SSID_BYTES: usize = 32;
const MIN_SCAN_INTERVAL: Duration = Duration::from_secs(10);
const MIN_SCAN_INTERVAL_EMPTY: Duration = Duration::from_secs(1);

static WIFI_RUNTIME: LazyLock<Mutex<WifiRuntime>> =
    LazyLock::new(|| Mutex::new(WifiRuntime::default()));

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct PersistedCredentials {
    ssid: String,
    password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct PersistedWifiFixture {
    ssid: String,
    bssid: [u8; 6],
    primary: i32,
    secondary: i32,
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
    last_scan_at: Option<Instant>,
    persisted: PersistedWifiState,
    cached_mac_address: Option<CString>,
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

fn wifi_runtime() -> MutexGuard<'static, WifiRuntime> {
    let mut runtime = WIFI_RUNTIME
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
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

fn wifi_root_directory() -> PathBuf {
    base_directory().join(WIFI_DIRECTORY_NAME)
}

fn state_file_path(root: &Path) -> PathBuf {
    root.join(STATE_FILE_NAME)
}

fn default_persisted_state() -> PersistedWifiState {
    PersistedWifiState {
        credentials: PersistedCredentials {
            ssid: DEFAULT_SSID.to_owned(),
            password: DEFAULT_PASSWORD.to_owned(),
        },
        fixtures: vec![
            PersistedWifiFixture {
                ssid: DEFAULT_SSID.to_owned(),
                bssid: [0x57, 0x48, 0x59, 0x25, 0x00, 0x01],
                primary: 1,
                secondary: 0,
                rssi: -42,
                authmode: wifi_auth_mode_t::WIFI_AUTH_OPEN as u32,
                wps: false,
                password: String::new(),
            },
            PersistedWifiFixture {
                ssid: "WHY2025-secure".to_owned(),
                bssid: [0x57, 0x48, 0x59, 0x25, 0x00, 0x02],
                primary: 6,
                secondary: 0,
                rssi: -55,
                authmode: wifi_auth_mode_t::WIFI_AUTH_WPA2_PSK as u32,
                wps: true,
                password: "why2025".to_owned(),
            },
            PersistedWifiFixture {
                ssid: "WHY2025-mesh".to_owned(),
                bssid: [0x57, 0x48, 0x59, 0x25, 0x00, 0x03],
                primary: 11,
                secondary: 0,
                rssi: -61,
                authmode: wifi_auth_mode_t::WIFI_AUTH_WPA2_WPA3_PSK as u32,
                wps: false,
                password: "mesh-password".to_owned(),
            },
        ],
        mac_address: None,
    }
}

fn load_persisted_state(root: &Path) -> PersistedWifiState {
    let default_state = default_persisted_state();
    let state_file = state_file_path(root);

    let Ok(content) = fs::read(&state_file) else {
        let _ = save_persisted_state(root, &default_state);
        return default_state;
    };

    serde_json::from_slice(&content).unwrap_or(default_state)
}

fn save_persisted_state(root: &Path, state: &PersistedWifiState) -> io::Result<()> {
    fs::create_dir_all(root)?;

    let temporary_file = root.join("state.tmp");
    let serialized = serde_json::to_vec_pretty(state)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    fs::write(&temporary_file, serialized)?;
    fs::rename(temporary_file, state_file_path(root))
}

fn persist_runtime(runtime: &WifiRuntime) -> io::Result<()> {
    save_persisted_state(&wifi_root_directory(), &runtime.persisted)
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

fn refresh_scan_results(runtime: &mut WifiRuntime) {
    let now = Instant::now();
    let minimum_interval = if runtime.scan_results.is_empty() {
        MIN_SCAN_INTERVAL_EMPTY
    } else {
        MIN_SCAN_INTERVAL
    };

    if let Some(last_scan_at) = runtime.last_scan_at
        && now.saturating_duration_since(last_scan_at) < minimum_interval
    {
        return;
    }

    reload_persisted_state(runtime);
    runtime.last_scan_at = Some(now);
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
    format!(
        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]
    )
}

fn normalize_mac_address(value: &str) -> Option<String> {
    let trimmed = value.trim();
    let segments: Vec<_> = trimmed.split(':').collect();
    if segments.len() != 6 {
        return None;
    }

    let mut bytes = [0u8; 6];
    for (index, segment) in segments.iter().enumerate() {
        if segment.len() != 2 {
            return None;
        }
        bytes[index] = u8::from_str_radix(segment, 16).ok()?;
    }

    Some(format_mac_address(bytes))
}

fn discover_host_mac_address() -> Option<String> {
    let entries = fs::read_dir("/sys/class/net").ok()?;

    for entry in entries.flatten() {
        if entry.file_name() == "lo" {
            continue;
        }

        let address = fs::read_to_string(entry.path().join("address")).ok()?;
        if let Some(mac_address) = normalize_mac_address(&address)
            && mac_address != ZERO_MAC_ADDRESS
        {
            return Some(mac_address);
        }
    }

    None
}

pub(crate) fn wifi_get_connection_status_inner() -> wifi_connection_status_t {
    wifi_runtime().connection_status
}

pub(crate) fn wifi_get_connection_station_inner() -> wifi_station_handle {
    let runtime = wifi_runtime();
    if runtime.connection_status != wifi_connection_status_t::WIFI_CONNECTED {
        return ptr::null_mut();
    }

    runtime
        .current_station
        .as_ref()
        .map_or(ptr::null_mut(), box_station)
}

pub(crate) fn wifi_connect_inner() -> wifi_connection_status_t {
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
        .find(|fixture| fixture.ssid == credentials.ssid);

    let outcome = match fixture {
        Some(fixture)
            if matches!(
                authmode_from_raw(fixture.authmode),
                wifi_auth_mode_t::WIFI_AUTH_OPEN | wifi_auth_mode_t::WIFI_AUTH_NONE
            ) || fixture.password == credentials.password =>
        {
            runtime.current_station = Some(HostWifiStation::from_fixture(fixture));
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

pub(crate) fn wifi_disconnect_inner() -> wifi_connection_status_t {
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

pub(crate) fn wifi_scan_free_station_inner(station: wifi_station_handle) {
    if station.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(station.cast::<HostWifiStation>()));
    }
}

pub(crate) fn wifi_scan_get_num_results_inner() -> c_int {
    let mut runtime = wifi_runtime();
    refresh_scan_results(&mut runtime);
    runtime.scan_results.len() as c_int
}

pub(crate) fn wifi_scan_get_result_inner(num: c_int) -> wifi_station_handle {
    if num < 0 {
        return ptr::null_mut();
    }

    let runtime = wifi_runtime();
    runtime
        .scan_results
        .get(num as usize)
        .map_or(ptr::null_mut(), box_station)
}

pub(crate) fn wifi_station_get_ssid_inner(station: wifi_station_handle) -> *const c_char {
    unsafe { station_ref(station) }
        .map(|station| station.ssid.as_ptr())
        .unwrap_or(ptr::null())
}

pub(crate) fn wifi_station_get_bssid_inner(station: wifi_station_handle) -> *mut mac_address_t {
    unsafe { station_ref(station) }
        .map(|station| ptr::from_ref(&station.bssid).cast_mut())
        .unwrap_or(ptr::null_mut())
}

pub(crate) fn wifi_station_get_primary_channel_inner(station: wifi_station_handle) -> c_int {
    unsafe { station_ref(station) }
        .map(|station| station.primary)
        .unwrap_or_default()
}

pub(crate) fn wifi_station_get_secondary_channel_inner(station: wifi_station_handle) -> c_int {
    unsafe { station_ref(station) }
        .map(|station| station.secondary)
        .unwrap_or_default()
}

pub(crate) fn wifi_station_get_rssi_inner(station: wifi_station_handle) -> c_int {
    unsafe { station_ref(station) }
        .map(|station| station.rssi as c_int)
        .unwrap_or_default()
}

pub(crate) fn wifi_station_get_mode_inner(station: wifi_station_handle) -> wifi_auth_mode_t {
    unsafe { station_ref(station) }
        .map(|station| station.authmode)
        .unwrap_or(wifi_auth_mode_t::WIFI_AUTH_NONE)
}

pub(crate) fn wifi_station_wps_inner(station: wifi_station_handle) -> bool {
    unsafe { station_ref(station) }
        .map(|station| station.wps)
        .unwrap_or(false)
}

pub(crate) fn wifi_set_connection_parameters_inner(
    ssid: *const c_char,
    password: *const c_char,
) -> bool {
    let Some(ssid) = read_truncated_string(ssid, MAX_CONFIG_SSID_BYTES) else {
        return false;
    };
    let Some(password) = read_truncated_string(password, MAX_CONFIG_PASSWORD_BYTES) else {
        return false;
    };

    let mut runtime = wifi_runtime();
    runtime.persisted.credentials.ssid = ssid;
    runtime.persisted.credentials.password = password;
    persist_runtime(&runtime).is_ok()
}

pub(crate) fn get_mac_address_inner() -> *const c_char {
    let mut runtime = wifi_runtime();
    if runtime.cached_mac_address.is_none() {
        reload_persisted_state(&mut runtime);
        let mac_address = runtime
            .persisted
            .mac_address
            .as_deref()
            .and_then(normalize_mac_address)
            .or_else(discover_host_mac_address)
            .unwrap_or_else(|| ZERO_MAC_ADDRESS.to_owned());

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

#[cfg(test)]
fn reset_wifi_runtime_for_tests() {
    let mut runtime = WIFI_RUNTIME
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *runtime = WifiRuntime::default();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emulated::badgevms::wifi::{
        get_mac_address, wifi_connect, wifi_disconnect, wifi_get_connection_station,
        wifi_get_connection_status, wifi_scan_free_station, wifi_scan_get_num_results,
        wifi_set_connection_parameters, wifi_station_get_mode, wifi_station_get_ssid,
    };
    use std::ffi::CStr;

    struct TestWifiDirectory {
        root: PathBuf,
        _guard: crate::emulated::badgevms::fs::paths::TestBaseDirectoryGuard,
    }

    impl TestWifiDirectory {
        fn new() -> Self {
            let suffix = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let root = std::env::temp_dir()
                .join(format!("why2025-wifi-test-{}-{suffix}", std::process::id()));
            let _guard = set_base_directory_for_tests(root.clone());
            let _ = fs::remove_dir_all(&root);
            reset_wifi_runtime_for_tests();

            Self { root, _guard }
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
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn fixture(ssid: &str, authmode: wifi_auth_mode_t, password: &str) -> PersistedWifiFixture {
        PersistedWifiFixture {
            ssid: ssid.to_owned(),
            bssid: [0x10, 0x20, 0x30, 0x40, 0x50, 0x60],
            primary: 1,
            secondary: 0,
            rssi: -50,
            authmode: authmode as u32,
            wps: false,
            password: password.to_owned(),
        }
    }

    #[test]
    fn credentials_persist_and_default_network_connects() {
        let directory = TestWifiDirectory::new();
        let ssid = CString::new(DEFAULT_SSID).unwrap();
        let password = CString::new(DEFAULT_PASSWORD).unwrap();

        assert!(wifi_set_connection_parameters(
            ssid.as_ptr(),
            password.as_ptr()
        ));

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

        assert!(wifi_set_connection_parameters(
            ssid.as_ptr(),
            password.as_ptr()
        ));
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
    fn scan_cache_throttles_and_truncates_visible_results() {
        let directory = TestWifiDirectory::new();
        let state = PersistedWifiState {
            credentials: PersistedCredentials {
                ssid: DEFAULT_SSID.to_owned(),
                password: DEFAULT_PASSWORD.to_owned(),
            },
            fixtures: (0..25)
                .map(|index| PersistedWifiFixture {
                    ssid: format!("scan-{index}"),
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
            runtime.last_scan_at =
                Some(Instant::now() - MIN_SCAN_INTERVAL - Duration::from_secs(1));
        }

        assert_eq!(wifi_scan_get_num_results(), 0);
    }

    #[test]
    fn mac_address_is_cached_after_first_lookup() {
        let directory = TestWifiDirectory::new();
        let state = PersistedWifiState {
            credentials: PersistedCredentials {
                ssid: DEFAULT_SSID.to_owned(),
                password: DEFAULT_PASSWORD.to_owned(),
            },
            fixtures: vec![fixture(DEFAULT_SSID, wifi_auth_mode_t::WIFI_AUTH_OPEN, "")],
            mac_address: Some("AA:BB:CC:DD:EE:FF".to_owned()),
        };
        directory.write_state(&state);

        let first = get_mac_address();
        let first_value = unsafe { CStr::from_ptr(first) }
            .to_str()
            .unwrap()
            .to_owned();
        assert_eq!(first_value, "AA:BB:CC:DD:EE:FF");

        directory.overwrite_state_without_reset(&PersistedWifiState {
            mac_address: Some("11:22:33:44:55:66".to_owned()),
            ..state
        });

        let second = get_mac_address();
        let second_value = unsafe { CStr::from_ptr(second) }
            .to_str()
            .unwrap()
            .to_owned();
        assert_eq!(first, second);
        assert_eq!(second_value, "AA:BB:CC:DD:EE:FF");
    }
}
