extern crate curl_sys;
extern crate libc;

use crate::*;

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn atoff(__nptr: *const ::core::ffi::c_char) -> f32 {
    unsafe {
        let result = atof(__nptr);
        return result as f32;
    }
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn fls(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int {
    for i in (0..32).rev() {
        if (arg1 & (1 << i)) != 0 {
            return i + 1;
        }
    }
    0
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn flsl(arg1: ::core::ffi::c_long) -> ::core::ffi::c_int {
    for i in (0..64).rev() {
        if (arg1 & (1 << i)) != 0 {
            return i + 1;
        }
    }
    0
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn flsll(arg1: ::core::ffi::c_longlong) -> ::core::ffi::c_int {
    for i in (0..64).rev() {
        if (arg1 & (1 << i)) != 0 {
            return i + 1;
        }
    }
    0
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn fpgetround() -> fp_rnd {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn fpsetround(arg1: fp_rnd) -> fp_rnd {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn fpgetmask() -> fp_except {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn fpsetmask(arg1: fp_except) -> fp_except {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn fpgetsticky() -> fp_except {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn fpsetsticky(arg1: fp_except) -> fp_except {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn isascii_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    unsafe { isascii(c) }
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn itoa(
    arg1: ::core::ffi::c_int,
    arg2: *mut ::core::ffi::c_char,
    arg3: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn sig2str(
    signum: ::core::ffi::c_int,
    str_: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unimplemented!("Implement this yourself if you need it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn str2sig(
    str_: *const ::core::ffi::c_char,
    pnum: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unimplemented!("Implement this yourself if you need it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn strlwr(arg1: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn strnstr(
    arg1: *const ::core::ffi::c_char,
    arg2: *const ::core::ffi::c_char,
    arg3: size_t,
) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn strtoimax_l(
    arg1: *const ::core::ffi::c_char,
    _restrict: *mut *mut ::core::ffi::c_char,
    arg2: ::core::ffi::c_int,
    arg3: locale_t,
) -> intmax_t {
    unimplemented!("Implement this yourself if you need it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn strtoumax_l(
    arg1: *const ::core::ffi::c_char,
    _restrict: *mut *mut ::core::ffi::c_char,
    arg2: ::core::ffi::c_int,
    arg3: locale_t,
) -> uintmax_t {
    unimplemented!("Implement this yourself if you need it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn strupr(arg1: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn timingsafe_bcmp(
    arg1: *const ::core::ffi::c_void,
    arg2: *const ::core::ffi::c_void,
    arg3: size_t,
) -> ::core::ffi::c_int {
    unimplemented!("Implement this yourself if you need it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn timingsafe_memcmp(
    arg1: *const ::core::ffi::c_void,
    arg2: *const ::core::ffi::c_void,
    arg3: size_t,
) -> ::core::ffi::c_int {
    unimplemented!("Implement this yourself if you need it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn toascii_l(c: ::core::ffi::c_int, l: locale_t) -> ::core::ffi::c_int {
    unimplemented!("Implement this yourself if you need it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn utoa(
    arg1: ::core::ffi::c_uint,
    arg2: *mut ::core::ffi::c_char,
    arg3: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn wcstoimax_l(
    arg1: *const _wchar_t,
    _restrict: *mut *mut _wchar_t,
    arg2: ::core::ffi::c_int,
    arg3: locale_t,
) -> intmax_t {
    unimplemented!("Implement this yourself if you need it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn wcstoumax_l(
    arg1: *const _wchar_t,
    _restrict: *mut *mut _wchar_t,
    arg2: ::core::ffi::c_int,
    arg3: locale_t,
) -> uintmax_t {
    unimplemented!("Implement this yourself if you need it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn gammaf_r(arg1: f32, arg2: *mut ::core::ffi::c_int) -> f32 {
    unimplemented!("Implement this yourself if you need it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn gamma_r(arg1: f64, arg2: *mut ::core::ffi::c_int) -> f64 {
    unimplemented!("Implement this yourself if you need it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn infinity() -> f64 {
    unimplemented!("Implement this yourself if you need it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn infinityf() -> f32 {
    unimplemented!("Implement this yourself if you need it");
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn exp10(arg1: f64) -> f64 {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn pow10(arg1: f64) -> f64 {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn exp10f(arg1: f32) -> f32 {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn pow10f(arg1: f32) -> f32 {
    unimplemented!("Implement this yourself if you need it");
}

unsafe extern "C" {
    #[link_name = "dprintf"]
    pub unsafe fn diprintf(
        a: ::core::ffi::c_int,
        b: *const ::core::ffi::c_char,
        ...
    ) -> ::core::ffi::c_int;
}

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub unsafe extern "C" fn asnprintf(
    str_: *mut ::core::ffi::c_char,
    lenp: *mut size_t,
    fmt: *const ::core::ffi::c_char,
    ...
) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn gcvtf(
    arg1: f32,
    arg2: ::core::ffi::c_int,
    arg3: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn gcvtl(
    arg1: u128,
    arg2: ::core::ffi::c_int,
    arg3: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn funopen(
    cookie: *const ::core::ffi::c_void,
    readfn: ::core::option::Option<
        unsafe extern "C" fn(
            cookie: *mut ::core::ffi::c_void,
            buf: *mut ::core::ffi::c_void,
            n: size_t,
        ) -> _ssize_t,
    >,
    writefn: ::core::option::Option<
        unsafe extern "C" fn(
            cookie: *mut ::core::ffi::c_void,
            buf: *const ::core::ffi::c_void,
            n: size_t,
        ) -> _ssize_t,
    >,
    seekfn: ::core::option::Option<
        unsafe extern "C" fn(
            cookie: *mut ::core::ffi::c_void,
            off: __off_t,
            whence: ::core::ffi::c_int,
        ) -> __off_t,
    >,
    closefn: ::core::option::Option<
        unsafe extern "C" fn(cookie: *mut ::core::ffi::c_void) -> ::core::ffi::c_int,
    >,
) -> *mut FILE {
    unimplemented!("Implement this yourself if you need it");
}

#[unsafe(no_mangle)]
pub extern "C" fn application_launch(unique_identifier: *const ::core::ffi::c_char) -> pid_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_create(
    unique_identifier: *const ::core::ffi::c_char,
    name: *const ::core::ffi::c_char,
    author: *const ::core::ffi::c_char,
    version: *const ::core::ffi::c_char,
    interpreter: *const ::core::ffi::c_char,
    source: application_source_t,
) -> *mut application_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_set_metadata(
    application: *mut application_t,
    metadata_file: *const ::core::ffi::c_char,
) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_set_binary_path(
    application: *mut application_t,
    binary_path: *const ::core::ffi::c_char,
) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_set_version(
    application: *mut application_t,
    version: *const ::core::ffi::c_char,
) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_set_author(
    application: *mut application_t,
    author: *const ::core::ffi::c_char,
) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_set_name(
    application: *mut application_t,
    name: *const ::core::ffi::c_char,
) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_set_interpreter(
    application: *mut application_t,
    interpreter: *const ::core::ffi::c_char,
) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_destroy(application: *mut application_t) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_create_file(
    application: *mut application_t,
    file_path: *const ::core::ffi::c_char,
) -> *mut FILE {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_create_file_string(
    application: *mut application_t,
    file_path: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_list(out: *mut *mut application_t) -> application_list_handle {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_list_get_next(list: application_list_handle) -> *mut application_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_list_close(list: application_list_handle) {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_get(
    unique_identifier: *const ::core::ffi::c_char,
) -> *mut application_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_free(application: *mut application_t) {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn window_create(
    title: *const ::core::ffi::c_char,
    size: window_size_t,
    flags: window_flag_t,
) -> window_handle_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn window_framebuffer_create(
    window: window_handle_t,
    size: window_size_t,
    pixel_format: pixel_format_t,
) -> *mut framebuffer_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn window_destroy(window: window_handle_t) {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn window_title_get(window: window_handle_t) -> *const ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn window_title_set(window: window_handle_t, title: *const ::core::ffi::c_char) {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn window_position_get(window: window_handle_t) -> window_coords_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn window_position_set(
    window: window_handle_t,
    coords: window_coords_t,
) -> window_coords_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn window_size_get(window: window_handle_t) -> window_size_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn window_size_set(window: window_handle_t, size: window_size_t) -> window_size_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn window_flags_get(window: window_handle_t) -> window_flag_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn window_flags_set(window: window_handle_t, flags: window_flag_t) -> window_flag_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn window_framebuffer_size_get(window: window_handle_t) -> window_size_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn window_framebuffer_size_set(
    window: window_handle_t,
    size: window_size_t,
) -> window_size_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn window_framebuffer_format_get(window: window_handle_t) -> pixel_format_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn window_framebuffer_get(window: window_handle_t) -> *mut framebuffer_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn window_present(
    window: window_handle_t,
    block: bool,
    rects: *mut window_rect_t,
    num_rects: ::core::ffi::c_int,
) {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn window_event_poll(
    window: window_handle_t,
    block: bool,
    timeout_msec: u32,
) -> event_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn get_screen_info(
    width: *mut ::core::ffi::c_int,
    height: *mut ::core::ffi::c_int,
    format: *mut pixel_format_t,
    refresh_rate: *mut f32,
) {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn task_priority_lower() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn task_priority_restore() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn get_num_tasks() -> u32 {
    unimplemented!("Implement this yourself if you need it");
}
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
pub extern "C" fn parse_path(
    path: *const ::core::ffi::c_char,
    result: *mut path_t,
) -> path_parse_result_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn path_free(path: *mut path_t) {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn mkdir_p(path: *const ::core::ffi::c_char) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn rm_rf(path: *const ::core::ffi::c_char) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn path_dirname(path: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn path_basename(path: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn path_devname(path: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn path_dirconcat(
    path: *const ::core::ffi::c_char,
    subdir: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn path_fileconcat(
    path: *const ::core::ffi::c_char,
    filename: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn path_concat(
    base_path: *const ::core::ffi::c_char,
    append_path: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn get_mac_address() -> *const ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn ota_session_open() -> ota_handle_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn ota_write(
    session: ota_handle_t,
    buffer: *mut ::core::ffi::c_void,
    block_size: ::core::ffi::c_int,
) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn ota_session_commit(session: ota_handle_t) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn ota_session_abort(session: ota_handle_t) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn ota_get_running_version(version: *mut ::core::ffi::c_char) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn ota_get_invalid_version(version: *mut ::core::ffi::c_char) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn process_create(
    path: *const ::core::ffi::c_char,
    stack_size: size_t,
    argc: ::core::ffi::c_int,
    argv: *mut *mut ::core::ffi::c_char,
) -> pid_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn thread_create(
    thread_entry: ::core::option::Option<unsafe extern "C" fn(user_data: *mut ::core::ffi::c_void)>,
    user_data: *mut ::core::ffi::c_void,
    stack_size: u16,
) -> pid_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn device_get(name: *const ::core::ffi::c_char) -> *mut device_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn vaddr_to_paddr(vaddr: u32) -> u32 {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn die(reason: *const ::core::ffi::c_char) {
    unimplemented!("Implement this yourself if you need it");
}

#[unsafe(no_mangle)]
pub extern "C" fn __riscv_save_0() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_save_1() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_save_2() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_save_3() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_save_4() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_save_5() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_save_6() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_save_7() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_save_8() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_save_9() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_save_10() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_save_11() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_save_12() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_restore_0() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_restore_1() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_restore_2() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_restore_3() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_restore_4() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_restore_5() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_restore_6() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_restore_7() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_restore_8() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_restore_9() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_restore_10() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_restore_11() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn __riscv_restore_12() {
    unimplemented!("Implement this yourself if you need it");
}
