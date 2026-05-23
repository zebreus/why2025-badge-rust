use std::{
    boxed::Box,
    process::ExitCode,
    string::{String, ToString},
    sync::LazyLock,
};

use futures::executor::block_on;
use mousefood::{TerminalAlignment, prelude::*};
use ocpncord_backend_opencode::OpenCodeBackend;
use ocpncord_tui::App;
use ratatui::Terminal;
use why2025_badge_embedded_graphics::{
    Why2025BadgeWindow, Why2025BadgeWindowConfig, WindowBuffering,
};
use why2025_badge_embedded_nal_async::{BadgeDns, BadgeTcpConnect};

#[cfg(not(target_arch = "riscv32"))]
use why2025_badge_emu_abi as _;

mod input;

static BADGE_DNS: LazyLock<BadgeDns> = LazyLock::new(BadgeDns::default);
static BADGE_TCP_CONNECT: LazyLock<BadgeTcpConnect> = LazyLock::new(BadgeTcpConnect::default);

fn main() -> ExitCode {
    ExitCode::from(run() as u8)
}

fn run() -> i32 {
    let base_url = base_url();
    let mut display = Why2025BadgeWindow::new(
        Why2025BadgeWindowConfig::new_fullscreen().buffering(WindowBuffering::SingleBuffered),
    );
    let window = unsafe { display.raw_handle() };

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
    let terminal = Terminal::new(backend).unwrap();
    let events = input::BadgeEventStream::new(window, 50);
    let backend = OpenCodeBackend::new(&base_url, &*BADGE_TCP_CONNECT, &*BADGE_DNS);
    let mut app = App::new(backend, events, terminal);

    app.set_cwd(current_working_directory());
    block_on(app.run());

    0
}

fn base_url() -> String {
    let mut args = std::env::args().skip(1);
    let mut url = "http://192.168.178.20:4096".to_string();

    while let Some(argument) = args.next() {
        if argument == "--url" {
            if let Some(value) = args.next() {
                url = value;
            }
            continue;
        }

        if let Some(value) = argument.strip_prefix("--url=") {
            url = value.to_string();
        }
    }

    url
}

fn current_working_directory() -> String {
    std::env::current_dir()
        .ok()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_default()
}
