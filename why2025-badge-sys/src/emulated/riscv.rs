//! Host-side placeholder exports for GCC's RISC-V save/restore millicode used by upstream
//! BadgeVMS firmware.
//!
//! These symbols are not implemented anywhere under `firmware/badgevms/`; upstream gets them from
//! the `riscv32-esp-elf` GCC toolchain's `libgcc/config/riscv/save-restore.S` because the
//! firmware build sets `IDF_TARGET=esp32p4` and enables
//! `CONFIG_COMPILER_SAVE_RESTORE_LIBCALLS=y`. They are also listed in
//! `firmware/badgevms/symbols.yml`, so the final firmware ABI exports them to loaded BadgeVMS
//! applications that were compiled with GCC's `-msave-restore` support.
//!
//! Exact upstream configuration and consequences:
//!
//! - the relevant libgcc path is the RV32 non-RVE implementation, because ESP32-P4 builds as
//!   `rv32imafc_zicsr_zifencei` with the normal `ilp32f` ABI rather than the reduced-register E ABI
//! - GCC emits prologues as `call t0,__riscv_save_N` and epilogues as `tail __riscv_restore_N`
//! - the save thunks therefore return with `jr t0` instead of `ret`; `t0` is the synthetic link
//!   register for the libcall sequence, not a preserved scratch register
//! - the larger save thunks also clobber `t1` while converting a temporary 64-byte reservation into
//!   the final 16-byte-aligned frame size
//! - only integer callee-saved registers plus `ra` are spilled here; the thunks never touch `fs*`
//!   registers, vector registers, CSRs, heap state, global variables, locks, or any I/O device
//! - all memory traffic is confined to the caller's current stack frame
//! - GCC emits DWARF CFI for unwinding; in the larger save thunks the source comments explicitly
//!   note that CFA information is temporarily inaccurate across the final stack-pointer correction
//!
//! On upstream ESP32-P4 firmware, the 13 exported symbol names collapse into four real machine-code
//! bodies:
//!
//! - `__riscv_save_0` through `__riscv_save_3` and `__riscv_restore_0` through
//!   `__riscv_restore_3` use a 16-byte frame and spill `s2`, `s1`, `s0`, and `ra`
//! - `__riscv_save_4` through `__riscv_save_7` and `__riscv_restore_4` through
//!   `__riscv_restore_7` use a 32-byte frame and spill `s6` down through `s0`, plus `ra`
//! - `__riscv_save_8` through `__riscv_save_11` and `__riscv_restore_8` through
//!   `__riscv_restore_11` use a 48-byte frame and spill `s10` down through `s0`, plus `ra`
//! - `__riscv_save_12` and `__riscv_restore_12` use a 64-byte frame and spill `s11` down through
//!   `s0`, plus `ra`; the lowest 12 bytes of that frame stay unused padding
//!
//! The numeric suffix is therefore not a literal "save exactly N registers" count on this target.
//! On RV32, GCC groups the save/restore libcalls into 16-byte frame buckets, and several suffixes
//! intentionally alias to the same body.
//!
//! Interaction with other GCC features:
//!
//! - leaf functions may avoid these libcalls entirely even when save/restore support is enabled
//! - GCC's `riscv-sr.cc` late optimization pass can remove a matched save/restore pair or rewrite
//!   the last ordinary call into a sibling call when it proves the libcall sequence is unnecessary
//! - the upstream source has a conditional `SET_LPAD` hook for `__riscv_zicfilp`; ESP32-P4's
//!   current march does not enable that extension, so upstream firmware executes no extra landing-pad
//!   instruction in these thunks today
//! - there is no BadgeVMS-specific logic here, so no BadgeVMS-only runtime bug was found; the only
//!   sharp edges are the toolchain-level aliasing and optimization behavior described above
//!
//! Current host-side status:
//!
//! - the Rust stubs below do not emulate any of this upstream behavior yet
//! - every host stub prints a diagnostic and aborts when called, because guest RISC-V
//!   save/restore millicode cannot execute meaningfully inside a native host process

use std::io::{self, Write};

#[cold]
#[inline(never)]
fn abort_riscv_millicode_call(symbol: &str) -> ! {
    let mut stderr = io::stderr().lock();
    let _ = writeln!(
        stderr,
        "{symbol} was called in the host emulator. This symbol is GCC's RISC-V save/restore millicode, not a normal C ABI function: it mutates a guest RISC-V stack frame and returns through guest registers, so it cannot execute correctly inside a native host process. Run the guest under a RISC-V CPU emulator/JIT or rebuild it without -msave-restore."
    );

    std::process::abort()
}

#[unsafe(no_mangle)]
/// RV32 non-RVE save bucket used by upstream `__riscv_save_0`, `__riscv_save_1`,
/// `__riscv_save_2`, and `__riscv_save_3`.
///
/// Exact upstream instructions on ESP32-P4:
///
/// - `addi sp, sp, -16`
/// - `sw s2, 0(sp)`
/// - `sw s1, 4(sp)`
/// - `sw s0, 8(sp)`
/// - `sw ra, 12(sp)`
/// - `jr t0`
///
/// Exact effects:
///
/// - final frame contribution: 16 bytes
/// - saved words at final frame offsets `0=s2`, `4=s1`, `8=s0`, `12=ra`
/// - control returns through `t0`, because GCC reached the thunk with `call t0,__riscv_save_0`
/// - no register other than `sp`, `s2`, `s1`, `s0`, and `ra` is touched in the actual save body
/// - the `0` suffix does not mean "save nothing" on upstream ESP32-P4 firmware
pub extern "C" fn __riscv_save_0() {
    abort_riscv_millicode_call("__riscv_save_0")
}
#[unsafe(no_mangle)]
/// Exact upstream alias of [`__riscv_save_0`].
///
/// On upstream ESP32-P4 firmware, `__riscv_save_1` is instruction-for-instruction identical to
/// [`__riscv_save_0`]. The differing suffix is only GCC's internal prologue selector; the runtime
/// spill set and 16-byte frame layout are the same.
pub extern "C" fn __riscv_save_1() {
    abort_riscv_millicode_call("__riscv_save_1")
}
#[unsafe(no_mangle)]
/// Exact upstream alias of [`__riscv_save_0`].
///
/// The ESP32-P4 RV32 non-RVE libgcc body for `__riscv_save_2` still saves only `s2`, `s1`, `s0`,
/// and `ra` in a 16-byte frame, then returns with `jr t0`.
pub extern "C" fn __riscv_save_2() {
    abort_riscv_millicode_call("__riscv_save_2")
}
#[unsafe(no_mangle)]
/// Exact upstream alias of [`__riscv_save_0`].
///
/// `__riscv_save_3` shares the same 16-byte RV32 spill bucket as `__riscv_save_0` through
/// `__riscv_save_2`; there is no extra `s3` spill in the actual ESP32-P4 machine code.
pub extern "C" fn __riscv_save_3() {
    abort_riscv_millicode_call("__riscv_save_3")
}
#[unsafe(no_mangle)]
/// RV32 non-RVE save bucket used by upstream `__riscv_save_4` through `__riscv_save_7`.
///
/// Exact upstream instruction shape on ESP32-P4:
///
/// - `addi sp, sp, -64`
/// - `li t1, -32`
/// - `sw s6, 32(sp)`
/// - `sw s5, 36(sp)`
/// - `sw s4, 40(sp)`
/// - `sw s3, 44(sp)`
/// - `sw s2, 48(sp)`
/// - `sw s1, 52(sp)`
/// - `sw s0, 56(sp)`
/// - `sw ra, 60(sp)`
/// - `sub sp, sp, t1` which, with `t1 = -32`, leaves a final 32-byte frame
/// - `jr t0`
///
/// Exact final frame layout after that correction:
///
/// - `0=s6`, `4=s5`, `8=s4`, `12=s3`, `16=s2`, `20=s1`, `24=s0`, `28=ra`
///
/// Side effects:
///
/// - clobbers `sp` and `t1`
/// - returns through `t0`
/// - temporarily over-reserves 64 bytes before sliding `sp` back up by 32 bytes
pub extern "C" fn __riscv_save_4() {
    abort_riscv_millicode_call("__riscv_save_4")
}
#[unsafe(no_mangle)]
/// Exact upstream alias of [`__riscv_save_4`].
///
/// On upstream ESP32-P4 firmware, `__riscv_save_5` uses the same 32-byte spill bucket and the same
/// saved-register layout as [`__riscv_save_4`].
pub extern "C" fn __riscv_save_5() {
    abort_riscv_millicode_call("__riscv_save_5")
}
#[unsafe(no_mangle)]
/// Exact upstream alias of [`__riscv_save_4`].
///
/// The ESP32-P4 RV32 non-RVE implementation of `__riscv_save_6` still saves `s6` down through
/// `s0`, plus `ra`, and finishes with the same final 32-byte frame as [`__riscv_save_4`].
pub extern "C" fn __riscv_save_6() {
    abort_riscv_millicode_call("__riscv_save_6")
}
#[unsafe(no_mangle)]
/// Exact upstream alias of [`__riscv_save_4`].
///
/// `__riscv_save_7` is not a distinct runtime body on upstream ESP32-P4 firmware; it resolves to
/// the same 32-byte `s6..s0,ra` spill sequence as [`__riscv_save_4`].
pub extern "C" fn __riscv_save_7() {
    abort_riscv_millicode_call("__riscv_save_7")
}
#[unsafe(no_mangle)]
/// RV32 non-RVE save bucket used by upstream `__riscv_save_8` through `__riscv_save_11`.
///
/// Exact upstream instruction shape on ESP32-P4:
///
/// - `addi sp, sp, -64`
/// - `li t1, -16`
/// - `sw s10, 16(sp)`
/// - `sw s9, 20(sp)`
/// - `sw s8, 24(sp)`
/// - `sw s7, 28(sp)`
/// - `sw s6, 32(sp)`
/// - `sw s5, 36(sp)`
/// - `sw s4, 40(sp)`
/// - `sw s3, 44(sp)`
/// - `sw s2, 48(sp)`
/// - `sw s1, 52(sp)`
/// - `sw s0, 56(sp)`
/// - `sw ra, 60(sp)`
/// - `sub sp, sp, t1` which, with `t1 = -16`, leaves a final 48-byte frame
/// - `jr t0`
///
/// Exact final frame layout after that correction:
///
/// - `0=s10`, `4=s9`, `8=s8`, `12=s7`, `16=s6`, `20=s5`, `24=s4`, `28=s3`, `32=s2`, `36=s1`,
///   `40=s0`, `44=ra`
///
/// Side effects:
///
/// - clobbers `sp` and `t1`
/// - returns through `t0`
/// - temporarily over-reserves 64 bytes before sliding `sp` back up by 16 bytes
pub extern "C" fn __riscv_save_8() {
    abort_riscv_millicode_call("__riscv_save_8")
}
#[unsafe(no_mangle)]
/// Exact upstream alias of [`__riscv_save_8`].
///
/// On upstream ESP32-P4 firmware, `__riscv_save_9` is not distinct code; it shares the same 48-byte
/// `s10..s0,ra` spill bucket as [`__riscv_save_8`].
pub extern "C" fn __riscv_save_9() {
    abort_riscv_millicode_call("__riscv_save_9")
}
#[unsafe(no_mangle)]
/// Exact upstream alias of [`__riscv_save_8`].
///
/// The ESP32-P4 RV32 non-RVE implementation of `__riscv_save_10` is instruction-for-instruction
/// identical to [`__riscv_save_8`].
pub extern "C" fn __riscv_save_10() {
    abort_riscv_millicode_call("__riscv_save_10")
}
#[unsafe(no_mangle)]
/// Exact upstream alias of [`__riscv_save_8`].
///
/// `__riscv_save_11` uses the same 48-byte save bucket and the same final stack layout as
/// [`__riscv_save_8`] on upstream ESP32-P4 firmware.
pub extern "C" fn __riscv_save_11() {
    abort_riscv_millicode_call("__riscv_save_11")
}
#[unsafe(no_mangle)]
/// Full RV32 non-RVE save bucket used by upstream `__riscv_save_12`.
///
/// Exact upstream instruction shape on ESP32-P4:
///
/// - `addi sp, sp, -64`
/// - `li t1, 0`
/// - `sw s11, 12(sp)`
/// - `sw s10, 16(sp)`
/// - `sw s9, 20(sp)`
/// - `sw s8, 24(sp)`
/// - `sw s7, 28(sp)`
/// - `sw s6, 32(sp)`
/// - `sw s5, 36(sp)`
/// - `sw s4, 40(sp)`
/// - `sw s3, 44(sp)`
/// - `sw s2, 48(sp)`
/// - `sw s1, 52(sp)`
/// - `sw s0, 56(sp)`
/// - `sw ra, 60(sp)`
/// - `sub sp, sp, t1` which is a no-op here because `t1 = 0`
/// - `jr t0`
///
/// Exact final frame layout:
///
/// - `0..=11` unused padding
/// - `12=s11`, `16=s10`, `20=s9`, `24=s8`, `28=s7`, `32=s6`, `36=s5`, `40=s4`, `44=s3`,
///   `48=s2`, `52=s1`, `56=s0`, `60=ra`
///
/// The 12-byte hole at the low end is intentional. Upstream keeps the whole spill area 16-byte
/// aligned even though 13 saved words consume only 52 bytes.
pub extern "C" fn __riscv_save_12() {
    abort_riscv_millicode_call("__riscv_save_12")
}
#[unsafe(no_mangle)]
/// RV32 non-RVE restore bucket used by upstream `__riscv_restore_0`, `__riscv_restore_1`,
/// `__riscv_restore_2`, and `__riscv_restore_3`.
///
/// Exact upstream instructions on ESP32-P4:
///
/// - `lw s2, 0(sp)`
/// - `lw s1, 4(sp)`
/// - `lw s0, 8(sp)`
/// - `lw ra, 12(sp)`
/// - `addi sp, sp, 16`
/// - `ret`
///
/// Exact effects:
///
/// - expects the 16-byte frame layout produced by [`__riscv_save_0`] through [`__riscv_save_3`]
/// - restores `s2`, `s1`, `s0`, and `ra`
/// - releases exactly 16 bytes of stack
/// - does not use `t0`; GCC reaches this thunk with `tail __riscv_restore_0`, so the thunk itself
///   performs the final architectural return
pub extern "C" fn __riscv_restore_0() {
    abort_riscv_millicode_call("__riscv_restore_0")
}
#[unsafe(no_mangle)]
/// Exact upstream alias of [`__riscv_restore_0`].
///
/// On upstream ESP32-P4 firmware, `__riscv_restore_1` is instruction-for-instruction identical to
/// [`__riscv_restore_0`] and expects the same 16-byte frame layout.
pub extern "C" fn __riscv_restore_1() {
    abort_riscv_millicode_call("__riscv_restore_1")
}
#[unsafe(no_mangle)]
/// Exact upstream alias of [`__riscv_restore_0`].
///
/// The ESP32-P4 RV32 non-RVE implementation of `__riscv_restore_2` still restores only `s2`,
/// `s1`, `s0`, and `ra` from a 16-byte frame, just like [`__riscv_restore_0`].
pub extern "C" fn __riscv_restore_2() {
    abort_riscv_millicode_call("__riscv_restore_2")
}
#[unsafe(no_mangle)]
/// Exact upstream alias of [`__riscv_restore_0`].
///
/// `__riscv_restore_3` does not restore an extra `s3` on upstream ESP32-P4 firmware; it reuses the
/// same 16-byte RV32 restore bucket as [`__riscv_restore_0`].
pub extern "C" fn __riscv_restore_3() {
    abort_riscv_millicode_call("__riscv_restore_3")
}
#[unsafe(no_mangle)]
/// RV32 non-RVE restore bucket used by upstream `__riscv_restore_4` through `__riscv_restore_7`.
///
/// Exact upstream instruction shape on ESP32-P4:
///
/// - `lw s6, 0(sp)`
/// - `lw s5, 4(sp)`
/// - `lw s4, 8(sp)`
/// - `lw s3, 12(sp)`
/// - `addi sp, sp, 16`
/// - `lw s2, 0(sp)`
/// - `lw s1, 4(sp)`
/// - `lw s0, 8(sp)`
/// - `lw ra, 12(sp)`
/// - `addi sp, sp, 16`
/// - `ret`
///
/// Exact effects:
///
/// - expects the 32-byte frame laid out by [`__riscv_save_4`] through [`__riscv_save_7`]
/// - restores `s6` down through `s0`, plus `ra`
/// - releases exactly 32 bytes of stack in two 16-byte steps
pub extern "C" fn __riscv_restore_4() {
    abort_riscv_millicode_call("__riscv_restore_4")
}
#[unsafe(no_mangle)]
/// Exact upstream alias of [`__riscv_restore_4`].
///
/// On upstream ESP32-P4 firmware, `__riscv_restore_5` is the same 32-byte `s6..s0,ra` restore
/// sequence as [`__riscv_restore_4`].
pub extern "C" fn __riscv_restore_5() {
    abort_riscv_millicode_call("__riscv_restore_5")
}
#[unsafe(no_mangle)]
/// Exact upstream alias of [`__riscv_restore_4`].
///
/// The ESP32-P4 RV32 non-RVE implementation of `__riscv_restore_6` shares the same machine code and
/// frame contract as [`__riscv_restore_4`].
pub extern "C" fn __riscv_restore_6() {
    abort_riscv_millicode_call("__riscv_restore_6")
}
#[unsafe(no_mangle)]
/// Exact upstream alias of [`__riscv_restore_4`].
///
/// `__riscv_restore_7` is not a distinct restore body on upstream ESP32-P4 firmware; it reuses the
/// same 32-byte restore bucket as [`__riscv_restore_4`].
pub extern "C" fn __riscv_restore_7() {
    abort_riscv_millicode_call("__riscv_restore_7")
}
#[unsafe(no_mangle)]
/// RV32 non-RVE restore bucket used by upstream `__riscv_restore_8` through `__riscv_restore_11`.
///
/// Exact upstream instruction shape on ESP32-P4:
///
/// - `lw s10, 0(sp)`
/// - `lw s9, 4(sp)`
/// - `lw s8, 8(sp)`
/// - `lw s7, 12(sp)`
/// - `addi sp, sp, 16`
/// - `lw s6, 0(sp)`
/// - `lw s5, 4(sp)`
/// - `lw s4, 8(sp)`
/// - `lw s3, 12(sp)`
/// - `addi sp, sp, 16`
/// - `lw s2, 0(sp)`
/// - `lw s1, 4(sp)`
/// - `lw s0, 8(sp)`
/// - `lw ra, 12(sp)`
/// - `addi sp, sp, 16`
/// - `ret`
///
/// Exact effects:
///
/// - expects the 48-byte frame laid out by [`__riscv_save_8`] through [`__riscv_save_11`]
/// - restores `s10` down through `s0`, plus `ra`
/// - releases exactly 48 bytes of stack in three 16-byte steps
pub extern "C" fn __riscv_restore_8() {
    abort_riscv_millicode_call("__riscv_restore_8")
}
#[unsafe(no_mangle)]
/// Exact upstream alias of [`__riscv_restore_8`].
///
/// On upstream ESP32-P4 firmware, `__riscv_restore_9` shares the same 48-byte restore sequence and
/// frame contract as [`__riscv_restore_8`].
pub extern "C" fn __riscv_restore_9() {
    abort_riscv_millicode_call("__riscv_restore_9")
}
#[unsafe(no_mangle)]
/// Exact upstream alias of [`__riscv_restore_8`].
///
/// The ESP32-P4 RV32 non-RVE implementation of `__riscv_restore_10` is instruction-for-instruction
/// identical to [`__riscv_restore_8`].
pub extern "C" fn __riscv_restore_10() {
    abort_riscv_millicode_call("__riscv_restore_10")
}
#[unsafe(no_mangle)]
/// Exact upstream alias of [`__riscv_restore_8`].
///
/// `__riscv_restore_11` uses the same 48-byte `s10..s0,ra` restore bucket as
/// [`__riscv_restore_8`] on upstream ESP32-P4 firmware.
pub extern "C" fn __riscv_restore_11() {
    abort_riscv_millicode_call("__riscv_restore_11")
}
#[unsafe(no_mangle)]
/// Full RV32 non-RVE restore bucket used by upstream `__riscv_restore_12`.
///
/// Exact upstream instruction shape on ESP32-P4:
///
/// - `lw s11, 12(sp)`
/// - `addi sp, sp, 16`
/// - `lw s10, 0(sp)`
/// - `lw s9, 4(sp)`
/// - `lw s8, 8(sp)`
/// - `lw s7, 12(sp)`
/// - `addi sp, sp, 16`
/// - `lw s6, 0(sp)`
/// - `lw s5, 4(sp)`
/// - `lw s4, 8(sp)`
/// - `lw s3, 12(sp)`
/// - `addi sp, sp, 16`
/// - `lw s2, 0(sp)`
/// - `lw s1, 4(sp)`
/// - `lw s0, 8(sp)`
/// - `lw ra, 12(sp)`
/// - `addi sp, sp, 16`
/// - `ret`
///
/// Exact effects:
///
/// - expects the 64-byte frame laid out by [`__riscv_save_12`], including the unused low 12-byte
///   pad before the stored `s11`
/// - restores `s11` down through `s0`, plus `ra`
/// - releases exactly 64 bytes of stack in four 16-byte steps
pub extern "C" fn __riscv_restore_12() {
    abort_riscv_millicode_call("__riscv_restore_12")
}

#[cfg(test)]
mod tests {
    use super::__riscv_save_0;

    #[cfg(unix)]
    use std::{os::unix::process::ExitStatusExt, process::Command};

    #[test]
    #[cfg(unix)]
    fn riscv_save_stub_aborts_with_diagnostic() {
        const ENV_NAME: &str = "WHY2025_RISCV_SAVE_STUB_TEST";
        const TEST_NAME: &str = "emulated::riscv::tests::riscv_save_stub_aborts_with_diagnostic";

        if std::env::var_os(ENV_NAME).is_some() {
            __riscv_save_0();
        }

        let output = Command::new(std::env::current_exe().expect("current test binary path"))
            .arg("--exact")
            .arg(TEST_NAME)
            .env(ENV_NAME, "1")
            .output()
            .expect("spawn child test process");

        assert!(!output.status.success());
        assert_eq!(output.status.signal(), Some(libc::SIGABRT));

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("__riscv_save_0 was called in the host emulator"));
        assert!(stderr.contains("-msave-restore"));
    }
}
