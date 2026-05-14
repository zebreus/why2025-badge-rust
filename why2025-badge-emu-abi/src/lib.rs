//! Experimental no_std + libc BadgeVMS ABI export crate.
//!
//! This crate is Linux/ELF-host-only for now. It exists to produce ABI artifacts
//! (`staticlib` and `cdylib`) and deliberately does not replace the existing
//! `why2025-badge-sys` host emulator.

#![no_std]
#![allow(non_camel_case_types)]
#![allow(nonstandard_style)]
#![feature(c_variadic)]

extern crate alloc;

#[cfg(test)]
extern crate std;

use core::ffi::{c_char, c_int};

pub mod deferred;
pub mod libc_compat;
pub mod runtime;

mod allocator;

pub mod types {
    pub use why2025_badge_sys_bindings::types::*;
}

pub use types::*;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    runtime::abort_with_message(b"why2025-badge-emu-abi panic\n")
}

/// Return the crate-local BadgeVMS errno slot.
///
/// BadgeVMS exports `__errno`, not glibc's `__errno_location`. This first
/// implementation is process-global; later runtime phases can replace the
/// storage with host-thread-local state without changing the exported ABI.
#[unsafe(no_mangle)]
pub extern "C" fn __errno() -> *mut c_int {
    runtime::__errno()
}

/// Normalized ctype export from ADR 0002.
#[unsafe(no_mangle)]
pub static _ctype_b: [c_char; 0] = [];

/// Compatibility alias for the firmware manifest's historical `_ctype_` name.
#[unsafe(export_name = "_ctype_")]
pub static CTYPE_ALIAS: [c_char; 0] = [];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errno_slot_is_stable() {
        let first = __errno();
        let second = __errno();

        assert!(!first.is_null());
        assert_eq!(first, second);
    }

    #[test]
    fn ctype_symbols_have_addresses() {
        assert_ne!(
            core::ptr::addr_of!(_ctype_b) as *const (),
            core::ptr::null()
        );
        assert_ne!(
            core::ptr::addr_of!(CTYPE_ALIAS) as *const (),
            core::ptr::null()
        );
    }
}
