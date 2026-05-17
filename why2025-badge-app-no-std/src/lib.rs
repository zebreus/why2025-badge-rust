#![cfg_attr(target_arch = "riscv32", no_std)]

use core::ffi::CStr;

#[cfg(all(target_arch = "riscv32", feature = "provided-panic-handler"))]
extern crate alloc;
#[cfg(not(target_arch = "riscv32"))]
use why2025_badge_emu_abi as _;
#[cfg(target_arch = "riscv32")]
use why2025_badge_sys_bindings as _;

pub use why2025_badge_sys_bindings as sys;

const PRINT_AS_STRING: &[u8] = b"%s\0";

pub mod console {
    use super::{CStr, PRINT_AS_STRING};

    pub fn print(message: &CStr) {
        unsafe {
            crate::sys::printf(PRINT_AS_STRING.as_ptr().cast(), message.as_ptr());
        }
    }

    pub fn print_bytes(message: &[u8]) {
        let message = CStr::from_bytes_with_nul(message)
            .expect("why2025-badge-app-no-std::console::print_bytes expects a trailing NUL byte");
        print(message);
    }
}

#[cfg(all(target_arch = "riscv32", feature = "provided-runtime"))]
mod runtime {
    #[cfg(feature = "provided-allocator")]
    use talc::{ClaimOnOom, Span, Talc, Talck};

    #[cfg(feature = "provided-allocator")]
    const HEAP_SIZE: usize = 1024 * 300;

    #[cfg(feature = "provided-allocator")]
    static mut HEAP: [u8; HEAP_SIZE] = [0u8; HEAP_SIZE];

    #[cfg(feature = "provided-allocator")]
    #[global_allocator]
    static ALLOCATOR: Talck<spin::Mutex<()>, ClaimOnOom> =
        Talc::new(unsafe { ClaimOnOom::new(Span::from_array((&raw const HEAP).cast_mut())) })
            .lock();

    #[cfg(feature = "provided-panic-handler")]
    #[panic_handler]
    fn panic(panic_info: &core::panic::PanicInfo) -> ! {
        unsafe {
            let maybe_msg = alloc::string::ToString::to_string(&panic_info.message());
            let msg = maybe_msg.as_ptr();
            crate::sys::printf(b"panic: %s\n\0".as_ptr(), msg);
            if let Some(location) = panic_info.location() {
                crate::sys::printf(
                    b"in %s:%d\n\0".as_ptr(),
                    location.file().as_ptr(),
                    location.line() as i32,
                );
            } else {
                crate::sys::printf(b"no location information available :(\n\0".as_ptr());
            }
        }
        loop {}
    }
}

#[macro_export]
macro_rules! app_main {
    ($run:path) => {
        #[cfg(target_arch = "riscv32")]
        #[unsafe(no_mangle)]
        pub extern "C" fn main() -> i32 {
            $run()
        }

        #[cfg(not(target_arch = "riscv32"))]
        fn main() {
            ::std::process::exit($run());
        }
    };
}
