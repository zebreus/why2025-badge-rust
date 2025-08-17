//! Helper functions for rendering graphics in the emulated badge environment.

use super::WindowData;
use crate::types::*;

const convert_565_to_8888: fn(u16) -> u32 = |color| {
    let r = ((color >> 11) & 0x1F) as u32 * 255 / 31;
    let g = ((color >> 5) & 0x3F) as u32 * 255 / 63;
    let b = (color & 0x1F) as u32 * 255 / 31;
    (r << 16) | (g << 8) | b
};

trait SomethingLikeABuffer<T> {
    fn slice(&self) -> &[T];
    fn slice_mut(&mut self) -> &mut [T];
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn len(&self) -> usize {
        self.slice().len()
    }
}
impl SomethingLikeABuffer<u16> for (Vec<u16>, (usize, usize)) {
    fn slice(&self) -> &[u16] {
        &self.0
    }
    fn slice_mut(&mut self) -> &mut [u16] {
        &mut self.0
    }
    fn width(&self) -> usize {
        self.1.0
    }
    fn height(&self) -> usize {
        self.1.1
    }
}
impl SomethingLikeABuffer<u16> for framebuffer_t {
    fn slice(&self) -> &[u16] {
        unsafe { std::slice::from_raw_parts(self.pixels as *const u16, (self.w * self.h) as usize) }
    }
    fn slice_mut(&mut self) -> &mut [u16] {
        unsafe {
            std::slice::from_raw_parts_mut(self.pixels as *mut u16, (self.w * self.h) as usize)
        }
    }
    fn width(&self) -> usize {
        self.w as usize
    }
    fn height(&self) -> usize {
        self.h as usize
    }
}
impl SomethingLikeABuffer<u32> for (Vec<u32>, (usize, usize)) {
    fn slice(&self) -> &[u32] {
        &self.0
    }
    fn slice_mut(&mut self) -> &mut [u32] {
        &mut self.0
    }
    fn width(&self) -> usize {
        self.1.0
    }
    fn height(&self) -> usize {
        self.1.1
    }
}

// Copy the framebuffer data into the window buffer
fn copy_framebuffer(
    framebuffer: &impl SomethingLikeABuffer<u16>,
    windowbuffer: &mut impl SomethingLikeABuffer<u32>,
) {
    assert_eq!(
        framebuffer.len(),
        framebuffer.width() * framebuffer.height(),
        "Framebuffer size does not match dimensions"
    );
    assert_eq!(
        windowbuffer.len(),
        windowbuffer.width() * windowbuffer.height(),
        "Window buffer size does not match dimensions"
    );
    if framebuffer.width() == windowbuffer.width() && framebuffer.height() == windowbuffer.height()
    {
        // If the framebuffer size matches the window size, we can just copy the buffer
        // directly to the framebuffer slice.
        for (pixel_565, pixel_8888) in framebuffer.slice().iter().zip(windowbuffer.slice_mut()) {
            *pixel_8888 = convert_565_to_8888(*pixel_565);
        }
    } else {
        for y in 0..windowbuffer.height().min(framebuffer.height()) {
            for x in 0..windowbuffer.width().min(framebuffer.width()) {
                let framebuffer_index = y * framebuffer.width() + x;
                let window_index = y * windowbuffer.width() + x;
                windowbuffer.slice_mut()[window_index] =
                    convert_565_to_8888(framebuffer.slice()[framebuffer_index]);
            }
        }
    }
}

/// Update the window buffer with the current framebuffer data.
pub fn update_windowbuffer(window_data: &mut WindowData, size: Option<(usize, usize)>) -> () {
    let WindowData {
        window,
        windowbuffer,
        buffer565,
        displayed_buffer565,
        framebuffer,
        ..
    } = window_data;

    let (window_width, window_height) = size.unwrap_or(window.get_size());

    // Grow the window buffer so we never segfault
    windowbuffer.0.resize(window_width * window_height, 0);
    windowbuffer.0.fill(0);
    windowbuffer.1 = (window_width, window_height);

    copy_framebuffer(framebuffer, windowbuffer);
    displayed_buffer565.0 = buffer565.clone();
    displayed_buffer565.1 = (framebuffer.w as usize, framebuffer.h as usize);
    buffer565.fill(0);
}

/// Regenerate the window buffer if it's size does not match the current window size.
pub fn rescale_windowbuffer(window_data: &mut WindowData, size: Option<(usize, usize)>) -> () {
    let WindowData {
        window,
        windowbuffer,
        displayed_buffer565,
        ..
    } = window_data;
    let (windowbuffer_width, windowbuffer_height) = windowbuffer.1;
    let (current_width, current_height) = size.unwrap_or(window.get_size());
    if windowbuffer_width == current_width && windowbuffer_height == current_height {
        // If the size didn't change, we don't need to rescale
        return;
    }

    // Grow the window buffer so we never segfault
    windowbuffer.0.resize(current_width * current_height, 0);
    windowbuffer.0.fill(0);
    windowbuffer.1 = (current_width, current_height);

    copy_framebuffer(displayed_buffer565, windowbuffer);
}

/// Update the window buffer with the current framebuffer data.
///
/// If the update with the buffer fails, it will print an error message and try again without rendering a buffer.
pub fn present_windowbuffer(window_data: &mut WindowData, size: Option<(usize, usize)>) -> () {
    let WindowData {
        window,
        windowbuffer: buffer,
        ..
    } = window_data;

    let (window_width, window_height) = size.unwrap_or(window.get_size());

    if buffer.1.0 != window_width || buffer.1.1 != window_height {
        eprintln!("Window buffer size does not match dimensions");
        // Maybe do rescale & recursion here instead?
        window.update();
        return;
    }

    match window.update_with_buffer(&buffer.0, window_width as usize, window_height as usize) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Failed to update window buffer: {:?}", e);
            // Make sure we still update the window, so it doesn't freeze
            window.update();
        }
    };
}
