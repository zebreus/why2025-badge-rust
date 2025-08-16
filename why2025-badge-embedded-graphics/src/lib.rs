#![no_std]
extern crate alloc;

use alloc::string::{String, ToString};
use core::time::Duration;
use core::{ffi::c_char, ptr::null_mut};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::{geometry::Size, pixelcolor::Rgb888, prelude::*};
use why2025_badge_sys::{
    event_t, event_type, framebuffer_t, key_code_t, key_mod_t, keyboard_event_t, keyboard_scancode_t, pixel_format_t, window_create, window_flag_t, window_framebuffer_create, window_handle_t, window_present, window_size_t
};

const fn rgb888_to_rgb565(r: u8, g: u8, b: u8) -> u16 {
    // Convert RGB888 to RGB565 format
    ((r as u16 & 0xF8) << 8) | ((g as u16 & 0xFC) << 3) | (b as u16 >> 3)
}

pub struct Why2025BadgeWindow {
    size: (i32, i32),
    title: [u8; 64],
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
                width: 720,
                height: 720,
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
            title_bytes.len() < 63,
            "Title must be less than 63 bytes long"
        );

        let mut this = Why2025BadgeWindow {
            size: (size.w, size.h),
            title: [0; 64],
            window: null_mut(),
            framebuffer: null_mut(),
        };
        this.title[..title_bytes.len()].copy_from_slice(title_bytes);

        let window = unsafe {
            window_create(
                this.title.as_ptr() as *const u8,
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

        let framebuffer = unsafe {
            window_framebuffer_create(
                window,
                size.clone(),
                pixel_format_t::BADGEVMS_PIXELFORMAT_RGB565,
            )
        };

        this.window = window;
        this.framebuffer = framebuffer;

        this
    }

    pub fn handle(&self) -> window_handle_t {
        self.window
    }

    pub fn events(&self) -> EventPump {
        EventPump(self.window)
    }

    pub fn flush(&mut self) {
        unsafe { window_present(self.window, false, null_mut(), 0) };
    }
}
impl OriginDimensions for Why2025BadgeWindow {
    fn size(&self) -> Size {
        Size {
            width: self.size.0 as u32,
            height: self.size.1 as u32,
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
        let canvas_width = self.size.0 as usize;
        let backing = unsafe {
            core::slice::from_raw_parts_mut(
                (*self.framebuffer).pixels,
                self.size.0 as usize * self.size.1 as usize,
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

#[derive(Debug, Clone)]
pub struct KeyboardEvent {
    pub timestamp: u64,
    pub scancode: keyboard_scancode_t,
    pub key: key_code_t,
    pub modifier: key_mod_t,
    pub text: c_char,
    pub down: bool,
    pub repeat: bool,
}

pub struct EventPump(window_handle_t);

impl EventPump {
    pub fn poll(&self, block: bool, delay: Duration) -> Option<Event> {
        Event::from_raw(unsafe {
            why2025_badge_sys::window_event_poll(self.0, block, delay.as_millis() as u32)
        })
    }
}


#[derive(Debug, Clone)]
pub enum Event {
    Quit,
    KeyDown(KeyboardEvent),
    KeyUp(KeyboardEvent),
    WindowResize,
}

impl KeyboardEvent {
    pub fn from_raw(e: keyboard_event_t) -> Self {
        Self {
            timestamp: e.timestamp,
            scancode: e.scancode,
            key: e.key,
            modifier: e.mod_,
            text: e.text,
            down: e.down,
            repeat: e.repeat,
        }
    }
}

impl Event {
    pub fn from_raw(e: event_t) -> Option<Self> {
        fn key(e: event_t) -> KeyboardEvent {
            KeyboardEvent::from_raw(unsafe { e.__bindgen_anon_1.keyboard })
        }
        match e.type_ {
            event_type::EVENT_NONE => None,
            event_type::EVENT_QUIT => Some(Self::Quit),
            event_type::EVENT_KEY_UP => Some(Self::KeyUp(key(e))),
            event_type::EVENT_KEY_DOWN => Some(Self::KeyDown(key(e))),
            event_type::EVENT_WINDOW_RESIZE => Some(Self::WindowResize),
        }
    }
}

