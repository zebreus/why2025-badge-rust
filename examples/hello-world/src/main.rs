#![cfg_attr(target_arch = "riscv32", no_std)]
#![cfg_attr(target_arch = "riscv32", no_main)]

#[cfg(target_arch = "riscv32")]
extern crate alloc;

#[cfg(target_arch = "riscv32")]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    run()
}

#[cfg(not(target_arch = "riscv32"))]
fn main() {
    std::process::exit(run());
}

fn run() -> i32 {
    unsafe {
        why2025_badge_sys::printf(b"Hello, world! (from rust)\n\0".as_ptr().cast());
    }
    121
}

// Allocator and panic handler setup
#[cfg(target_arch = "riscv32")]
use talc::{ClaimOnOom, Span, Talc, Talck};

#[cfg(target_arch = "riscv32")]
const HEAP_SIZE: usize = 1024 * 300; // 300KB heap size
#[cfg(target_arch = "riscv32")]
static mut HEAP: [u8; HEAP_SIZE] = [0u8; HEAP_SIZE];
#[cfg(target_arch = "riscv32")]
#[global_allocator]
static ALLOCATOR: Talck<spin::Mutex<()>, ClaimOnOom> =
    Talc::new(unsafe { ClaimOnOom::new(Span::from_array((&raw const HEAP).cast_mut())) }).lock();

#[cfg(target_arch = "riscv32")]
#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        let maybe_msg = alloc::string::ToString::to_string(&panic_info.message());
        let msg = maybe_msg.as_ptr();
        why2025_badge_sys::printf(b"panic: %s\n\0".as_ptr(), msg);
        if let Some(location) = panic_info.location() {
            why2025_badge_sys::printf(
                b"in %s:%d\n\0".as_ptr(),
                location.file().as_ptr(),
                location.line() as i32,
            );
        } else {
            why2025_badge_sys::printf(b"no location information available :(\n\0".as_ptr());
        }
    }
    loop {}
}
