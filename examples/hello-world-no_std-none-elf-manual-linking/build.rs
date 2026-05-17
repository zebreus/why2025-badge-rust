use std::{env, path::PathBuf};

const BADGE_TARGET: &str = "riscv32imafc-unknown-none-elf";

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=retain.txt");

    if env::var("TARGET").as_deref() != Ok(BADGE_TARGET) {
        return;
    }

    let retain_symbols_file = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by Cargo"),
    )
    .join("retain.txt");

    println!("cargo::rustc-link-arg-bins=--shared");
    println!(
        "cargo::rustc-link-arg-bins=--retain-symbols-file={}",
        retain_symbols_file.display()
    );
    println!("cargo::rustc-link-arg-bins=--gc-sections");
    println!("cargo::rustc-link-arg-bins=--strip-debug");
    println!("cargo::rustc-link-arg-bins=--discard-locals");
    println!("cargo::rustc-link-arg-bins=--entry=main");
}
