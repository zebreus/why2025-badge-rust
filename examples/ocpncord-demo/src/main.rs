use std::{boxed::Box, process::ExitCode, string::String, sync::LazyLock};

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

const OPENCODE_BASE_URL: &str = "http://192.168.178.20:4096";

static BADGE_DNS: LazyLock<BadgeDns> = LazyLock::new(BadgeDns::default);
static BADGE_TCP_CONNECT: LazyLock<BadgeTcpConnect> = LazyLock::new(BadgeTcpConnect::default);

fn main() -> ExitCode {
    ExitCode::from(run() as u8)
}

fn run() -> i32 {
    let base_url = base_url();
    log_startup_network_info(base_url);
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

fn base_url() -> &'static str {
    OPENCODE_BASE_URL
}

#[cfg(not(target_arch = "riscv32"))]
fn log_startup_network_info(base_url: &str) {
    println!("ocpncord-demo base_url={base_url}");
    match best_effort_local_ip(base_url) {
        Some(ip) => println!("ocpncord-demo local_ip={ip}"),
        None => println!("ocpncord-demo local_ip=unavailable"),
    }
}

#[cfg(not(target_arch = "riscv32"))]
fn best_effort_local_ip(base_url: &str) -> Option<std::net::IpAddr> {
    let endpoint = base_url
        .strip_prefix("http://")
        .or_else(|| base_url.strip_prefix("https://"))?;
    let authority = endpoint.split('/').next()?;
    let (host, port) = authority.rsplit_once(':')?;
    let port = port.parse::<u16>().ok()?;

    let socket = std::net::UdpSocket::bind(("0.0.0.0", 0)).ok()?;
    socket.connect((host, port)).ok()?;
    socket.local_addr().ok().map(|addr| addr.ip())
}

#[cfg(target_arch = "riscv32")]
fn log_startup_network_info(base_url: &str) {
    use std::ffi::CStr;

    use why2025_badge_sys_bindings as abi;

    println!("ocpncord-demo base_url={base_url}");

    let status = unsafe { abi::wifi_get_connection_status() };
    println!("ocpncord-demo wifi_status={status:?}");

    if status == abi::wifi_connection_status_t::WIFI_CONNECTED {
        let station = unsafe { abi::wifi_get_connection_station() };
        if station.is_null() {
            println!("ocpncord-demo wifi_station=null");
        } else {
            let ssid_ptr = unsafe { abi::wifi_station_get_ssid(station) };
            if ssid_ptr.is_null() {
                println!("ocpncord-demo wifi_ssid=<null>");
            } else {
                let ssid = unsafe { CStr::from_ptr(ssid_ptr) }.to_string_lossy();
                println!("ocpncord-demo wifi_ssid={ssid}");
            }

            let rssi = unsafe { abi::wifi_station_get_rssi(station) };
            println!("ocpncord-demo wifi_rssi={rssi}");
        }
    }

    println!("ocpncord-demo local_ip=unavailable current BadgeVMS ABI does not expose station IP");
}

fn current_working_directory() -> String {
    std::env::current_dir()
        .ok()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_default()
}
