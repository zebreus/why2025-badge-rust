//! Canonical raw generated bindings for the functions exported by the WHY2025 badge.
//!
//! This crate owns the generated ABI surface together with the regeneration workflow and input
//! snapshots used to produce it.

#![cfg_attr(target_arch = "riscv32", no_std)]
#![allow(nonstandard_style)]
#![allow(non_camel_case_types)]
#![feature(c_variadic)]

pub mod bindings;
pub mod types;

pub use bindings::*;
pub use types::*;
