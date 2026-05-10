//! Low-level BadgeVMS bindings plus host-side Emulation helpers.
//!
//! ## Example
//!
//! This crate also provides no_std badge-app-link metadata behind the `badge-app-link` feature.
//!
//! The symbols definitely need more documentation. If you want to add some, please add it to the
//! C code in the firmware repository so the regenerated raw bindings can pick it up.
//!
//! ## Interesting symbols
//!
//! * [printf]
//! * [window_create]
//! * [window_framebuffer_create]
//! * [window_present]

#![cfg_attr(target_arch = "riscv32", no_std)]
#![allow(nonstandard_style)]
#![allow(non_camel_case_types)]
#![feature(c_variadic)]
#![feature(linkage)]
#![cfg_attr(not(target_arch = "riscv32"), feature(thread_sleep_until))]

mod bindings {
    pub use why2025_badge_sys_bindings::bindings::*;
}

#[cfg(not(target_arch = "riscv32"))]
mod emulated;
#[cfg(not(target_arch = "riscv32"))]
mod linker_test;
mod types {
    pub use why2025_badge_sys_bindings::types::*;
}

pub use bindings::*;
pub use types::*;

#[cfg(not(target_arch = "riscv32"))]
pub use emulated::diprintf;
