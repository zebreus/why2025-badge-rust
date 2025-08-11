#![no_std]
use core::ptr::null_mut;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::{geometry::Size, pixelcolor::Rgb888, prelude::*};
use why2025_badge_sys::{
    framebuffer_t, pixel_format_t, window_create, window_flag_t, window_framebuffer_create,
    window_handle_t, window_present, window_size_t,
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

impl Why2025BadgeWindow {
    /// Creates a new Window.
    // TODO: Add an option for flags
    pub fn new(size: Size, title: &str) -> Self {
        let size = window_size_t {
            w: size.width as i32,
            h: size.height as i32,
        };
        let title_bytes = title.as_bytes();
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
                window_flag_t::WINDOW_FLAG_DOUBLE_BUFFERED,
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
