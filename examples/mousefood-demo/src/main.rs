#![cfg_attr(target_arch = "riscv32", no_std)]
#![cfg_attr(target_arch = "riscv32", no_main)]
extern crate alloc;

use embedded_graphics::{mono_font::ascii::FONT_7X13, prelude::*};
use mousefood::{TerminalAlignment, prelude::*};
use ratatui::{Frame, Terminal, widgets::Paragraph};
use why2025_badge_embedded_graphics::Why2025BadgeWindow;

why2025_badge_app_no_std::app_main!(run);

fn run() -> i32 {
    why2025_badge_app_no_std::console::print_bytes(b"Hello, world! (from rust)\n\0");
    let mut display = Why2025BadgeWindow::new_floating(
        Size {
            width: 200,
            height: 200,
        },
        "Mousefood Demo",
    );

    why2025_badge_app_no_std::console::print_bytes(b"Frame drawn A\n\0");

    let config = EmbeddedBackendConfig {
        flush_callback: alloc::boxed::Box::new(|d: &mut Why2025BadgeWindow| {
            d.flush();
        }),
        font_regular: FONT_7X13,
        font_bold: None,
        font_italic: None,
        vertical_alignment: TerminalAlignment::Center,
        horizontal_alignment: TerminalAlignment::Center,
        ..Default::default()
    };
    why2025_badge_app_no_std::console::print_bytes(b"Frame drawn B\n\0");
    let backend = EmbeddedBackend::new(&mut display, config);

    why2025_badge_app_no_std::console::print_bytes(b"Frame drawn C\n\0");
    let mut terminal = Terminal::new(backend).unwrap();

    loop {
        why2025_badge_app_no_std::console::print_bytes(b"Frame drawn D\n\0");
        terminal.draw(draw).unwrap();
    }
}

/// Render the application. This is where you would draw the application UI. This example draws a
/// greeting.
fn draw(frame: &mut Frame) {
    let greeting = Paragraph::new("Hello World! (press 'q' to quit)");
    frame.render_widget(greeting, frame.area());
}
