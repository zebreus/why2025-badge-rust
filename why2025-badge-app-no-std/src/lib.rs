#![cfg_attr(target_arch = "riscv32", no_std)]

use core::ffi::CStr;

pub use why2025_badge_sys as sys;

const PRINT_AS_STRING: &[u8] = b"%s\0";

pub mod console {
    use super::{CStr, PRINT_AS_STRING};

    pub fn print(message: &CStr) {
        unsafe {
            crate::sys::printf(PRINT_AS_STRING.as_ptr().cast(), message.as_ptr());
        }
    }

    pub fn print_bytes(message: &[u8]) {
        let message = CStr::from_bytes_with_nul(message)
            .expect("why2025-badge-app-no-std::console::print_bytes expects a trailing NUL byte");
        print(message);
    }
}

#[cfg(not(target_arch = "riscv32"))]
const BADGE_TARGET: &str = "riscv32imafc-unknown-none-elf";

#[cfg(not(target_arch = "riscv32"))]
pub fn configure_build(build_script_path: &str) {
    use std::env;

    println!("cargo::rerun-if-changed={build_script_path}");

    if env::var("TARGET").as_deref() != Ok(BADGE_TARGET) {
        return;
    }

    let retain_symbols_file = env::var("DEP_WHY2025_BADGE_APP_NO_STD_RETAIN_SYMBOLS_FILE")
        .expect("why2025-badge-app-no-std expected facade badge link metadata");
    let entry_symbol =
        env::var("DEP_WHY2025_BADGE_APP_NO_STD_ENTRY_SYMBOL").unwrap_or_else(|_| "main".to_owned());

    println!("cargo::rustc-link-arg-bins=--shared");
    println!("cargo::rustc-link-arg-bins=--retain-symbols-file={retain_symbols_file}");
    println!("cargo::rustc-link-arg-bins=--gc-sections");
    println!("cargo::rustc-link-arg-bins=--strip-debug");
    println!("cargo::rustc-link-arg-bins=--discard-locals");
    println!("cargo::rustc-link-arg-bins=--entry={entry_symbol}");
}

#[macro_export]
macro_rules! app_main {
    ($run:path) => {
        #[cfg(target_arch = "riscv32")]
        #[unsafe(no_mangle)]
        pub extern "C" fn main() -> i32 {
            $run()
        }

        #[cfg(not(target_arch = "riscv32"))]
        fn main() {
            ::std::process::exit($run());
        }
    };
}
