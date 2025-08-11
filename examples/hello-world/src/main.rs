#![no_std]
#![no_main]

unsafe extern "C" {
    fn printf(format: *const u8, ...) -> i32;
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    unsafe {
        printf(b"Hello, world 2! (from rust)\n\0".as_ptr());
    }
    121
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
