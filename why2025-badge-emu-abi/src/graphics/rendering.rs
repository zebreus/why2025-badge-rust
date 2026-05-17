use alloc::vec::Vec;

use crate::graphics::WindowData;
use crate::types::*;

pub(crate) const fn convert_565_to_8888(color: u16) -> u32 {
    let r = ((color >> 11) & 0x1F) as u32 * 255 / 31;
    let g = ((color >> 5) & 0x3F) as u32 * 255 / 63;
    let b = (color & 0x1F) as u32 * 255 / 31;
    (r << 16) | (g << 8) | b
}

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
        unsafe {
            ::core::slice::from_raw_parts(self.pixels as *const u16, (self.w * self.h) as usize)
        }
    }

    fn slice_mut(&mut self) -> &mut [u16] {
        unsafe { ::core::slice::from_raw_parts_mut(self.pixels, (self.w * self.h) as usize) }
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

pub(crate) fn update_windowbuffer(window_data: &mut WindowData, size: Option<(usize, usize)>) {
    let (window_width, window_height) = size.unwrap_or(window_data.get_size());

    window_data
        .windowbuffer
        .0
        .resize(window_width * window_height, 0);
    window_data.windowbuffer.0.fill(0);
    window_data.windowbuffer.1 = (window_width, window_height);

    copy_framebuffer(&window_data.framebuffer, &mut window_data.windowbuffer);
    window_data.displayed_buffer565.0 = window_data.buffer565.clone();
    window_data.displayed_buffer565.1 = (
        window_data.framebuffer.w as usize,
        window_data.framebuffer.h as usize,
    );
    window_data.buffer565.fill(0);
}

pub(crate) fn rescale_windowbuffer(window_data: &mut WindowData, size: Option<(usize, usize)>) {
    let (windowbuffer_width, windowbuffer_height) = window_data.windowbuffer.1;
    let (current_width, current_height) = size.unwrap_or(window_data.get_size());
    if windowbuffer_width == current_width && windowbuffer_height == current_height {
        return;
    }

    window_data
        .windowbuffer
        .0
        .resize(current_width * current_height, 0);
    window_data.windowbuffer.0.fill(0);
    window_data.windowbuffer.1 = (current_width, current_height);

    copy_framebuffer(
        &window_data.displayed_buffer565,
        &mut window_data.windowbuffer,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_rgb565_to_8888() {
        assert_eq!(convert_565_to_8888(0b11111_000000_00000), 0x00ff0000);
        assert_eq!(convert_565_to_8888(0b00000_111111_00000), 0x0000ff00);
        assert_eq!(convert_565_to_8888(0b00000_000000_11111), 0x000000ff);
    }
}
