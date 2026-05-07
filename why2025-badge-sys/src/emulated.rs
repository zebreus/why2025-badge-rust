//! Implementations for the symbols not available in glibc or modeled locally

extern crate libc;

pub mod badgevms;
pub mod libc_fallback;
pub mod riscv;

pub use libc_fallback::diprintf;
