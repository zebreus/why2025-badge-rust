use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use spin::{Lazy, Mutex};

use crate::runtime;
use crate::types::*;

mod input;
mod rendering;
mod x11;

use input::empty_event;
use rendering::{rescale_windowbuffer, update_windowbuffer};

const FULLSCREEN_WIDTH: usize = 720;
const FULLSCREEN_HEIGHT: usize = 720;
const FULLSCREEN_REFRESH_RATE: f32 = 60.0;

type WindowHandleEntry = Option<Arc<Mutex<WindowData>>>;

static WINDOWS: Lazy<Mutex<Vec<WindowHandleEntry>>> = Lazy::new(|| Mutex::new(vec![None]));

pub(crate) struct WindowData {
    title: [u8; 128],
    position: (usize, usize),
    size: (usize, usize),
    windowbuffer: (Vec<u32>, (usize, usize)),
    buffer565: Vec<u16>,
    displayed_buffer565: (Vec<u16>, (usize, usize)),
    framebuffer: framebuffer_t,
    topmost: bool,
    undecorated: bool,
    fullscreen: bool,
    event_queue: VecDeque<event_t>,
    floating_location: Option<(Option<(usize, usize)>, (usize, usize))>,
    backend: Option<x11::X11Window>,
}

unsafe impl Send for WindowData {}
unsafe impl Sync for WindowData {}

impl WindowData {
    fn new(
        title: [u8; 128],
        requested_size: (usize, usize),
        topmost: bool,
        undecorated: bool,
        fullscreen: bool,
    ) -> Self {
        let size = if fullscreen {
            (FULLSCREEN_WIDTH, FULLSCREEN_HEIGHT)
        } else {
            requested_size
        };
        let mut buffer565 = vec![0; size.0 * size.1];
        let framebuffer = framebuffer_t {
            w: size.0 as u32,
            h: size.1 as u32,
            format: pixel_format_t::BADGEVMS_PIXELFORMAT_RGB565,
            pixels: buffer565.as_mut_ptr(),
        };
        #[cfg(test)]
        let backend = None;
        #[cfg(not(test))]
        let backend = Some(
            x11::X11Window::try_open(&title, size, undecorated || fullscreen).unwrap_or_else(
                || {
                    runtime::abort_with_message(
                        b"why2025-badge-emu-abi failed to create an X11 window backend\n",
                    )
                },
            ),
        );

        Self {
            title,
            position: (0, 0),
            size,
            windowbuffer: (vec![0; size.0 * size.1], size),
            displayed_buffer565: (
                buffer565.clone(),
                (framebuffer.w as usize, framebuffer.h as usize),
            ),
            buffer565,
            framebuffer,
            topmost,
            undecorated,
            fullscreen,
            event_queue: VecDeque::new(),
            floating_location: if fullscreen {
                Some((None, requested_size))
            } else {
                None
            },
            backend,
        }
    }

    fn get_position(&self) -> (usize, usize) {
        self.position
    }

    fn set_position(&mut self, position: (usize, usize)) {
        if self.fullscreen {
            let floating_size = self
                .floating_location
                .map(|(_, size)| size)
                .unwrap_or(self.size);
            self.floating_location = Some((Some(position), floating_size));
            return;
        }

        self.position = position;
        if let Some(backend) = self.backend.as_mut() {
            backend.set_position(position);
        }
    }

    fn get_size(&self) -> (usize, usize) {
        self.size
    }

    fn set_size(&mut self, size: (usize, usize)) -> (usize, usize) {
        if self.fullscreen {
            self.floating_location = Some((self.floating_location.and_then(|(pos, _)| pos), size));
            return self.get_size();
        }

        if self.size == size {
            return size;
        }

        self.size = size;
        rescale_windowbuffer(self, Some(size));
        if let Some(backend) = self.backend.as_mut() {
            backend.set_size(size);
        }
        size
    }

    fn set_framebuffer_size(&mut self, size: (usize, usize)) -> (usize, usize) {
        if self.framebuffer.w as usize == size.0 && self.framebuffer.h as usize == size.1 {
            return size;
        }

        self.framebuffer.w = size.0 as u32;
        self.framebuffer.h = size.1 as u32;
        self.buffer565.fill(0);
        self.buffer565.resize(size.0 * size.1, 0);
        self.framebuffer.pixels = self.buffer565.as_mut_ptr();
        size
    }

    fn flags(&self) -> window_flag_t {
        let mut flags = window_flag_t::WINDOW_FLAG_NONE;
        if self.fullscreen {
            flags |= window_flag_t::WINDOW_FLAG_FULLSCREEN;
        }
        if self.topmost {
            flags |= window_flag_t::WINDOW_FLAG_ALWAYS_ON_TOP;
        }
        if self.undecorated {
            flags |= window_flag_t::WINDOW_FLAG_UNDECORATED;
        }
        flags
    }

    fn sync_backend(&mut self) {
        let Some(update) = self.backend.as_mut().map(|backend| backend.pump_events()) else {
            return;
        };

        if let Some(position) = update.position {
            self.position = position;
        }

        if let Some(size) = update.size {
            self.size = size;
            rescale_windowbuffer(self, Some(size));
        }

        self.event_queue.extend(update.events);
    }
}

fn monotonic_millis() -> u64 {
    unsafe {
        let mut ts: libc::timespec = core::mem::zeroed();
        if libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts) != 0 {
            return 0;
        }

        (ts.tv_sec as u64)
            .saturating_mul(1_000)
            .saturating_add((ts.tv_nsec as u64) / 1_000_000)
    }
}

fn wait_without_backend(timeout_msec: Option<u32>) -> bool {
    let timeout = timeout_msec
        .map(|timeout_msec| timeout_msec.min(i32::MAX as u32) as c_int)
        .unwrap_or(-1);

    unsafe { libc::poll(ptr::null_mut(), 0, timeout) > 0 }
}

fn window_data(window: window_handle_t) -> Arc<Mutex<WindowData>> {
    let windows = WINDOWS.lock();
    windows
        .get(window as usize)
        .and_then(|entry| entry.as_ref())
        .cloned()
        .expect("invalid window handle")
}

fn title_bytes_from_ptr(title: *const c_char) -> [u8; 128] {
    let mut title_bytes = [0u8; 128];
    if title.is_null() {
        return title_bytes;
    }

    let title_cstr = unsafe { CStr::from_ptr(title) };
    let bytes = title_cstr.to_bytes();
    let len = bytes.len().min(127);
    title_bytes[..len].copy_from_slice(&bytes[..len]);
    title_bytes
}

fn window_size_tuple(size: window_size_t) -> (usize, usize) {
    (size.w.max(0) as usize, size.h.max(0) as usize)
}

fn coords_tuple(coords: window_coords_t) -> (usize, usize) {
    (coords.x.max(0) as usize, coords.y.max(0) as usize)
}

fn unsupported_pixel_format() -> ! {
    runtime::abort_with_message(
        b"why2025-badge-emu-abi graphics only supports RGB565 framebuffers\n",
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn window_create(
    title: *const c_char,
    size: window_size_t,
    flags: window_flag_t,
) -> window_handle_t {
    let title_bytes = title_bytes_from_ptr(title);
    let topmost =
        (flags & window_flag_t::WINDOW_FLAG_ALWAYS_ON_TOP) != window_flag_t::WINDOW_FLAG_NONE;
    let undecorated =
        (flags & window_flag_t::WINDOW_FLAG_UNDECORATED) != window_flag_t::WINDOW_FLAG_NONE;
    let fullscreen =
        (flags & window_flag_t::WINDOW_FLAG_FULLSCREEN) != window_flag_t::WINDOW_FLAG_NONE;
    let window = WindowData::new(
        title_bytes,
        window_size_tuple(size),
        topmost,
        undecorated,
        fullscreen,
    );

    let mut windows = WINDOWS.lock();
    let index = windows.len();
    windows.push(Some(Arc::new(Mutex::new(window))));
    index as window_handle_t
}

#[unsafe(no_mangle)]
pub extern "C" fn window_framebuffer_create(
    window: window_handle_t,
    size: window_size_t,
    pixel_format: pixel_format_t,
) -> *mut framebuffer_t {
    if pixel_format != pixel_format_t::BADGEVMS_PIXELFORMAT_RGB565 {
        unsupported_pixel_format();
    }

    window_framebuffer_size_set(window, size);
    window_framebuffer_get(window)
}

#[unsafe(no_mangle)]
pub extern "C" fn window_destroy(window: window_handle_t) {
    let mut windows = WINDOWS.lock();
    let entry = windows
        .get_mut(window as usize)
        .expect("invalid window handle for destroy");
    *entry = None;
}

#[unsafe(no_mangle)]
pub extern "C" fn window_title_get(window: window_handle_t) -> *const c_char {
    let window = window_data(window);
    let window = window.lock();
    window.title.as_ptr() as *const c_char
}

#[unsafe(no_mangle)]
pub extern "C" fn window_title_set(window: window_handle_t, title: *const c_char) {
    let window = window_data(window);
    let mut window = window.lock();
    let title_bytes = title_bytes_from_ptr(title);
    window.title = title_bytes;
    if let Some(backend) = window.backend.as_mut() {
        backend.set_title(&title_bytes);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn window_position_get(window: window_handle_t) -> window_coords_t {
    let window = window_data(window);
    let mut window = window.lock();
    window.sync_backend();
    let position = window.get_position();
    window_coords_t {
        x: position.0 as c_int,
        y: position.1 as c_int,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn window_position_set(
    window: window_handle_t,
    coords: window_coords_t,
) -> window_coords_t {
    let window = window_data(window);
    let mut window = window.lock();
    window.set_position(coords_tuple(coords));
    let position = window.get_position();
    window_coords_t {
        x: position.0 as c_int,
        y: position.1 as c_int,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn window_size_get(window: window_handle_t) -> window_size_t {
    let window = window_data(window);
    let mut window = window.lock();
    window.sync_backend();
    let size = window.get_size();
    window_size_t {
        w: size.0 as c_int,
        h: size.1 as c_int,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn window_size_set(window: window_handle_t, size: window_size_t) -> window_size_t {
    if window.is_null() {
        return window_size_t { w: 0, h: 0 };
    }

    let window = window_data(window);
    let mut window = window.lock();
    let size = window.set_size(window_size_tuple(size));
    window_size_t {
        w: size.0 as c_int,
        h: size.1 as c_int,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn window_flags_get(window: window_handle_t) -> window_flag_t {
    let window = window_data(window);
    let mut window = window.lock();
    window.sync_backend();
    window.flags()
}

#[unsafe(no_mangle)]
pub extern "C" fn window_flags_set(window: window_handle_t, flags: window_flag_t) -> window_flag_t {
    let topmost =
        (flags & window_flag_t::WINDOW_FLAG_ALWAYS_ON_TOP) != window_flag_t::WINDOW_FLAG_NONE;
    let fullscreen =
        (flags & window_flag_t::WINDOW_FLAG_FULLSCREEN) != window_flag_t::WINDOW_FLAG_NONE;

    let window = window_data(window);
    let mut window = window.lock();
    window.sync_backend();

    if fullscreen && !window.fullscreen {
        let size = window.get_size();
        window.floating_location = Some((Some(window.get_position()), size));
        window.fullscreen = true;
        window.size = (FULLSCREEN_WIDTH, FULLSCREEN_HEIGHT);
        let fullscreen_size = window.size;
        rescale_windowbuffer(&mut window, Some(fullscreen_size));
    }

    if !fullscreen && window.fullscreen {
        window.fullscreen = false;
        if let Some((position, size)) = window.floating_location {
            if let Some(position) = position {
                window.position = position;
            }
            window.size = size;
        } else {
            window.size = (300, 300);
        }
        let restored_size = window.size;
        rescale_windowbuffer(&mut window, Some(restored_size));
    }

    window.topmost = topmost;
    let position = window.position;
    let size = window.size;
    if let Some(backend) = window.backend.as_mut() {
        backend.set_position(position);
        backend.set_size(size);
    }
    window.flags()
}

#[unsafe(no_mangle)]
pub extern "C" fn window_framebuffer_size_get(window: window_handle_t) -> window_size_t {
    let window = window_data(window);
    let window = window.lock();
    window_size_t {
        w: window.framebuffer.w as c_int,
        h: window.framebuffer.h as c_int,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn window_framebuffer_size_set(
    window: window_handle_t,
    size: window_size_t,
) -> window_size_t {
    let window = window_data(window);
    let mut window = window.lock();
    let size = window.set_framebuffer_size(window_size_tuple(size));
    window_size_t {
        w: size.0 as c_int,
        h: size.1 as c_int,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn window_framebuffer_format_get(window: window_handle_t) -> pixel_format_t {
    let window = window_data(window);
    let window = window.lock();
    window.framebuffer.format
}

#[unsafe(no_mangle)]
pub extern "C" fn window_framebuffer_get(window: window_handle_t) -> *mut framebuffer_t {
    let window = window_data(window);
    let mut window = window.lock();
    &mut window.framebuffer as *mut framebuffer_t
}

#[unsafe(no_mangle)]
pub extern "C" fn window_present(
    window: window_handle_t,
    block: bool,
    rects: *mut window_rect_t,
    num_rects: c_int,
) {
    let _ = (block, rects, num_rects);
    let window = window_data(window);
    let mut window = window.lock();
    window.sync_backend();
    let size = window.get_size();
    update_windowbuffer(&mut window, Some(size));
    let WindowData {
        backend,
        windowbuffer,
        ..
    } = &mut *window;
    if let Some(backend) = backend.as_mut() {
        backend.present(&windowbuffer.0, windowbuffer.1);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn window_event_poll(
    window: window_handle_t,
    block: bool,
    timeout_msec: u32,
) -> event_t {
    let window = window_data(window);
    let mut window = window.lock();

    let deadline = block.then(|| monotonic_millis().saturating_add(timeout_msec as u64));
    loop {
        window.sync_backend();
        if let Some(event) = window.event_queue.pop_front() {
            return event;
        }

        let wait_timeout = deadline.map(|deadline| {
            let now = monotonic_millis();
            if now >= deadline {
                0
            } else {
                (deadline - now).min(u32::MAX as u64) as u32
            }
        });

        if matches!(wait_timeout, Some(0)) {
            return empty_event();
        }

        let _ = if let Some(backend) = window.backend.as_ref() {
            backend.wait_for_event(wait_timeout)
        } else {
            wait_without_backend(wait_timeout)
        };

        if deadline.is_some() && monotonic_millis() >= deadline.unwrap_or_default() {
            window.sync_backend();
            if let Some(event) = window.event_queue.pop_front() {
                return event;
            }
            return empty_event();
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_screen_info(
    width: *mut c_int,
    height: *mut c_int,
    format: *mut pixel_format_t,
    refresh_rate: *mut f32,
) {
    unsafe {
        if !width.is_null() {
            *width = FULLSCREEN_WIDTH as c_int;
        }
        if !height.is_null() {
            *height = FULLSCREEN_HEIGHT as c_int;
        }
        if !format.is_null() {
            *format = pixel_format_t::BADGEVMS_PIXELFORMAT_RGB565;
        }
        if !refresh_rate.is_null() {
            *refresh_rate = FULLSCREEN_REFRESH_RATE;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};

    fn window_title(window: window_handle_t) -> &'static CStr {
        unsafe { CStr::from_ptr(window_title_get(window)) }
    }

    #[test]
    fn graphics_symbols_have_addresses() {
        assert_ne!(window_create as *const (), core::ptr::null());
        assert_ne!(window_present as *const (), core::ptr::null());
        assert_ne!(get_screen_info as *const (), core::ptr::null());
    }

    #[test]
    fn create_set_and_destroy_window() {
        let title = CString::new("hello").unwrap();
        let window = window_create(
            title.as_ptr(),
            window_size_t { w: 12, h: 8 },
            window_flag_t::WINDOW_FLAG_ALWAYS_ON_TOP,
        );
        assert!(!window.is_null());
        assert_eq!(window_size_get(window).w, 12);
        assert_eq!(window_size_get(window).h, 8);
        assert_eq!(window_title(window).to_bytes(), b"hello");
        assert_eq!(
            window_flags_get(window) & window_flag_t::WINDOW_FLAG_ALWAYS_ON_TOP,
            window_flag_t::WINDOW_FLAG_ALWAYS_ON_TOP,
        );

        let updated_title = CString::new("updated").unwrap();
        window_title_set(window, updated_title.as_ptr());
        assert_eq!(window_title(window).to_bytes(), b"updated");

        window_destroy(window);
    }

    #[test]
    fn framebuffer_resize_updates_metadata() {
        let title = CString::new("fb").unwrap();
        let window = window_create(
            title.as_ptr(),
            window_size_t { w: 2, h: 2 },
            window_flag_t::WINDOW_FLAG_NONE,
        );

        let framebuffer = window_framebuffer_create(
            window,
            window_size_t { w: 3, h: 4 },
            pixel_format_t::BADGEVMS_PIXELFORMAT_RGB565,
        );
        assert!(!framebuffer.is_null());
        assert_eq!(unsafe { (*framebuffer).w }, 3);
        assert_eq!(unsafe { (*framebuffer).h }, 4);
        assert_eq!(
            window_framebuffer_format_get(window),
            pixel_format_t::BADGEVMS_PIXELFORMAT_RGB565,
        );

        window_destroy(window);
    }

    #[test]
    fn present_copies_and_clears_framebuffer() {
        let title = CString::new("present").unwrap();
        let window = window_create(
            title.as_ptr(),
            window_size_t { w: 1, h: 1 },
            window_flag_t::WINDOW_FLAG_NONE,
        );
        let framebuffer = window_framebuffer_get(window);
        unsafe {
            *(*framebuffer).pixels = 0xffff;
        }

        window_present(window, false, core::ptr::null_mut(), 0);

        unsafe {
            assert_eq!(*(*framebuffer).pixels, 0);
        }
        window_destroy(window);
    }

    #[test]
    fn resize_before_first_present_keeps_blank_windowbuffer() {
        let mut window = WindowData::new([0; 128], (2, 2), false, false, false);

        rescale_windowbuffer(&mut window, Some((4, 3)));

        assert_eq!(window.windowbuffer.1, (4, 3));
        assert_eq!(window.windowbuffer.0.len(), 12);
        assert!(window.windowbuffer.0.iter().all(|pixel| *pixel == 0));
    }

    #[test]
    fn get_screen_info_reports_fixed_badge_values() {
        let mut width = 0;
        let mut height = 0;
        let mut format = pixel_format_t::BADGEVMS_PIXELFORMAT_UNKNOWN;
        let mut refresh = 0.0;

        get_screen_info(&mut width, &mut height, &mut format, &mut refresh);

        assert_eq!(width, FULLSCREEN_WIDTH as c_int);
        assert_eq!(height, FULLSCREEN_HEIGHT as c_int);
        assert_eq!(format, pixel_format_t::BADGEVMS_PIXELFORMAT_RGB565);
        assert_eq!(refresh, FULLSCREEN_REFRESH_RATE);
    }

    #[test]
    fn empty_queue_returns_empty_event() {
        let title = CString::new("events").unwrap();
        let window = window_create(
            title.as_ptr(),
            window_size_t { w: 1, h: 1 },
            window_flag_t::WINDOW_FLAG_NONE,
        );

        assert_eq!(
            window_event_poll(window, true, 0).type_,
            event_type_t::EVENT_NONE
        );
        window_destroy(window);
    }
}
