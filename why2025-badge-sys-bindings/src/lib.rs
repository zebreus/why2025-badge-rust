//! Generated Rust bindings for the functions exported by the WHY2025 badge.
//!
//! The regeneration workflow and input snapshots for this crate live in this directory.
//! The repository's raw-ABI boundary is recorded in
//! `docs/adr/0004-canonical-badgevms-abi-layering.md`.

#![cfg_attr(target_arch = "riscv32", no_std)]
#![allow(nonstandard_style)]
#![allow(non_camel_case_types)]
#![feature(c_variadic)]

pub mod bindings;
pub mod types;

pub use bindings::*;
pub use types::*;
