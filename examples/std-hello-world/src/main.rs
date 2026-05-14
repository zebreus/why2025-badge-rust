#![allow(unexpected_cfgs)]

use std::env;

#[cfg(target_os = "badgevms")]
#[allow(dead_code)]
fn compile_check_badgevms_std_surface() {
    use std::io::IsTerminal;
    use std::net::ToSocketAddrs;

    let _ = std::io::stdin().is_terminal();
    let _tcp_connect = || std::net::TcpStream::connect("127.0.0.1:80");
    let _tcp_bind = || std::net::TcpListener::bind("127.0.0.1:0");
    let _lookup = || "localhost:80".to_socket_addrs();
}

fn main() {
    println!("Hello from a Rust std App for BadgeVMS");
    println!("args:");
    for (index, argument) in env::args().enumerate() {
        println!("  {index}: {argument}");
    }

    #[cfg(target_os = "badgevms")]
    println!("target_os=badgevms target_family=unix");

    #[cfg(not(target_os = "badgevms"))]
    println!("host smoke build; run with the BadgeVMS std target for on-device behavior");
}
