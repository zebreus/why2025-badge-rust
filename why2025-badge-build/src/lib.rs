use std::{env, fs, path::PathBuf};

const BADGE_TARGET: &str = "riscv32imafc-unknown-none-elf";

fn badge_link_metadata(var: &str) -> Option<String> {
    env::var(format!("DEP_WHY2025_BADGE_APP_NO_STD_{var}"))
        .ok()
        .or_else(|| env::var(format!("DEP_WHY2025_BADGE_SYS_{var}")).ok())
}

fn generated_retain_symbols_file(entry_symbol: &str) -> PathBuf {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR must be set by Cargo"));
    let retain_symbols_file = out_dir.join("retain.txt");
    let retain_symbols = badge_link_metadata("RETAIN_SYMBOLS_FILE")
        .map(|path| {
            fs::read_to_string(&path).unwrap_or_else(|error| {
                panic!("failed to read retain symbols file {path}: {error}")
            })
        })
        .unwrap_or_else(|| format!("{entry_symbol}\n"));

    fs::write(&retain_symbols_file, retain_symbols).unwrap_or_else(|error| {
        panic!(
            "failed to write retain symbols file {}: {error}",
            retain_symbols_file.display()
        )
    });

    retain_symbols_file
}

pub fn configure() {
    println!("cargo::rerun-if-changed=build.rs");

    if env::var("TARGET").as_deref() != Ok(BADGE_TARGET) {
        return;
    }

    let entry_symbol = badge_link_metadata("ENTRY_SYMBOL").unwrap_or_else(|| "main".to_owned());
    let retain_symbols_file = generated_retain_symbols_file(&entry_symbol);

    println!("cargo::rustc-link-arg-bins=--shared");
    println!(
        "cargo::rustc-link-arg-bins=--retain-symbols-file={}",
        retain_symbols_file.display()
    );
    println!("cargo::rustc-link-arg-bins=--gc-sections");
    println!("cargo::rustc-link-arg-bins=--strip-debug");
    println!("cargo::rustc-link-arg-bins=--discard-locals");
    println!("cargo::rustc-link-arg-bins=--entry={entry_symbol}");
}
