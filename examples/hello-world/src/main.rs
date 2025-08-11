#![no_std]
#![no_main]

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    unsafe {
        why2025_badge_sys::printf(b"Hello, world! (from rust)\n\0".as_ptr());
    }
    121
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
