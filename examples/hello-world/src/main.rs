#![cfg_attr(target_arch = "riscv32", no_std)]
#![cfg_attr(target_arch = "riscv32", no_main)]

why2025_badge_app_no_std::app_main!(run);

fn run() -> i32 {
    why2025_badge_app_no_std::console::print_bytes(b"Hello, world! (from rust)\n\0");
    121
}
