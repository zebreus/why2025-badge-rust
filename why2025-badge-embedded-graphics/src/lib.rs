#![no_std]
extern crate alloc;

use alloc::ffi::CString;
use alloc::string::{String, ToString};
use core::ptr::null_mut;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::{geometry::Size, pixelcolor::Rgb888, prelude::*};
use why2025_badge_sys::{
    framebuffer_t, pixel_format_t, window_coords_t, window_create, window_flag_t, window_flags_get,
    window_flags_set, window_framebuffer_create, window_framebuffer_size_set, window_handle_t,
    window_position_get, window_position_set, window_present, window_size_get, window_size_set,
    window_size_t, window_title_set,
};

const fn rgb888_to_rgb565(r: u8, g: u8, b: u8) -> u16 {
    // Convert RGB888 to RGB565 format
    ((r as u16 & 0xF8) << 8) | ((g as u16 & 0xFC) << 3) | (b as u16 >> 3)
}

pub struct Why2025BadgeWindow {
    title: String,
    window: window_handle_t,
    framebuffer: *mut framebuffer_t,
}

pub struct Why2025BadgeWindowConfig {
    size: Size,
    title: String,
    fullscreen: bool,
}

impl Why2025BadgeWindowConfig {
    /// Config for a new floating Window
    pub fn new_floating(size: Size, title: &str) -> Self {
        Self {
            size,
            title: title.to_string(),
            fullscreen: false,
        }
    }
    /// Config for a new fullscreen Window
    pub fn new_fullscreen() -> Self {
        Self {
            size: Size {
                width: 400,
                height: 400,
            },
            title: String::new(),
            fullscreen: true,
        }
    }
    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }
}

impl Why2025BadgeWindow {
    /// Creates a new floating Window
    pub fn new_floating(size: Size, title: &str) -> Self {
        Self::new(Why2025BadgeWindowConfig::new_floating(size, title))
    }
    /// Creates a new fullscreen Window
    pub fn new_fullscreen() -> Self {
        Self::new(Why2025BadgeWindowConfig::new_fullscreen())
    }
    /// Creates a new Window with the given configuration.
    pub fn new(config: Why2025BadgeWindowConfig) -> Self {
        let size = window_size_t {
            w: config.size.width as i32,
            h: config.size.height as i32,
        };
        let title_bytes = config.title.as_bytes();
        assert!(
            title_bytes.len() < 127,
            "Title must be less than 127 bytes long"
        );

        let mut this = Why2025BadgeWindow {
            title: config.title,
            window: null_mut(),
            framebuffer: null_mut(),
        };

        let window = unsafe {
            window_create(
                CString::new(this.title.as_str()).unwrap().as_ptr(),
                size.clone(),
                window_flag_t::WINDOW_FLAG_DOUBLE_BUFFERED
                    | (if config.fullscreen {
                        window_flag_t::WINDOW_FLAG_UNDECORATED
                            | window_flag_t::WINDOW_FLAG_FULLSCREEN
                    } else {
                        window_flag_t::WINDOW_FLAG_NONE
                    }),
            )
        };

        let real_size = unsafe { window_size_get(window) };

        let framebuffer = unsafe {
            window_framebuffer_create(
                window,
                real_size.clone(),
                pixel_format_t::BADGEVMS_PIXELFORMAT_RGB565,
            )
        };

        this.window = window;
        this.framebuffer = framebuffer;

        this
    }

    /// Flush the window to the screen
    pub fn flush(&mut self) {
        unsafe { window_present(self.window, false, null_mut(), 0) };
    }

    /// Change the size of the window
    pub fn resize(&mut self, size: Size) {
        let new_size = window_size_t {
            w: size.width as i32,
            h: size.height as i32,
        };
        unsafe {
            window_size_set(self.window, new_size);
        }
        let real_size = unsafe { window_size_get(self.window) };
        unsafe {
            window_framebuffer_size_set(self.window, real_size);
        }
    }

    /// Get the position of the window
    pub fn get_position(&mut self) -> (u32, u32) {
        let position = unsafe { window_position_get(self.window) };
        (position.x as u32, position.y as u32)
    }
    /// Set the position of the window
    pub fn set_position(&mut self, position: (u32, u32)) {
        unsafe {
            window_position_set(
                self.window,
                window_coords_t {
                    x: position.0 as i32,
                    y: position.1 as i32,
                },
            );
        }
    }

    /// Check if the window is in fullscreen mode
    pub fn is_fullscreen(&self) -> bool {
        let flags = unsafe { window_flags_get(self.window) };
        flags & window_flag_t::WINDOW_FLAG_FULLSCREEN != window_flag_t::WINDOW_FLAG_NONE
    }
    /// Set the window to fullscreen mode
    pub fn set_fullscreen(&mut self, fullscreen: bool) -> bool {
        let flags = unsafe { window_flags_get(self.window) };
        let old_fullscreen =
            flags & window_flag_t::WINDOW_FLAG_FULLSCREEN != window_flag_t::WINDOW_FLAG_NONE;
        if old_fullscreen == fullscreen {
            return fullscreen; // No change needed
        }
        let flags = unsafe {
            window_flags_set(
                self.window,
                window_flag_t(flags.0 ^ window_flag_t::WINDOW_FLAG_FULLSCREEN.0),
            )
        };
        let real_size = unsafe { window_size_get(self.window) };
        unsafe {
            window_framebuffer_size_set(self.window, real_size);
        }
        return flags & window_flag_t::WINDOW_FLAG_FULLSCREEN != window_flag_t::WINDOW_FLAG_NONE;
    }
    /// Check if the window is undecorated (no title bar, no borders)
    pub fn is_undecorated(&self) -> bool {
        let flags = unsafe { window_flags_get(self.window) };
        flags & window_flag_t::WINDOW_FLAG_UNDECORATED != window_flag_t::WINDOW_FLAG_NONE
    }
    /// Set the window to be undecorated (no title bar, no borders)
    pub fn set_undecorated(&mut self, undecorated: bool) -> bool {
        let flags = unsafe { window_flags_get(self.window) };
        let old_undecorated =
            flags & window_flag_t::WINDOW_FLAG_UNDECORATED != window_flag_t::WINDOW_FLAG_NONE;
        if old_undecorated == undecorated {
            return undecorated; // No change needed
        }
        let flags = unsafe {
            window_flags_set(
                self.window,
                window_flag_t(flags.0 ^ window_flag_t::WINDOW_FLAG_UNDECORATED.0),
            )
        };
        return flags & window_flag_t::WINDOW_FLAG_UNDECORATED != window_flag_t::WINDOW_FLAG_NONE;
    }
    /// Check if the window is always on top
    pub fn is_always_on_top(&self) -> bool {
        let flags = unsafe { window_flags_get(self.window) };
        flags & window_flag_t::WINDOW_FLAG_ALWAYS_ON_TOP != window_flag_t::WINDOW_FLAG_NONE
    }
    /// Set the window to always be on top of other windows
    pub fn set_always_on_top(&mut self, always_on_top: bool) -> bool {
        let flags = unsafe { window_flags_get(self.window) };
        let old_always_on_top =
            flags & window_flag_t::WINDOW_FLAG_ALWAYS_ON_TOP != window_flag_t::WINDOW_FLAG_NONE;
        if old_always_on_top == always_on_top {
            return always_on_top; // No change needed
        }
        let flags = unsafe {
            window_flags_set(
                self.window,
                window_flag_t(flags.0 ^ window_flag_t::WINDOW_FLAG_ALWAYS_ON_TOP.0),
            )
        };
        return flags & window_flag_t::WINDOW_FLAG_ALWAYS_ON_TOP != window_flag_t::WINDOW_FLAG_NONE;
    }

    /// Get the title of the window
    pub fn get_title(&self) -> &str {
        &self.title
    }
    /// Set the title of the window
    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
        unsafe {
            window_title_set(
                self.window,
                CString::new(self.title.as_str()).unwrap().as_ptr(),
            );
        }
    }

    /// Get the raw handle of the window.
    ///
    /// Marked unsafe, because you can cause funny memory issues
    /// if you carelessly resize the window
    pub unsafe fn raw_handle(&self) -> window_handle_t {
        self.window
    }
}
impl OriginDimensions for Why2025BadgeWindow {
    fn size(&self) -> Size {
        // let size = unsafe { window_size_get(self.window) };
        Size {
            width: unsafe { (*self.framebuffer).w } as u32,
            height: unsafe { (*self.framebuffer).h } as u32,
        }
    }
}

impl DrawTarget for Why2025BadgeWindow {
    type Color = Rgb565;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), ()>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let size = self.size();
        let canvas_width = size.width as usize;
        let backing = unsafe {
            core::slice::from_raw_parts_mut(
                (*self.framebuffer).pixels,
                size.height as usize * size.width as usize,
            )
        };

        for pixel in pixels.into_iter() {
            let point = pixel.0;
            let offset: usize = point.y as usize * canvas_width + point.x as usize;
            if offset < backing.len() {
                let rgb: Rgb888 = pixel.1.into();
                let data = rgb888_to_rgb565(rgb.r(), rgb.g(), rgb.b());
                backing[offset] = data;
            }
        }

        Ok(())
    }
}
