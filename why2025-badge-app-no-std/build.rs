use std::env;

const BADGE_TARGET: &str = "riscv32imafc-unknown-none-elf";

fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    if env::var("TARGET").as_deref() != Ok(BADGE_TARGET) {
        return;
    }

    let retain_symbols_file = env::var("DEP_WHY2025_BADGE_SYS_RETAIN_SYMBOLS_FILE")
        .expect("why2025-badge-app-no-std expected why2025-badge-sys badge link metadata");
    let entry_symbol =
        env::var("DEP_WHY2025_BADGE_SYS_ENTRY_SYMBOL").unwrap_or_else(|_| "main".to_owned());

    println!("cargo::metadata=retain_symbols_file={retain_symbols_file}");
    println!("cargo::metadata=entry_symbol={entry_symbol}");
}
