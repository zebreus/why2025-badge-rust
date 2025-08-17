use crate::{
    emulated::badgevms::graphics::{
        input::WindowEventCallback,
        rendering::{present_windowbuffer, rescale_windowbuffer, update_windowbuffer},
    },
    types::*,
};
use minifb::{HasWindowHandle, Window, WindowOptions};
use raw_window_handle::{HasDisplayHandle, RawDisplayHandle, RawWindowHandle};
use std::sync::{Arc, LazyLock, RwLock, mpsc::channel};

mod input;
mod rendering;

#[derive(Debug)]
struct WindowData {
    window: minifb::Window,
    /// The title of the window, as a byte array.
    ///
    /// The title is utf-8 encoded and ends with a null byte.
    title: [u8; 128],
    /// Always the backing buffer for the real window. Not handed out to the user.
    /// Also stores the size it is currently at.
    ///
    /// The format is always ARGB8888
    windowbuffer: (Vec<u32>, (usize, usize)),
    /// Framebuffer thats handed out to the application.
    buffer565: Vec<u16>,
    /// Framebuffer thats currently displayed on the screen.
    /// We use this to avoid flickering when the application is resized.
    displayed_buffer565: (Vec<u16>, (usize, usize)),
    framebuffer: framebuffer_t,
    /// Wheter the topmost flag is set
    topmost: bool,
    /// Wheter the window is undecorated
    /// (no title bar, no borders)
    undecorated: bool,
    /// Wheter the window is fullscreen
    /// (covers the whole screen, no borders)
    fullscreen: bool,
    /// Receiver for input events
    receiver: std::sync::mpsc::Receiver<event_t>,
    /// When the window is ready to be used
    ready_at: Option<std::time::Instant>,
    /// Position and size of the window when it is not in fullscreen mode.
    floating_location: Option<(Option<(usize, usize)>, (usize, usize))>,
}
impl WindowData {
    /// Wait until the window is ready to be used.
    /// This is necessary because the window may not be ready immediately after creation.
    fn wait_until_ready(&self) {
        let Some(ready_at) = self.ready_at else {
            return;
        };
        std::thread::sleep_until(ready_at);
    }

    fn set_position(&mut self, position: (usize, usize)) {
        self.window
            .set_position(position.0 as isize, position.1 as isize);
    }

    fn get_position(&self) -> (usize, usize) {
        self.wait_until_ready();
        let position = self.window.get_position();
        (position.0 as usize, position.1 as usize)
    }

    fn set_size(&mut self, size: (usize, usize)) -> (usize, usize) {
        if self.fullscreen {
            // If we are in fullscreen mode, we cannot change the size
            // But we can save the size for later when we go back to floating mode
            self.floating_location = Some((self.floating_location.and_then(|(pos, _)| pos), size));
            return self.get_size();
        }

        let old_size = self.window.get_size();
        if old_size.0 == size.0 && old_size.1 == size.1 {
            // No change needed
            return size;
        }

        // Obtain handles for the X11 window and display
        let raw_handle = self.window.window_handle().unwrap().as_raw();
        let RawWindowHandle::Xlib(window_handle) = raw_handle else {
            panic!("Not a Xlib window handle");
        };
        let x11_window = window_handle.window;
        let display = self.window.display_handle().unwrap().as_raw();
        let RawDisplayHandle::Xlib(display_handle) = display else {
            panic!("Not a Xlib display handle");
        };
        let x11_display = display_handle.display.unwrap().as_ptr() as *mut x11_dl::xlib::Display;

        unsafe { ((XLIB).XResizeWindow)(x11_display, x11_window, size.0 as u32, size.1 as u32) };

        // Wait until the window has been resized
        let one_second_later = std::time::Instant::now() + std::time::Duration::from_secs(1);
        loop {
            let current_size = self.window.get_size();
            rescale_windowbuffer(self, Some(current_size));
            present_windowbuffer(self, Some(current_size));
            // println!("Current size: {:?}", current_size);
            if current_size.0 == size.0 && current_size.1 == size.1 {
                break current_size;
            }
            if current_size.0 != old_size.0 || current_size.1 != old_size.1 {
                // If the size didn't change, we can stop trying to resize
                break current_size;
            }
            if std::time::Instant::now() > one_second_later {
                // If we waited for more than a second, we give up
                eprintln!(
                    "Failed to resize window {} to {}x{} in time",
                    x11_window, size.0, size.1
                );
                break current_size;
            }
        }
    }

    fn get_size(&self) -> (usize, usize) {
        self.wait_until_ready();
        let size = self.window.get_size();
        (size.0 as usize, size.1 as usize)
    }
}
/// A few calls to get information about the window depend on the window being ready.
///
/// This is the duration we wait until we assume the window is ready.
const DURATION_UNTIL_WINDOW_READY: std::time::Duration = std::time::Duration::from_millis(150);

// This is fine.
unsafe impl Sync for WindowData {}
unsafe impl Send for WindowData {}

static WINDOWS: LazyLock<Arc<RwLock<Vec<Option<Arc<RwLock<WindowData>>>>>>> = LazyLock::new(|| {
    // Initialize the vector with a single None element, so index 0 is a kind of nullptr
    let initial_vec: Vec<Option<Arc<RwLock<WindowData>>>> = vec![None; 1];
    Arc::new(RwLock::new(initial_vec))
});

static XLIB: LazyLock<x11_dl::xlib::Xlib> =
    LazyLock::new(|| x11_dl::xlib::Xlib::open().expect("Failed to open Xlib library"));

static FULLSCREEN_WIDTH: usize = 720;
static FULLSCREEN_HEIGHT: usize = 720;

/// Create a new window with the given title and size.
///
/// This function will create a new window and return a handle to it.
/// The window will be created with the given flags.
///
/// The title must be a valid UTF-8 string and will be truncated to 127 bytes.
/// Try to keep the title well below 127 bytes, as UTF-8 graphemes will be cut off at the byte level,
///
/// The size must be a valid size, otherwise the window will not be created.
///
/// Currently the only supported flags are:
/// - `WINDOW_FLAG_ALWAYS_ON_TOP`: The window will always be on top of other windows
/// - `WINDOW_FLAG_UNDECORATED`: The window will not have a title bar or borders
/// - `WINDOW_FLAG_FULLSCREEN`: The window will be fullscreen
#[unsafe(no_mangle)]
pub extern "C" fn window_create(
    title: *const ::core::ffi::c_char,
    size: window_size_t,
    flags: window_flag_t,
) -> window_handle_t {
    let title_cstr = unsafe { std::ffi::CStr::from_ptr(title) };
    let title = title_cstr.to_string_lossy().into_owned();
    let title_len = title.len();
    let mut title_bytes = [0u8; 128];
    title_bytes[..title_len.min(127)].copy_from_slice(title.as_bytes());

    let topmost =
        (flags & window_flag_t::WINDOW_FLAG_ALWAYS_ON_TOP) != window_flag_t::WINDOW_FLAG_NONE;
    let undecorated =
        (flags & window_flag_t::WINDOW_FLAG_UNDECORATED) != window_flag_t::WINDOW_FLAG_NONE;
    let fullscreen =
        (flags & window_flag_t::WINDOW_FLAG_FULLSCREEN) != window_flag_t::WINDOW_FLAG_NONE;

    let window_size = (size.w as usize, size.h as usize);
    let size = if fullscreen {
        (FULLSCREEN_WIDTH, FULLSCREEN_HEIGHT)
    } else {
        window_size
    };

    let mut options = WindowOptions::default();
    options.borderless = undecorated || fullscreen;
    options.resize = true;
    options.scale = minifb::Scale::X1;
    options.transparency = false;
    options.title = true;
    options.none = false;
    options.scale_mode = minifb::ScaleMode::Center;
    options.topmost = topmost;

    let mut window = Window::new(&title, size.0, size.1, options).unwrap();
    window.set_target_fps(9999999);
    // Basically disable key repeat, as we don't support it in the native bindings.
    window.set_key_repeat_delay(9999999f32);

    let mut buffer565: Vec<u16> = vec![0; size.0 * size.1];
    let framebuffer = framebuffer_t {
        w: size.0 as u32,
        h: size.1 as u32,
        format: pixel_format_t::BADGEVMS_PIXELFORMAT_RGB565,
        pixels: buffer565.as_mut_ptr(),
    };

    // badgevms only sends KEY_DOWN and KEY_UP events
    // it never sends EVENT_QUIT or EVENT_RESIZE
    // ... which is nice, because handling resize with minifb would be a pain.
    let (input_event_sender, input_event_receiver) = channel::<event_t>();

    let input_handler = Box::new(WindowEventCallback::new(input_event_sender));
    window.set_input_callback(input_handler);

    let window: WindowData = WindowData {
        window,
        title: title_bytes,
        windowbuffer: (vec![0; size.0 * size.1], (size.0, size.1)),
        displayed_buffer565: (Vec::new(), (framebuffer.w as usize, framebuffer.h as usize)),
        buffer565,
        framebuffer: framebuffer,
        topmost,
        undecorated,
        fullscreen,
        receiver: input_event_receiver,
        ready_at: Some(std::time::Instant::now() + DURATION_UNTIL_WINDOW_READY),
        floating_location: if fullscreen {
            Some((None, window_size))
        } else {
            None
        },
    };

    let mut windows = WINDOWS.write().unwrap();
    let index = windows.len();
    windows.push(Some(Arc::new(RwLock::new(window))));

    return index as window_handle_t;
}
/// Create a framebuffer for the window.
///
/// This is essentially a noop in the emulated implementation, as we always have a framebuffer.
#[unsafe(no_mangle)]
pub extern "C" fn window_framebuffer_create(
    window: window_handle_t,
    size: window_size_t,
    pixel_format: pixel_format_t,
) -> *mut framebuffer_t {
    if pixel_format != pixel_format_t::BADGEVMS_PIXELFORMAT_RGB565 {
        unimplemented!(
            "For now the native bindings only supports RGB565. Feel free to contribute support for your pixel format."
        );
    }

    window_framebuffer_size_set(window, size);
    return window_framebuffer_get(window);
}
/// Close the window and free all resources associated with it.
///
/// This will also destroy the framebuffer and the window buffer.
/// Accessing the window after this will result in undefined behavior (most likely a crash).
#[unsafe(no_mangle)]
pub extern "C" fn window_destroy(window: window_handle_t) {
    let mut windows = WINDOWS.write().unwrap();
    let window_data = windows.get_mut(window as usize).unwrap();
    *window_data = None;
}
/// Get the title of the window.
#[unsafe(no_mangle)]
pub extern "C" fn window_title_get(window: window_handle_t) -> *const ::core::ffi::c_char {
    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let window_data = window_data.read().unwrap();
    return window_data.title.as_ptr() as *const ::core::ffi::c_char;
}
/// Set the title of the window.
///
/// If your title is longer than 127 bytes, it will be truncated to 127 bytes.
///
/// If your title is not a valid UTF-8 string, it may loose some characters or not display correctly.
#[unsafe(no_mangle)]
pub extern "C" fn window_title_set(window: window_handle_t, title: *const ::core::ffi::c_char) {
    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let mut window_data = window_data.write().unwrap();

    let title_cstr = unsafe { std::ffi::CStr::from_ptr(title) };
    let title = title_cstr.to_string_lossy().into_owned();
    let title_len = title.len();
    let mut title_bytes = [0u8; 128];
    title_bytes[..title_len.min(127)].copy_from_slice(title.as_bytes());

    window_data.title = title_bytes;
    window_data.window.set_title(&title);
}
/// Get the position of the window.
#[unsafe(no_mangle)]
pub extern "C" fn window_position_get(window: window_handle_t) -> window_coords_t {
    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let window_data = window_data.read().unwrap();
    let position = window_data.get_position();
    return window_coords_t {
        x: position.0 as i32,
        y: position.1 as i32,
    };
}
/// Set the position of the window.
#[unsafe(no_mangle)]
pub extern "C" fn window_position_set(
    window: window_handle_t,
    coords: window_coords_t,
) -> window_coords_t {
    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let mut window_data = window_data.write().unwrap();
    window_data.set_position((coords.x as usize, coords.y as usize));
    let new_position = window_data.get_position();
    return window_coords_t {
        x: new_position.0 as i32,
        y: new_position.1 as i32,
    };
}
/// Get the current size of the window.
#[unsafe(no_mangle)]
pub extern "C" fn window_size_get(window: window_handle_t) -> window_size_t {
    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let window_data = window_data.read().unwrap();
    let position = window_data.get_size();
    return window_size_t {
        w: position.0 as i32,
        h: position.1 as i32,
    };
}
/// Set the size of the window.
///
/// This does not change the size of the framebuffer, you have to do that manually.
#[unsafe(no_mangle)]
pub extern "C" fn window_size_set(window: window_handle_t, size: window_size_t) -> window_size_t {
    if window as usize == 0 {
        // The C implementation also does this.
        return window_size_t { w: 0, h: 0 };
    }

    let windows: std::sync::RwLockReadGuard<'_, Vec<Option<Arc<RwLock<WindowData>>>>> =
        WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let mut window_data = window_data.write().unwrap();

    let size = window_data.set_size((size.w as usize, size.h as usize));

    return window_size_t {
        w: size.0 as i32,
        h: size.1 as i32,
    };
}
/// Get the currently set window flags.
///
/// The emulated implementation does only support the always on top, undecorated, and fullscreen flags.
#[unsafe(no_mangle)]
pub extern "C" fn window_flags_get(window: window_handle_t) -> window_flag_t {
    let windows: std::sync::RwLockReadGuard<'_, Vec<Option<Arc<RwLock<WindowData>>>>> =
        WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let window_data = window_data.read().unwrap();
    let mut flags: window_flag_t = window_flag_t::WINDOW_FLAG_NONE;
    if window_data.fullscreen {
        flags |= window_flag_t::WINDOW_FLAG_FULLSCREEN;
    }
    if window_data.topmost {
        flags |= window_flag_t::WINDOW_FLAG_ALWAYS_ON_TOP;
    }
    if window_data.undecorated {
        flags |= window_flag_t::WINDOW_FLAG_UNDECORATED;
    }
    flags
}
/// Set window flags.
///
/// Currently only supports setting the topmost and fullscreen flags.
#[unsafe(no_mangle)]
pub extern "C" fn window_flags_set(window: window_handle_t, flags: window_flag_t) -> window_flag_t {
    let topmost =
        (flags & window_flag_t::WINDOW_FLAG_ALWAYS_ON_TOP) != window_flag_t::WINDOW_FLAG_NONE;
    // Cant change undecorated after creation
    let fullscreen =
        (flags & window_flag_t::WINDOW_FLAG_FULLSCREEN) != window_flag_t::WINDOW_FLAG_NONE;
    {
        let windows = WINDOWS.read().unwrap();
        let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
        let mut window_data = window_data.write().unwrap();

        if fullscreen && !window_data.fullscreen {
            // If we are going fullscreen, we need to set the size to the fullscreen size
            let size = window_data.get_size();
            window_data.floating_location = Some((Some(window_data.get_position()), size));
            window_data.set_size((FULLSCREEN_WIDTH, FULLSCREEN_HEIGHT));
            window_data.fullscreen = true;
        }
        if !fullscreen && window_data.fullscreen {
            window_data.fullscreen = false;
            // If we are going back to floating mode, we need to restore the previous position and size
            if let Some((position, size)) = window_data.floating_location {
                if let Some(position) = position {
                    window_data.set_position(position);
                }
                window_data.set_size(size);
            } else {
                // If we don't have a previous position and size, we just set it to 300x300 and keep the current position
                window_data.set_size((300, 300));
            }
        }
        if window_data.topmost != topmost {
            window_data.topmost = topmost;
        }
    }
    // TODO: Implement fullscreen
    return window_flags_get(window);
}
/// Get the current size of the framebuffer.
#[unsafe(no_mangle)]
pub extern "C" fn window_framebuffer_size_get(window: window_handle_t) -> window_size_t {
    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let window_data = window_data.read().unwrap();
    return window_size_t {
        w: window_data.framebuffer.w as i32,
        h: window_data.framebuffer.h as i32,
    };
}
/// Set the framebuffer size.
///
/// If the framebuffer size does not match the window size, the framebuffer will
/// be rendered starting in the top-left corner of the window.
///
/// Not implemented in badgevms, so I am guessing the behavior here.
#[unsafe(no_mangle)]
pub extern "C" fn window_framebuffer_size_set(
    window: window_handle_t,
    size: window_size_t,
) -> window_size_t {
    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let mut window_data = window_data.write().unwrap();
    if window_data.framebuffer.w as usize == size.w as usize
        && window_data.framebuffer.h as usize == size.h as usize
    {
        // No change needed
        return size;
    }

    window_data.framebuffer.w = size.w as u32;
    window_data.framebuffer.h = size.h as u32;

    window_data.buffer565.fill(0);
    window_data
        .buffer565
        .resize(size.w as usize * size.h as usize, 0);
    // Set the pointer again in case it got reallocated
    window_data.framebuffer.pixels = window_data.buffer565.as_mut_ptr();

    size
}
#[unsafe(no_mangle)]
pub extern "C" fn window_framebuffer_format_get(window: window_handle_t) -> pixel_format_t {
    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let window_data = window_data.read().unwrap();
    return window_data.framebuffer.format;
}
#[unsafe(no_mangle)]
pub extern "C" fn window_framebuffer_get(window: window_handle_t) -> *mut framebuffer_t {
    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let mut window_data = window_data.write().unwrap();
    let framebuffer: *mut framebuffer_t = &mut window_data.framebuffer as *mut framebuffer_t;
    return framebuffer;
}
/// Present the window with the current framebuffer.
///
/// The emulated implementation always blocks as it is not event based.
///
/// `rects` and `num_rects` are not implemented in badgevms, so we also ignore them here.
#[unsafe(no_mangle)]
pub extern "C" fn window_present(
    window: window_handle_t,
    _block: bool,
    _rects: *mut window_rect_t,
    _num_rects: ::core::ffi::c_int,
) {
    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let mut window_data = window_data.write().unwrap();

    update_windowbuffer(&mut window_data, None);
    present_windowbuffer(&mut window_data, None);
}

#[unsafe(no_mangle)]
pub extern "C" fn window_event_poll(
    window: window_handle_t,
    block: bool,
    timeout_millis: u32,
) -> event_t {
    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let window_data = window_data.read().unwrap();

    let event = if block {
        window_data
            .receiver
            .recv_timeout(std::time::Duration::from_millis(timeout_millis as u64))
            .ok()
    } else {
        window_data.receiver.recv().ok()
    };

    let Some(event) = event else {
        return event_t::new_empty();
    };

    return event;
}

#[unsafe(no_mangle)]
pub extern "C" fn get_screen_info(
    width: *mut ::core::ffi::c_int,
    height: *mut ::core::ffi::c_int,
    format: *mut pixel_format_t,
    refresh_rate: *mut f32,
) {
    // TODO: Use real information
    unsafe {
        if !width.is_null() {
            *width = FULLSCREEN_WIDTH as i32; // Width of the badge
        }
        if !height.is_null() {
            *height = FULLSCREEN_HEIGHT as i32; // Height of the badge
        }
        if !format.is_null() {
            *format = pixel_format_t::BADGEVMS_PIXELFORMAT_RGB565; // Only supported color format
        }
        if !refresh_rate.is_null() {
            *refresh_rate = 60.0; // Example refresh rate
        }
    }
}
