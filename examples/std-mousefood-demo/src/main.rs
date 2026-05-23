use embedded_graphics::{mono_font::ascii::FONT_7X13, prelude::*};
use mousefood::{TerminalAlignment, prelude::*};
use ratatui::{Frame, Terminal, widgets::Paragraph};
use why2025_badge_embedded_graphics::Why2025BadgeWindow;

#[cfg(not(target_arch = "riscv32"))]
use why2025_badge_emu_abi as _;

fn main() {
    let mut display = Why2025BadgeWindow::new_floating(
        Size {
            width: 200,
            height: 200,
        },
        "Std Mousefood Demo",
    );

    println!("aaaa");
    eprintln!("aaaa err");
    let config = EmbeddedBackendConfig {
        flush_callback: Box::new(|display: &mut Why2025BadgeWindow| {
            display.flush();
        }),
        font_regular: FONT_7X13,
        font_bold: None,
        font_italic: None,
        vertical_alignment: TerminalAlignment::Center,
        horizontal_alignment: TerminalAlignment::Center,
        ..Default::default()
    };
    println!("bbbb");
    eprintln!("bbbb err");
    let backend = EmbeddedBackend::new(&mut display, config);
    println!("cccc");
    eprintln!("cccc err");
    let mut terminal = Terminal::new(backend).unwrap();

    loop {
        println!("before draw out");
        eprintln!("before draw err");
        terminal.draw(draw).unwrap();
        eprintln!("after draw");
    }
}

fn draw(frame: &mut Frame) {
    let greeting = Paragraph::new("Hello World! (std mousefood demo)");
    frame.render_widget(greeting, frame.area());
}
