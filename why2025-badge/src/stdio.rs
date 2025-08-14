use core::{fmt, ffi::c_void};

struct Stdio;

impl fmt::Write for Stdio {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unsafe {
            why2025_badge_sys::fwrite(
                s.as_ptr() as *mut c_void,
                1,
                s.len() as u32,
                why2025_badge_sys::stdout,
            );
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    Stdio.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => { $crate::stdio::_print(format_args!($($arg)*)) };
}

#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => { $crate::print!("{}\n", format_args!($($arg)*)) };
}
