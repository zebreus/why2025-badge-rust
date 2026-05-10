#![allow(unexpected_cfgs)]

fn main() {
    println!("BadgeVMS std unsupported-behavior smoke");
    check_current_dir();
    check_udp();
    check_process_stdio_redirection();
}

#[cfg(target_os = "badgevms")]
fn check_current_dir() {
    use std::env;

    match env::current_dir() {
        Ok(path) => println!("unexpected current_dir success: {}", path.display()),
        Err(error) => println!("current_dir unsupported as expected: {error}"),
    }
}

#[cfg(not(target_os = "badgevms"))]
fn check_current_dir() {
    println!("host current_dir check skipped; BadgeVMS should return unsupported");
}

#[cfg(target_os = "badgevms")]
fn check_udp() {
    use std::net::UdpSocket;

    match UdpSocket::bind("0.0.0.0:0") {
        Ok(_) => println!("unexpected UDP success"),
        Err(error) => println!("UDP unsupported as expected: {error}"),
    }
}

#[cfg(not(target_os = "badgevms"))]
fn check_udp() {
    println!("host UDP check skipped; BadgeVMS should return unsupported");
}

#[cfg(target_os = "badgevms")]
fn check_process_stdio_redirection() {
    use std::process::{Command, Stdio};

    match Command::new("APP:std-hello-world")
        .stdout(Stdio::null())
        .status()
    {
        Ok(status) => println!("unexpected redirected process success: {status}"),
        Err(error) => println!("process stdio redirection unsupported as expected: {error}"),
    }
}

#[cfg(not(target_os = "badgevms"))]
fn check_process_stdio_redirection() {
    println!("host process-redirection check skipped; BadgeVMS should return unsupported");
}
