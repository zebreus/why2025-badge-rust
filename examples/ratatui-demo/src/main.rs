#![no_std]
#![no_main]
#![allow(internal_features)]
#![feature(core_intrinsics)]
extern crate alloc;

use crate::app::App;
use mousefood::{TerminalAlignment, prelude::*};
use ratatui::Terminal;
use why2025_badge_embedded_graphics::Why2025BadgeWindow;

mod app;
mod ui;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    let mut display = Why2025BadgeWindow::new_fullscreen();

    let config = EmbeddedBackendConfig {
        flush_callback: alloc::boxed::Box::new(|d: &mut Why2025BadgeWindow| {
            d.flush();
        }),
        font_regular: embedded_graphics_unicodefonts::MONO_9X15,
        font_bold: Some(embedded_graphics_unicodefonts::MONO_9X15_BOLD),
        font_italic: None,
        vertical_alignment: TerminalAlignment::Center,
        horizontal_alignment: TerminalAlignment::Center,
    };
    let backend = EmbeddedBackend::new(&mut display, config);

    let mut terminal = Terminal::new(backend).unwrap();

    let mut app = App::new("Crossterm Demo", false);

    loop {
        terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();
        app.on_tick();
    }
}

// Allocator and panic handler setup
use talc::{ClaimOnOom, Span, Talc, Talck};

const HEAP_SIZE: usize = 1024 * 300; // 300KB heap size
static mut HEAP: [u8; HEAP_SIZE] = [0u8; HEAP_SIZE];
#[global_allocator]
static ALLOCATOR: Talck<spin::Mutex<()>, ClaimOnOom> =
    Talc::new(unsafe { ClaimOnOom::new(Span::from_array((&raw const HEAP).cast_mut())) }).lock();

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        let maybe_msg = alloc::string::ToString::to_string(&panic_info.message());
        let msg = maybe_msg.as_ptr();
        why2025_badge_sys::printf(b"panic: %s\n\0".as_ptr(), msg);
        if let Some(location) = panic_info.location() {
            why2025_badge_sys::printf(
                b"in %s:%d\n\0".as_ptr(),
                location.file().as_ptr(),
                location.line() as i32,
            );
        } else {
            why2025_badge_sys::printf(b"no location information available :(\n\0".as_ptr());
        }
    }
    loop {}
}
