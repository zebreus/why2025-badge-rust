#![allow(unexpected_cfgs)]

use std::env;

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
