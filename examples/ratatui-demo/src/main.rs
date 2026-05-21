#![cfg_attr(target_arch = "riscv32", no_std)]
#![cfg_attr(target_arch = "riscv32", no_main)]
#![allow(internal_features)]
#![feature(core_intrinsics)]
extern crate alloc;

use crate::app::App;
use mousefood::{TerminalAlignment, prelude::*};
use ratatui::Terminal;
use why2025_badge_embedded_graphics::Why2025BadgeWindow;

mod app;
mod ui;

why2025_badge_app_no_std::app_main!(run);

fn run() -> i32 {
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
        ..Default::default()
    };
    let backend = EmbeddedBackend::new(&mut display, config);

    let mut terminal = Terminal::new(backend).unwrap();

    let mut app = App::new("Crossterm Demo", false);

    loop {
        terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();
        app.on_tick();
    }
}
