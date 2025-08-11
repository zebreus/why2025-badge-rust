#![no_std]
#![no_main]
extern crate alloc;

use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{
        Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Triangle,
    },
    text::{Alignment, Text},
};
use why2025_badge_embedded_graphics::Why2025BadgeWindow;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    unsafe {
        why2025_badge_sys::printf(b"Hello, world! (from rust)\n\0".as_ptr());
    }
    let mut display = Why2025BadgeWindow::new_floating(
        Size {
            width: 200,
            height: 200,
        },
        "Graphics Demo",
    );
    loop {
        let val = draw_stuff(&mut display);
        display.flush();
        if val.is_err() {
            unsafe {
                why2025_badge_sys::printf(b"Error drawing to display\n\0".as_ptr());
            }
        } else {
            unsafe {
                why2025_badge_sys::printf(b"Drawing complete\n\0".as_ptr());
            }
        }
    }
}

fn draw_stuff(display: &mut Why2025BadgeWindow) -> Result<(), ()> {
    // This function is just a placeholder to show where you would put additional drawing logic.
    // You can add more shapes, text, or other graphics here.

    // Create styles used by the drawing operations.
    let thin_stroke = PrimitiveStyle::with_stroke(Rgb565::BLUE, 1);
    let thick_stroke = PrimitiveStyle::with_stroke(Rgb565::RED, 3);
    let border_stroke = PrimitiveStyleBuilder::new()
        .stroke_color(Rgb565::YELLOW)
        .stroke_width(3)
        .stroke_alignment(StrokeAlignment::Inside)
        .build();
    let fill = PrimitiveStyle::with_fill(Rgb565::GREEN);
    let character_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

    let yoffset = 10;

    unsafe {
        why2025_badge_sys::printf(b"BBBBBB\n\0".as_ptr());
    }
    // Draw a 3px wide outline around the display.
    display
        .bounding_box()
        .into_styled(border_stroke)
        .draw(display)?;

    unsafe {
        why2025_badge_sys::printf(b"CCCCCC\n\0".as_ptr());
    }

    // Draw a triangle.
    Triangle::new(
        Point::new(16, 16 + yoffset),
        Point::new(16 + 16, 16 + yoffset),
        Point::new(16 + 8, yoffset),
    )
    .into_styled(thin_stroke)
    .draw(display)?;

    // Draw a filled square
    Rectangle::new(Point::new(52, yoffset), Size::new(16, 16))
        .into_styled(fill)
        .draw(display)?;

    // Draw a circle with a 3px wide stroke.
    Circle::new(Point::new(88, yoffset), 17)
        .into_styled(thick_stroke)
        .draw(display)?;

    unsafe {
        why2025_badge_sys::printf(b"DDDDDD\n\0".as_ptr());
    }

    // Draw centered text.
    let text = "embedded-graphics";
    Text::with_alignment(
        text,
        display.bounding_box().center() + Point::new(0, 15),
        character_style,
        Alignment::Center,
    )
    .draw(display)?;
    unsafe {
        why2025_badge_sys::printf(b"EEEEEEE\n\0".as_ptr());
    }

    Ok(())
}

// Allocator and panic handler setup
use talc::{ClaimOnOom, Span, Talc, Talck};

const HEAP_SIZE: usize = 1024 * 300; // 300KB heap size
static mut HEAP: [u8; HEAP_SIZE] = [0u8; HEAP_SIZE];
#[global_allocator]
static ALLOCATOR: Talck<spin::Mutex<()>, ClaimOnOom> =
    Talc::new(unsafe { ClaimOnOom::new(Span::from_array((&raw const HEAP).cast_mut())) }).lock();

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
