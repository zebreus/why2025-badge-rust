use crate::types::*;

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
