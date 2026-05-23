use crate::app::App;
use mousefood::{TerminalAlignment, prelude::*};
use ratatui::Terminal;
use why2025_badge_embedded_graphics::{
    Why2025BadgeWindow, Why2025BadgeWindowConfig, WindowBuffering,
};

#[cfg(not(target_arch = "riscv32"))]
use why2025_badge_emu_abi as _;

mod app;
mod ui;

fn main() {
    let mut display = Why2025BadgeWindow::new(
        Why2025BadgeWindowConfig::new_fullscreen()
            .buffering(WindowBuffering::SingleBuffered),
    );

    let config = EmbeddedBackendConfig {
        flush_callback: Box::new(|display: &mut Why2025BadgeWindow| {
            display.flush();
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
    let mut app = App::new("Std RataTUI Demo", false);

    loop {
        terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();
        app.on_tick();
    }
}
