use core::marker::PhantomData;
use std::{
    error::Error,
    ptr::{null, null_mut},
};

use embedded_graphics::{
    framebuffer,
    geometry::Size,
    pixelcolor::{PixelColor, Rgb888},
    prelude::*,
    primitives::Rectangle,
};

#[repr(C)]
pub enum PixelFormat {
    UNKNOWN = 0,
    INDEX1LSB = 0x11100100,
    INDEX1MSB = 0x11200100,
    INDEX2LSB = 0x1c100200,
    INDEX2MSB = 0x1c200200,
    INDEX4LSB = 0x12100400,
    INDEX4MSB = 0x12200400,
    INDEX8 = 0x13000801,
    RGB332 = 0x14110801,
    XRGB4444 = 0x15120c02,
    XBGR4444 = 0x15520c02,
    XRGB1555 = 0x15130f02,
    XBGR1555 = 0x15530f02,
    ARGB4444 = 0x15321002,
    RGBA4444 = 0x15421002,
    ABGR4444 = 0x15721002,
    BGRA4444 = 0x15821002,
    ARGB1555 = 0x15331002,
    RGBA5551 = 0x15441002,
    ABGR1555 = 0x15731002,
    BGRA5551 = 0x15841002,
    RGB565 = 0x15151002,
    BGR565 = 0x15551002,
    RGB24 = 0x17101803,
    BGR24 = 0x17401803,
    XRGB8888 = 0x16161804,
    RGBX8888 = 0x16261804,
    XBGR8888 = 0x16561804,
    BGRX8888 = 0x16661804,
    ARGB8888 = 0x16362004,
    RGBA8888 = 0x16462004,
    ABGR8888 = 0x16762004,
    BGRA8888 = 0x16862004,
    XRGB2101010 = 0x16172004,
    XBGR2101010 = 0x16572004,
    ARGB2101010 = 0x16372004,
    ABGR2101010 = 0x16772004,
    RGB48 = 0x18103006,
    BGR48 = 0x18403006,
    RGBA64 = 0x18204008,
    ARGB64 = 0x18304008,
    BGRA64 = 0x18504008,
    ABGR64 = 0x18604008,
    RGB48_FLOAT = 0x1a103006,
    BGR48_FLOAT = 0x1a403006,
    RGBA64_FLOAT = 0x1a204008,
    ARGB64_FLOAT = 0x1a304008,
    BGRA64_FLOAT = 0x1a504008,
    ABGR64_FLOAT = 0x1a604008,
    RGB96_FLOAT = 0x1b10600c,
    BGR96_FLOAT = 0x1b40600c,
    RGBA128_FLOAT = 0x1b208010,
    ARGB128_FLOAT = 0x1b308010,
    BGRA128_FLOAT = 0x1b508010,
    ABGR128_FLOAT = 0x1b608010,

    YV12 = 0x32315659,
    /**< Planar mode: Y + V + U  (3 planes) */
    IYUV = 0x56555949,
    /**< Planar mode: Y + U + V  (3 planes) */
    YUY2 = 0x32595559,
    /**< Packed mode: Y0+U0+Y1+V0 (1 plane) */
    UYVY = 0x59565955,
    /**< Packed mode: U0+Y0+V0+Y1 (1 plane) */
    YVYU = 0x55595659,
    /**< Packed mode: Y0+V0+Y1+U0 (1 plane) */
    NV12 = 0x3231564e,
    /**< Planar mode: Y + U/V interleaved  (2 planes) */
    NV21 = 0x3132564e,
    /**< Planar mode: Y + V/U interleaved  (2 planes) */
    P010 = 0x30313050,
    /**< Planar mode: Y + U/V interleaved  (2 planes) */
    EXTERNAL_OES = 0x2053454f,
    /**< Android video texture format */
    MJPG = 0x47504a4d,
}

pub const TOP_BAR_PX: usize = 50;
pub const SIDE_BAR_PX: usize = 0;
pub const MAX_VISIBLE_RECTS: usize = 64;

#[repr(C)]
pub struct RectArray {
    rects: [WindowRect; MAX_VISIBLE_RECTS],
    count: u32,
}

#[repr(C)]
pub struct SmallRectArray {
    rects: [WindowRect; 4],
    count: u32,
}

#[repr(C)]
pub enum WindowFlag {
    NONE = 0,
    FULLSCREEN = (1 << 0), // Only one fullscreen application can run at a tim
    ALWAYS_ON_TOP = (1 << 1), // Does not apply to fullscreen apps
    UNDECORATED = (1 << 2), // Create a floating window
    MAXIMIZED = (1 << 3),  // Create an application window of the maximum size
    MAXIMIZED_LEFT = (1 << 4), // Create a window and have it cover the whole left of the screen
    MAXIMIZED_RIGHT = (1 << 5), // Create a window and have it cover the whole right of the screen
    DOUBLE_BUFFERED = (1 << 6), // Create a double buffered window
}

#[repr(C)]
pub struct WindowCoords {
    x: u32,
    y: u32,
}

#[derive(Clone)]
#[repr(C)]
pub struct WindowSize {
    w: u32,
    h: u32,
}

#[repr(C)]
pub struct WindowRect {
    coords: WindowCoords,
    size: WindowSize,
}
#[repr(C)]
pub struct ManagedFramebuffer {
    framebuffer: Framebuffer,
    w: u32,
    h: u32,
    format: PixelFormat,
    head_pages: *mut u32, // Actually a pointer to an array of allocation_range_t
    tail_pages: *mut u32, // Actually a pointer to an array of allocation_range
    num_pages: usize,
    clean: bool,
}

#[repr(C)]
pub struct Window {
    framebuffer_a: *mut ManagedFramebuffer,
    framebuffer_b: *mut ManagedFramebuffer,
    front_fb: u8,
    back_fb: u8,
    flags: WindowFlag,
    title: *mut u8, // Actually a pointer to a C string
    fb_dirty: u32,
    rect: WindowRect,
    rect_orig: WindowRect,
    visible: RectArray,
    task_info: *mut u32,  // atomic_uintptr_t
    event_queue: *mut u8, // Actually a pointer to an event queue
    next: *mut Window,
    prev: *mut Window,
}

unsafe extern "C" {
    pub fn window_create(title: *const u8, size: WindowSize, flags: WindowFlag) -> *mut Window;
    pub fn window_framebuffer_create(
        window: *mut Window,
        size: WindowSize,
        pixel_format: PixelFormat,
    ) -> *mut Framebuffer;
    pub fn window_present(
        window: *mut Window,
        block: bool,
        rects: *mut WindowRect,
        num_rects: u32,
    ) -> ();
}

#[repr(C)]
pub struct Framebuffer {
    w: u32,
    h: u32,
    format: PixelFormat,
    pixels: *mut u8, // The C implementation uses a *mut u16, but I am not sure why
}

// const fn rgb888_to_rgb565(r: u8, g: u8, b: u8) -> u16 {
//     // Convert RGB888 to RGB565 format
//     ((r & 0xF8) << 8) | ((g as u8 & 0xFC) << 3) | (b >> 3)
// }

pub struct Why2025BadgeDisplay {
    size: (u32, u32),
    window: *mut Window,
    framebuffer: *mut Framebuffer,
}

impl Why2025BadgeDisplay {
    /// Creates a new display.
    ///
    /// This appends a `<canvas>` element with size corresponding to scale and pixel spacing used
    /// The display is filled with black.
    pub fn new(size: (u32, u32)) -> Self {
        let size = WindowSize {
            w: size.0,
            h: size.1,
        };
        // source: https://github.com/embedded-graphics/simulator/blob/master/src/output_settings.rs

        let window = unsafe {
            window_create(
                "Hello".as_ptr() as *const u8,
                size.clone(),
                WindowFlag::NONE,
            )
        };

        let framebuffer =
            unsafe { window_framebuffer_create(window, size.clone(), PixelFormat::RGB24) };

        // source: https://github.com/embedded-graphics/simulator/blob/master/src/output_settings.rs#L39

        Why2025BadgeDisplay {
            size: (size.w, size.h),
            window,
            framebuffer,
        }
    }

    pub fn flush(&mut self) {
        unsafe { window_present(self.window, true, null_mut(), 0) };
    }
}
impl OriginDimensions for Why2025BadgeDisplay {
    fn size(&self) -> Size {
        Size {
            width: self.size.0 as u32,
            height: self.size.1 as u32,
        }
    }
}

impl DrawTarget for Why2025BadgeDisplay {
    type Color = Rgb888;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), ()>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let canvas_width = self.size.0 as usize;
        let framebuffer = unsafe { &mut (*self.window) };
        let backing = unsafe {
            core::slice::from_raw_parts_mut(
                (*self.framebuffer).pixels,
                self.size.0 as usize * self.size.1 as usize * 3,
            )
        };

        let scale = 1;
        // source: https://github.com/embedded-graphics/simulator/blob/master/src/output_settings.rs#L39
        let pitch = scale as usize;

        let bounding_box = Rectangle::new(Point::new(0, 0), self.size());
        for pixel in pixels.into_iter() {
            let point = pixel.0;
            if bounding_box.contains(point) {
                let rgb: Rgb888 = pixel.1.into();
                let rgb_slice = &[rgb.r(), rgb.g(), rgb.b()];
                let py = point.y as usize;
                let px = point.x as usize;

                let x_offset = px * 4 * pitch;
                for y in 0..scale {
                    let y_offset = py * 4 * canvas_width * pitch + y * 4 * canvas_width;
                    for x in 0..scale {
                        let pixel_offset = y_offset + x_offset + x * 3;
                        backing[pixel_offset..pixel_offset + 3].copy_from_slice(rgb_slice);
                    }
                }
            }
        }

        Ok(())
    }
}
