//! Raw generated bindings for the functions exported by the WHY2025 badge.

#![cfg_attr(target_arch = "riscv32", no_std)]
#![allow(nonstandard_style)]
#![allow(non_camel_case_types)]
#![feature(c_variadic)]

pub mod bindings;
pub mod types;

pub use bindings::*;
pub use types::*;