use crate::runtime;
use crate::types::*;
use core::ffi::{c_char, c_int, c_long};

macro_rules! aborting_export {
    ($family:literal, fn $name:ident($($arg:ident : $arg_ty:ty),* $(,)?) -> $ret:ty) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name($($arg : $arg_ty),*) -> $ret {
            let _ = ($($arg),*);
            runtime::abort_unimplemented_symbol(stringify!($name), $family)
        }
    };
    ($family:literal, fn $name:ident($($arg:ident : $arg_ty:ty),* $(,)?)) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name($($arg : $arg_ty),*) {
            let _ = ($($arg),*);
            runtime::abort_unimplemented_symbol(stringify!($name), $family)
        }
    };
}

aborting_export!("graphics", fn window_create(title: *const c_char, size: window_size_t, flags: window_flag_t) -> window_handle_t);
aborting_export!("graphics", fn window_framebuffer_create(window: window_handle_t, size: window_size_t, pixel_format: pixel_format_t) -> *mut framebuffer_t);
aborting_export!("graphics", fn window_destroy(window: window_handle_t));
aborting_export!("graphics", fn window_title_get(window: window_handle_t) -> *const c_char);
aborting_export!("graphics", fn window_title_set(window: window_handle_t, title: *const c_char));
aborting_export!("graphics", fn window_position_get(window: window_handle_t) -> window_coords_t);
aborting_export!("graphics", fn window_position_set(window: window_handle_t, coords: window_coords_t) -> window_coords_t);
aborting_export!("graphics", fn window_size_get(window: window_handle_t) -> window_size_t);
aborting_export!("graphics", fn window_size_set(window: window_handle_t, size: window_size_t) -> window_size_t);
aborting_export!("graphics", fn window_flags_get(window: window_handle_t) -> window_flag_t);
aborting_export!("graphics", fn window_flags_set(window: window_handle_t, flags: window_flag_t) -> window_flag_t);
aborting_export!("graphics", fn window_framebuffer_size_get(window: window_handle_t) -> window_size_t);
aborting_export!("graphics", fn window_framebuffer_size_set(window: window_handle_t, size: window_size_t) -> window_size_t);
aborting_export!("graphics", fn window_framebuffer_format_get(window: window_handle_t) -> pixel_format_t);
aborting_export!("graphics", fn window_framebuffer_get(window: window_handle_t) -> *mut framebuffer_t);
aborting_export!("graphics", fn window_present(window: window_handle_t, block: bool, rects: *mut window_rect_t, num_rects: c_int));
aborting_export!("graphics", fn window_event_poll(window: window_handle_t, block: bool, timeout_msec: u32) -> event_t);
aborting_export!("graphics", fn get_screen_info(width: *mut c_int, height: *mut c_int, format: *mut pixel_format_t, refresh_rate: *mut f32));

aborting_export!("wifi", fn wifi_get_status() -> wifi_status_t);
aborting_export!("wifi", fn wifi_get_connection_status() -> wifi_connection_status_t);
aborting_export!("wifi", fn wifi_get_connection_station() -> wifi_station_handle);
aborting_export!("wifi", fn wifi_connect() -> wifi_connection_status_t);
aborting_export!("wifi", fn wifi_disconnect() -> wifi_connection_status_t);
aborting_export!("wifi", fn wifi_scan_free_station(station: wifi_station_handle));
aborting_export!("wifi", fn wifi_scan_get_num_results() -> c_int);
aborting_export!("wifi", fn wifi_scan_get_result(num: c_int) -> wifi_station_handle);
aborting_export!("wifi", fn wifi_station_get_ssid(station: wifi_station_handle) -> *const c_char);
aborting_export!("wifi", fn wifi_station_get_bssid(station: wifi_station_handle) -> *mut mac_address_t);
aborting_export!("wifi", fn wifi_station_get_primary_channel(station: wifi_station_handle) -> c_int);
aborting_export!("wifi", fn wifi_station_get_secondary_channel(station: wifi_station_handle) -> c_int);
aborting_export!("wifi", fn wifi_station_get_rssi(station: wifi_station_handle) -> c_int);
aborting_export!("wifi", fn wifi_station_get_mode(station: wifi_station_handle) -> wifi_auth_mode_t);
aborting_export!("wifi", fn wifi_station_wps(station: wifi_station_handle) -> bool);
aborting_export!("wifi", fn wifi_set_connection_parameters(ssid: *const c_char, password: *const c_char) -> bool);

aborting_export!("networking", fn curl_easy_init() -> *mut CURL);
aborting_export!("networking", fn curl_easy_perform(curl: *mut CURL) -> CURLcode);
aborting_export!("networking", fn curl_easy_cleanup(curl: *mut CURL));
aborting_export!("networking", fn curl_easy_strerror(error: CURLcode) -> *const c_char);
aborting_export!("networking", fn curl_slist_append(list: *mut curl_slist, string: *const c_char) -> *mut curl_slist);
aborting_export!("networking", fn curl_slist_free_all(list: *mut curl_slist));
aborting_export!("networking", fn curl_global_init(flags: c_long) -> CURLcode);
aborting_export!("networking", fn curl_global_cleanup());

#[unsafe(no_mangle)]
pub unsafe extern "C" fn curl_easy_setopt(
    curl: *mut CURL,
    option: CURLoption,
    mut _args: ...
) -> CURLcode {
    let _ = (curl, option);
    runtime::abort_unimplemented_symbol("curl_easy_setopt", "networking")
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn curl_easy_getinfo(
    curl: *mut CURL,
    info: curl_easy_info_t,
    mut _args: ...
) -> CURLcode {
    let _ = (curl, info);
    runtime::abort_unimplemented_symbol("curl_easy_getinfo", "networking")
}

aborting_export!("networking", fn inet_ntoa(__in: in_addr) -> *mut c_char);
aborting_export!("networking", fn inet_aton(__cp: *const c_char, __inp: *mut in_addr) -> c_int);
aborting_export!("networking", fn accept(s: c_int, addr: *mut sockaddr, addrlen: *mut socklen_t) -> c_int);
aborting_export!("networking", fn bind(s: c_int, name: *const sockaddr, namelen: socklen_t) -> c_int);
aborting_export!("networking", fn connect(s: c_int, name: *const sockaddr, namelen: socklen_t) -> c_int);
aborting_export!("networking", fn listen(s: c_int, backlog: c_int) -> c_int);
aborting_export!("networking", fn socket(domain: c_int, type_: c_int, protocol: c_int) -> c_int);
aborting_export!("networking", fn freeaddrinfo(ai: *mut addrinfo));
aborting_export!("networking", fn getaddrinfo(nodename: *const c_char, servname: *const c_char, hints: *const addrinfo, res: *mut *mut addrinfo) -> c_int);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deferred_symbols_have_addresses() {
        assert_ne!(window_create as *const (), core::ptr::null());
        assert_ne!(wifi_get_status as *const (), core::ptr::null());
        assert_ne!(curl_easy_init as *const (), core::ptr::null());
        assert_ne!(socket as *const (), core::ptr::null());
    }
}
