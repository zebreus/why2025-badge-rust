#![no_std]
#![no_main]

#[cfg(target_arch = "riscv32")]
#[panic_handler]
fn panic(_panic_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    run()
}

fn run() -> i32 {
    unsafe {
        why2025_badge_sys::printf(b"Hello, world! (manual linking on stock nightly)\n\0".as_ptr().cast());
    }
    121
}
