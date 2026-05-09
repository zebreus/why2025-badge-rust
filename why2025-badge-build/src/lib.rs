use std::env;

const BADGE_TARGET: &str = "riscv32imafc-unknown-none-elf";

pub fn configure(build_script_path: &str) {
    println!("cargo::rerun-if-changed={build_script_path}");

    if env::var("TARGET").as_deref() != Ok(BADGE_TARGET) {
        return;
    }

    let retain_symbols_file = match env::var("DEP_WHY2025_BADGE_SYS_RETAIN_SYMBOLS_FILE") {
        Ok(path) => path,
        Err(_) => return,
    };

    let entry_symbol =
        env::var("DEP_WHY2025_BADGE_SYS_ENTRY_SYMBOL").unwrap_or_else(|_| "main".to_owned());

    println!("cargo::rustc-link-arg-bins=--shared");
    println!("cargo::rustc-link-arg-bins=--retain-symbols-file={retain_symbols_file}");
    println!("cargo::rustc-link-arg-bins=--gc-sections");
    println!("cargo::rustc-link-arg-bins=--strip-debug");
    println!("cargo::rustc-link-arg-bins=--discard-locals");
    println!("cargo::rustc-link-arg-bins=--entry={entry_symbol}");
}