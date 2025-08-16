use crate::types::*;

#[unsafe(no_mangle)]
pub extern "C" fn wifi_get_status() -> wifi_status_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn wifi_get_connection_status() -> wifi_connection_status_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn wifi_get_connection_station() -> wifi_station_handle {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn wifi_connect() -> wifi_connection_status_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn wifi_disconnect() -> wifi_connection_status_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn wifi_scan_free_station(station: wifi_station_handle) {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn wifi_scan_get_num_results() -> ::core::ffi::c_int {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn wifi_scan_get_result(num: ::core::ffi::c_int) -> wifi_station_handle {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn wifi_station_get_ssid(
    station: wifi_station_handle,
) -> *const ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn wifi_station_get_bssid(station: wifi_station_handle) -> *mut mac_address_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn wifi_station_get_primary_channel(
    station: wifi_station_handle,
) -> ::core::ffi::c_int {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn wifi_station_get_secondary_channel(
    station: wifi_station_handle,
) -> ::core::ffi::c_int {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn wifi_station_get_rssi(station: wifi_station_handle) -> ::core::ffi::c_int {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn wifi_station_get_mode(station: wifi_station_handle) -> wifi_auth_mode_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn wifi_station_wps(station: wifi_station_handle) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn get_mac_address() -> *const ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}
