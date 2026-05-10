//! Canonical raw generated bindings for the functions exported by the WHY2025 badge.
//!
//! This crate is the authoritative raw BadgeVMS ABI artifact for both std and no_std consumers in
//! this repository. It owns the generated ABI surface together with the regeneration workflow and
//! input snapshots used to produce it.
//!
//! The patched BadgeVMS std port consumes this crate directly. The sibling `why2025-badge-sys`
//! crate re-exports the same raw surface and adds wrapper-only behavior such as Host builds using
//! Emulation and no_std badge-link support.

#![cfg_attr(target_arch = "riscv32", no_std)]
#![allow(nonstandard_style)]
#![allow(non_camel_case_types)]
#![feature(c_variadic)]

pub mod bindings;
pub mod types;

pub use bindings::*;
pub use types::*;
