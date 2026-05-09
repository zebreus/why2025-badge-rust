use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo::rerun-if-changed=src/build_script.rs");

    let target = env::var("TARGET").unwrap();
    let badge_app_link = env::var_os("CARGO_FEATURE_BADGE_APP_LINK").is_some();

    if target != "riscv32imafc-unknown-none-elf" || !badge_app_link {
        return;
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let retain_symbols_file = out_dir.join("why2025-retain.txt");

    fs::write(&retain_symbols_file, "main\n").unwrap();

    println!(
        "cargo::metadata=retain_symbols_file={}",
        retain_symbols_file.display()
    );
    println!("cargo::metadata=entry_symbol=main");
}