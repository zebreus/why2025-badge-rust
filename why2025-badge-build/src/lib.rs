use std::env;

const BADGE_TARGET: &str = "riscv32imafc-unknown-none-elf";

fn badge_link_metadata(var: &str) -> Option<String> {
    env::var(format!("DEP_WHY2025_BADGE_APP_NO_STD_{var}")).ok().or_else(|| {
        env::var(format!("DEP_WHY2025_BADGE_SYS_{var}")).ok()
    })
}

pub fn configure() {
    println!("cargo::rerun-if-changed=build.rs");

    if env::var("TARGET").as_deref() != Ok(BADGE_TARGET) {
        return;
    }

    let retain_symbols_file = match badge_link_metadata("RETAIN_SYMBOLS_FILE") {
        Some(path) => path,
        None => return,
    };

    let entry_symbol = badge_link_metadata("ENTRY_SYMBOL").unwrap_or_else(|| "main".to_owned());

    println!("cargo::rustc-link-arg-bins=--shared");
    println!("cargo::rustc-link-arg-bins=--retain-symbols-file={retain_symbols_file}");
    println!("cargo::rustc-link-arg-bins=--gc-sections");
    println!("cargo::rustc-link-arg-bins=--strip-debug");
    println!("cargo::rustc-link-arg-bins=--discard-locals");
    println!("cargo::rustc-link-arg-bins=--entry={entry_symbol}");
}