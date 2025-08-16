use crate::{gettimeofday, types::*};
use minifb::{Window, WindowOptions};
use std::sync::{Arc, LazyLock, RwLock};

pub fn minifb_key_to_char(key: &minifb::Key) -> Option<core::ffi::c_char> {
    // TODO: Process modifiers
    let char = match key {
        minifb::Key::Key0 => '0' as core::ffi::c_char,
        minifb::Key::Key1 => '1' as core::ffi::c_char,
        minifb::Key::Key2 => '2' as core::ffi::c_char,
        minifb::Key::Key3 => '3' as core::ffi::c_char,
        minifb::Key::Key4 => '4' as core::ffi::c_char,
        minifb::Key::Key5 => '5' as core::ffi::c_char,
        minifb::Key::Key6 => '6' as core::ffi::c_char,
        minifb::Key::Key7 => '7' as core::ffi::c_char,
        minifb::Key::Key8 => '8' as core::ffi::c_char,
        minifb::Key::Key9 => '9' as core::ffi::c_char,
        minifb::Key::A => 'a' as core::ffi::c_char,
        minifb::Key::B => 'b' as core::ffi::c_char,
        minifb::Key::C => 'c' as core::ffi::c_char,
        minifb::Key::D => 'd' as core::ffi::c_char,
        minifb::Key::E => 'e' as core::ffi::c_char,
        minifb::Key::F => 'f' as core::ffi::c_char,
        minifb::Key::G => 'g' as core::ffi::c_char,
        minifb::Key::H => 'h' as core::ffi::c_char,
        minifb::Key::I => 'i' as core::ffi::c_char,
        minifb::Key::J => 'j' as core::ffi::c_char,
        minifb::Key::K => 'k' as core::ffi::c_char,
        minifb::Key::L => 'l' as core::ffi::c_char,
        minifb::Key::M => 'm' as core::ffi::c_char,
        minifb::Key::N => 'n' as core::ffi::c_char,
        minifb::Key::O => 'o' as core::ffi::c_char,
        minifb::Key::P => 'p' as core::ffi::c_char,
        minifb::Key::Q => 'q' as core::ffi::c_char,
        minifb::Key::R => 'r' as core::ffi::c_char,
        minifb::Key::S => 's' as core::ffi::c_char,
        minifb::Key::T => 't' as core::ffi::c_char,
        minifb::Key::U => 'u' as core::ffi::c_char,
        minifb::Key::V => 'v' as core::ffi::c_char,
        minifb::Key::W => 'w' as core::ffi::c_char,
        minifb::Key::X => 'x' as core::ffi::c_char,
        minifb::Key::Y => 'y' as core::ffi::c_char,
        minifb::Key::Z => 'z' as core::ffi::c_char,
        minifb::Key::Backquote => '`' as core::ffi::c_char,
        minifb::Key::Backslash => '\\' as core::ffi::c_char,
        minifb::Key::Comma => ',' as core::ffi::c_char,
        minifb::Key::Equal => '=' as core::ffi::c_char,
        minifb::Key::LeftBracket => '[' as core::ffi::c_char,
        minifb::Key::Minus => '-' as core::ffi::c_char,
        minifb::Key::Period => '.' as core::ffi::c_char,
        minifb::Key::RightBracket => ']' as core::ffi::c_char,
        minifb::Key::Semicolon => ';' as core::ffi::c_char,
        minifb::Key::Slash => '/' as core::ffi::c_char,
        minifb::Key::Space => ' ' as core::ffi::c_char,
        minifb::Key::Tab => '\t' as core::ffi::c_char,
        _ => return None,
    };
    Some(char)
}

pub fn minifb_key_to_keycode(key: &minifb::Key) -> u32 {
    // TODO: Process modifiers
    match key {
        minifb::Key::Key0 => '0' as u32,
        minifb::Key::Key1 => '1' as u32,
        minifb::Key::Key2 => '2' as u32,
        minifb::Key::Key3 => '3' as u32,
        minifb::Key::Key4 => '4' as u32,
        minifb::Key::Key5 => '5' as u32,
        minifb::Key::Key6 => '6' as u32,
        minifb::Key::Key7 => '7' as u32,
        minifb::Key::Key8 => '8' as u32,
        minifb::Key::Key9 => '9' as u32,
        minifb::Key::A => 'a' as u32,
        minifb::Key::B => 'b' as u32,
        minifb::Key::C => 'c' as u32,
        minifb::Key::D => 'd' as u32,
        minifb::Key::E => 'e' as u32,
        minifb::Key::F => 'f' as u32,
        minifb::Key::G => 'g' as u32,
        minifb::Key::H => 'h' as u32,
        minifb::Key::I => 'i' as u32,
        minifb::Key::J => 'j' as u32,
        minifb::Key::K => 'k' as u32,
        minifb::Key::L => 'l' as u32,
        minifb::Key::M => 'm' as u32,
        minifb::Key::N => 'n' as u32,
        minifb::Key::O => 'o' as u32,
        minifb::Key::P => 'p' as u32,
        minifb::Key::Q => 'q' as u32,
        minifb::Key::R => 'r' as u32,
        minifb::Key::S => 's' as u32,
        minifb::Key::T => 't' as u32,
        minifb::Key::U => 'u' as u32,
        minifb::Key::V => 'v' as u32,
        minifb::Key::W => 'w' as u32,
        minifb::Key::X => 'x' as u32,
        minifb::Key::Y => 'y' as u32,
        minifb::Key::Z => 'z' as u32,
        // minifb::Key::F1 => todo!(),
        // minifb::Key::F2 => todo!(),
        // minifb::Key::F3 => todo!(),
        // minifb::Key::F4 => todo!(),
        // minifb::Key::F5 => todo!(),
        // minifb::Key::F6 => todo!(),
        // minifb::Key::F7 => todo!(),
        // minifb::Key::F8 => todo!(),
        // minifb::Key::F9 => todo!(),
        // minifb::Key::F10 => todo!(),
        // minifb::Key::F11 => todo!(),
        // minifb::Key::F12 => todo!(),
        // minifb::Key::F13 => todo!(),
        // minifb::Key::F14 => todo!(),
        // minifb::Key::F15 => todo!(),
        // minifb::Key::Down => todo!(),
        // minifb::Key::Left => todo!(),
        // minifb::Key::Right => todo!(),
        // minifb::Key::Up => todo!(),
        // minifb::Key::Apostrophe => todo!(),
        minifb::Key::Backquote => '`' as u32,
        minifb::Key::Backslash => '\\' as u32,
        minifb::Key::Comma => ',' as u32,
        minifb::Key::Equal => '=' as u32,
        minifb::Key::LeftBracket => '[' as u32,
        minifb::Key::Minus => '-' as u32,
        minifb::Key::Period => '.' as u32,
        minifb::Key::RightBracket => ']' as u32,
        minifb::Key::Semicolon => ';' as u32,
        minifb::Key::Slash => '/' as u32,
        // minifb::Key::Backspace => todo!(),
        // minifb::Key::Delete => todo!(),
        // minifb::Key::End => todo!(),
        // minifb::Key::Enter => todo!(),
        // minifb::Key::Escape => '\x1B',
        // minifb::Key::Home => todo!(),
        // minifb::Key::Insert => todo!(),
        // minifb::Key::Menu => todo!(),
        // minifb::Key::PageDown => todo!(),
        // minifb::Key::PageUp => todo!(),
        // minifb::Key::Pause => todo!(),
        minifb::Key::Space => ' ' as u32,
        minifb::Key::Tab => '\t' as u32,
        // minifb::Key::NumLock => todo!(),
        // minifb::Key::CapsLock => todo!(),
        // minifb::Key::ScrollLock => todo!(),
        // minifb::Key::LeftShift => todo!(),
        // minifb::Key::RightShift => todo!(),
        // minifb::Key::LeftCtrl => todo!(),
        // minifb::Key::RightCtrl => todo!(),
        // minifb::Key::NumPad0 => todo!(),
        // minifb::Key::NumPad1 => todo!(),
        // minifb::Key::NumPad2 => todo!(),
        // minifb::Key::NumPad3 => todo!(),
        // minifb::Key::NumPad4 => todo!(),
        // minifb::Key::NumPad5 => todo!(),
        // minifb::Key::NumPad6 => todo!(),
        // minifb::Key::NumPad7 => todo!(),
        // minifb::Key::NumPad8 => todo!(),
        // minifb::Key::NumPad9 => todo!(),
        // minifb::Key::NumPadDot => todo!(),
        // minifb::Key::NumPadSlash => todo!(),
        // minifb::Key::NumPadAsterisk => todo!(),
        // minifb::Key::NumPadMinus => todo!(),
        // minifb::Key::NumPadPlus => todo!(),
        // minifb::Key::NumPadEnter => todo!(),
        // minifb::Key::LeftAlt => todo!(),
        // minifb::Key::RightAlt => todo!(),
        // minifb::Key::LeftSuper => todo!(),
        // minifb::Key::RightSuper => todo!(),
        // minifb::Key::Unknown => todo!(),
        // minifb::Key::Count => todo!(),
        key => minifb_key_to_scancode(key) as u32 | (1 << 30),
    }
}

pub fn minifb_key_to_scancode(key: &minifb::Key) -> keyboard_scancode_t {
    match key {
        minifb::Key::Key0 => keyboard_scancode_t::KEY_SCANCODE_0,
        minifb::Key::Key1 => keyboard_scancode_t::KEY_SCANCODE_1,
        minifb::Key::Key2 => keyboard_scancode_t::KEY_SCANCODE_2,
        minifb::Key::Key3 => keyboard_scancode_t::KEY_SCANCODE_3,
        minifb::Key::Key4 => keyboard_scancode_t::KEY_SCANCODE_4,
        minifb::Key::Key5 => keyboard_scancode_t::KEY_SCANCODE_5,
        minifb::Key::Key6 => keyboard_scancode_t::KEY_SCANCODE_6,
        minifb::Key::Key7 => keyboard_scancode_t::KEY_SCANCODE_7,
        minifb::Key::Key8 => keyboard_scancode_t::KEY_SCANCODE_8,
        minifb::Key::Key9 => keyboard_scancode_t::KEY_SCANCODE_9,
        minifb::Key::A => keyboard_scancode_t::KEY_SCANCODE_A,
        minifb::Key::B => keyboard_scancode_t::KEY_SCANCODE_B,
        minifb::Key::C => keyboard_scancode_t::KEY_SCANCODE_C,
        minifb::Key::D => keyboard_scancode_t::KEY_SCANCODE_D,
        minifb::Key::E => keyboard_scancode_t::KEY_SCANCODE_E,
        minifb::Key::F => keyboard_scancode_t::KEY_SCANCODE_F,
        minifb::Key::G => keyboard_scancode_t::KEY_SCANCODE_G,
        minifb::Key::H => keyboard_scancode_t::KEY_SCANCODE_H,
        minifb::Key::I => keyboard_scancode_t::KEY_SCANCODE_I,
        minifb::Key::J => keyboard_scancode_t::KEY_SCANCODE_J,
        minifb::Key::K => keyboard_scancode_t::KEY_SCANCODE_K,
        minifb::Key::L => keyboard_scancode_t::KEY_SCANCODE_L,
        minifb::Key::M => keyboard_scancode_t::KEY_SCANCODE_M,
        minifb::Key::N => keyboard_scancode_t::KEY_SCANCODE_N,
        minifb::Key::O => keyboard_scancode_t::KEY_SCANCODE_O,
        minifb::Key::P => keyboard_scancode_t::KEY_SCANCODE_P,
        minifb::Key::Q => keyboard_scancode_t::KEY_SCANCODE_Q,
        minifb::Key::R => keyboard_scancode_t::KEY_SCANCODE_R,
        minifb::Key::S => keyboard_scancode_t::KEY_SCANCODE_S,
        minifb::Key::T => keyboard_scancode_t::KEY_SCANCODE_T,
        minifb::Key::U => keyboard_scancode_t::KEY_SCANCODE_U,
        minifb::Key::V => keyboard_scancode_t::KEY_SCANCODE_V,
        minifb::Key::W => keyboard_scancode_t::KEY_SCANCODE_W,
        minifb::Key::X => keyboard_scancode_t::KEY_SCANCODE_X,
        minifb::Key::Y => keyboard_scancode_t::KEY_SCANCODE_Y,
        minifb::Key::Z => keyboard_scancode_t::KEY_SCANCODE_Z,
        minifb::Key::F1 => keyboard_scancode_t::KEY_SCANCODE_ESCAPE,
        minifb::Key::F2 => keyboard_scancode_t::KEY_SCANCODE_SQUARE,
        minifb::Key::F3 => keyboard_scancode_t::KEY_SCANCODE_TRIANGLE,
        minifb::Key::F4 => keyboard_scancode_t::KEY_SCANCODE_CROSS,
        minifb::Key::F5 => keyboard_scancode_t::KEY_SCANCODE_CIRCLE,
        minifb::Key::F6 => keyboard_scancode_t::KEY_SCANCODE_CLOUD,
        minifb::Key::F7 => keyboard_scancode_t::KEY_SCANCODE_DIAMOND,
        minifb::Key::F8 => keyboard_scancode_t::KEY_SCANCODE_BACKSPACE,
        minifb::Key::F9 => keyboard_scancode_t::KEY_SCANCODE_F9,
        minifb::Key::F10 => keyboard_scancode_t::KEY_SCANCODE_F10,
        minifb::Key::F11 => keyboard_scancode_t::KEY_SCANCODE_F11,
        minifb::Key::F12 => keyboard_scancode_t::KEY_SCANCODE_F12,
        minifb::Key::F13 => keyboard_scancode_t::KEY_SCANCODE_F13,
        minifb::Key::F14 => keyboard_scancode_t::KEY_SCANCODE_F14,
        minifb::Key::F15 => keyboard_scancode_t::KEY_SCANCODE_F15,
        minifb::Key::Down => keyboard_scancode_t::KEY_SCANCODE_DOWN,
        minifb::Key::Left => keyboard_scancode_t::KEY_SCANCODE_LEFT,
        minifb::Key::Right => keyboard_scancode_t::KEY_SCANCODE_RIGHT,
        minifb::Key::Up => keyboard_scancode_t::KEY_SCANCODE_UP,
        minifb::Key::Apostrophe => keyboard_scancode_t::KEY_SCANCODE_APOSTROPHE,
        minifb::Key::Backquote => keyboard_scancode_t::KEY_SCANCODE_GRAVE,
        minifb::Key::Backslash => keyboard_scancode_t::KEY_SCANCODE_BACKSLASH,
        minifb::Key::Comma => keyboard_scancode_t::KEY_SCANCODE_COMMA,
        minifb::Key::Equal => keyboard_scancode_t::KEY_SCANCODE_EQUALS,
        minifb::Key::LeftBracket => keyboard_scancode_t::KEY_SCANCODE_LEFTBRACKET,
        minifb::Key::Minus => keyboard_scancode_t::KEY_SCANCODE_MINUS,
        minifb::Key::Period => keyboard_scancode_t::KEY_SCANCODE_PERIOD,
        minifb::Key::RightBracket => keyboard_scancode_t::KEY_SCANCODE_RIGHTBRACKET,
        minifb::Key::Semicolon => keyboard_scancode_t::KEY_SCANCODE_SEMICOLON,
        minifb::Key::Slash => keyboard_scancode_t::KEY_SCANCODE_SLASH,
        minifb::Key::Backspace => keyboard_scancode_t::KEY_SCANCODE_BACKSPACE,
        minifb::Key::Delete => keyboard_scancode_t::KEY_SCANCODE_DELETE,
        minifb::Key::End => keyboard_scancode_t::KEY_SCANCODE_END,
        minifb::Key::Enter => keyboard_scancode_t::KEY_SCANCODE_RETURN,
        minifb::Key::Escape => keyboard_scancode_t::KEY_SCANCODE_ESCAPE,
        minifb::Key::Home => keyboard_scancode_t::KEY_SCANCODE_HOME,
        minifb::Key::Insert => keyboard_scancode_t::KEY_SCANCODE_INSERT,
        minifb::Key::Menu => keyboard_scancode_t::KEY_SCANCODE_MENU,
        minifb::Key::PageDown => keyboard_scancode_t::KEY_SCANCODE_PAGEDOWN,
        minifb::Key::PageUp => keyboard_scancode_t::KEY_SCANCODE_PAGEUP,
        minifb::Key::Pause => keyboard_scancode_t::KEY_SCANCODE_PAUSE,
        minifb::Key::Space => keyboard_scancode_t::KEY_SCANCODE_SPACE,
        minifb::Key::Tab => keyboard_scancode_t::KEY_SCANCODE_TAB,
        minifb::Key::NumLock => keyboard_scancode_t::KEY_SCANCODE_NUMLOCKCLEAR,
        minifb::Key::CapsLock => keyboard_scancode_t::KEY_SCANCODE_CAPSLOCK,
        minifb::Key::ScrollLock => keyboard_scancode_t::KEY_SCANCODE_SCROLLLOCK,
        minifb::Key::LeftShift => keyboard_scancode_t::KEY_SCANCODE_LSHIFT,
        minifb::Key::RightShift => keyboard_scancode_t::KEY_SCANCODE_RSHIFT,
        minifb::Key::LeftCtrl => keyboard_scancode_t::KEY_SCANCODE_LCTRL,
        minifb::Key::RightCtrl => keyboard_scancode_t::KEY_SCANCODE_RCTRL,
        minifb::Key::NumPad0 => keyboard_scancode_t::KEY_SCANCODE_KP_0,
        minifb::Key::NumPad1 => keyboard_scancode_t::KEY_SCANCODE_KP_1,
        minifb::Key::NumPad2 => keyboard_scancode_t::KEY_SCANCODE_KP_2,
        minifb::Key::NumPad3 => keyboard_scancode_t::KEY_SCANCODE_KP_3,
        minifb::Key::NumPad4 => keyboard_scancode_t::KEY_SCANCODE_KP_4,
        minifb::Key::NumPad5 => keyboard_scancode_t::KEY_SCANCODE_KP_5,
        minifb::Key::NumPad6 => keyboard_scancode_t::KEY_SCANCODE_KP_6,
        minifb::Key::NumPad7 => keyboard_scancode_t::KEY_SCANCODE_KP_7,
        minifb::Key::NumPad8 => keyboard_scancode_t::KEY_SCANCODE_KP_8,
        minifb::Key::NumPad9 => keyboard_scancode_t::KEY_SCANCODE_KP_9,
        minifb::Key::NumPadDot => keyboard_scancode_t::KEY_SCANCODE_KP_PERIOD,
        minifb::Key::NumPadSlash => keyboard_scancode_t::KEY_SCANCODE_KP_DIVIDE,
        minifb::Key::NumPadAsterisk => keyboard_scancode_t::KEY_SCANCODE_KP_MULTIPLY,
        minifb::Key::NumPadMinus => keyboard_scancode_t::KEY_SCANCODE_KP_MINUS,
        minifb::Key::NumPadPlus => keyboard_scancode_t::KEY_SCANCODE_KP_PLUS,
        minifb::Key::NumPadEnter => keyboard_scancode_t::KEY_SCANCODE_KP_ENTER,
        minifb::Key::LeftAlt => keyboard_scancode_t::KEY_SCANCODE_LALT,
        minifb::Key::RightAlt => keyboard_scancode_t::KEY_SCANCODE_RALT,
        minifb::Key::LeftSuper => keyboard_scancode_t::KEY_SCANCODE_LGUI,
        minifb::Key::RightSuper => keyboard_scancode_t::KEY_SCANCODE_RGUI,
        minifb::Key::Unknown => keyboard_scancode_t::KEY_SCANCODE_UNKNOWN,
        minifb::Key::Count => keyboard_scancode_t::KEY_SCANCODE_UNKNOWN,
    }
}

#[derive(Debug)]
struct WindowData {
    window: minifb::Window,
    title: [u8; 128],
    buffer: Vec<u32>,
    buffer565: Vec<u16>,
    framebuffer: framebuffer_t,
    topmost: bool,
    undecorated: bool,
    fullscreen: bool,
}

unsafe impl Sync for WindowData {}
unsafe impl Send for WindowData {}

static WINDOWS: LazyLock<Arc<RwLock<Vec<Option<Arc<RwLock<WindowData>>>>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(Vec::new())));

const convert_565_to_8888: fn(u16) -> u32 = |color| {
    let r = ((color >> 11) & 0x1F) as u32 * 255 / 31;
    let g = ((color >> 5) & 0x3F) as u32 * 255 / 63;
    let b = (color & 0x1F) as u32 * 255 / 31;
    (r << 16) | (g << 8) | b
};

#[unsafe(no_mangle)]
pub extern "C" fn window_create(
    title: *const ::core::ffi::c_char,
    size: window_size_t,
    flags: window_flag_t,
) -> window_handle_t {
    let title_cstr = unsafe { std::ffi::CStr::from_ptr(title) };
    let title = title_cstr.to_string_lossy().into_owned();
    let title_len = title.len();
    let mut title_bytes = [0u8; 128];
    title_bytes[..title_len.min(127)].copy_from_slice(title.as_bytes());

    let buffer: Vec<u32> = vec![0; size.w as usize * size.h as usize];
    let mut buffer565: Vec<u16> = vec![0; size.w as usize * size.h as usize];

    let topmost =
        (flags & window_flag_t::WINDOW_FLAG_ALWAYS_ON_TOP) != window_flag_t::WINDOW_FLAG_NONE;
    let undecorated =
        (flags & window_flag_t::WINDOW_FLAG_UNDECORATED) != window_flag_t::WINDOW_FLAG_NONE;
    let fullscreen =
        (flags & window_flag_t::WINDOW_FLAG_FULLSCREEN) != window_flag_t::WINDOW_FLAG_NONE;

    let mut options = WindowOptions::default();
    options.borderless = undecorated;
    options.resize = false;
    options.scale = minifb::Scale::X1;
    options.transparency = false;
    options.title = true;
    options.none = false;
    options.scale_mode = minifb::ScaleMode::UpperLeft;
    options.topmost = topmost;

    let mut window = Window::new(&title, size.w as usize, size.h as usize, options).unwrap();
    window.set_target_fps(60);

    let framebuffer = framebuffer_t {
        w: size.w as u32,
        h: size.h as u32,
        format: pixel_format_t::BADGEVMS_PIXELFORMAT_RGB565,
        pixels: buffer565.as_mut_ptr(),
    };

    let window: WindowData = WindowData {
        window,
        title: title_bytes,
        buffer,
        buffer565,
        framebuffer: framebuffer,
        topmost,
        undecorated,
        fullscreen,
    };

    let mut windows = WINDOWS.write().unwrap();
    let index = windows.len();
    windows.push(Some(Arc::new(RwLock::new(window))));

    return index as window_handle_t;
}
#[unsafe(no_mangle)]
pub extern "C" fn window_framebuffer_create(
    window: window_handle_t,
    size: window_size_t,
    pixel_format: pixel_format_t,
) -> *mut framebuffer_t {
    if pixel_format != pixel_format_t::BADGEVMS_PIXELFORMAT_RGB565 {
        unimplemented!(
            "For now the native bindings only supports RGB565. Feel free to contribute support for your pixel format."
        );
    }

    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();

    let mut window_data = window_data.write().unwrap();
    assert_eq!(size.w as u32, window_data.framebuffer.w as u32);
    assert_eq!(size.h as u32, window_data.framebuffer.h as u32);
    return &mut window_data.framebuffer as *mut framebuffer_t;
}
#[unsafe(no_mangle)]
pub extern "C" fn window_destroy(window: window_handle_t) {
    let mut windows = WINDOWS.write().unwrap();
    let window_data = windows.get_mut(window as usize).unwrap();
    *window_data = None;
}
#[unsafe(no_mangle)]
pub extern "C" fn window_title_get(window: window_handle_t) -> *const ::core::ffi::c_char {
    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let window_data = window_data.read().unwrap();
    return window_data.title.as_ptr() as *const ::core::ffi::c_char;
}
#[unsafe(no_mangle)]
pub extern "C" fn window_title_set(window: window_handle_t, title: *const ::core::ffi::c_char) {
    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let mut window_data = window_data.write().unwrap();

    let title_cstr = unsafe { std::ffi::CStr::from_ptr(title) };
    let title = title_cstr.to_string_lossy().into_owned();
    let title_len = title.len();
    let mut title_bytes = [0u8; 128];
    title_bytes[..title_len.min(127)].copy_from_slice(title.as_bytes());

    window_data.title = title_bytes;
    window_data.window.set_title(&title);
}
#[unsafe(no_mangle)]
pub extern "C" fn window_position_get(window: window_handle_t) -> window_coords_t {
    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let window_data = window_data.read().unwrap();
    let position = window_data.window.get_position();
    return window_coords_t {
        x: position.0 as i32,
        y: position.1 as i32,
    };
}
#[unsafe(no_mangle)]
pub extern "C" fn window_position_set(
    window: window_handle_t,
    coords: window_coords_t,
) -> window_coords_t {
    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let mut window_data = window_data.write().unwrap();
    window_data
        .window
        .set_position(coords.x as isize, coords.y as isize);
    return coords;
}
#[unsafe(no_mangle)]
pub extern "C" fn window_size_get(window: window_handle_t) -> window_size_t {
    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let window_data = window_data.read().unwrap();
    let position = window_data.window.get_size();
    return window_size_t {
        w: position.0 as i32,
        h: position.1 as i32,
    };
}
#[unsafe(no_mangle)]
pub extern "C" fn window_size_set(window: window_handle_t, size: window_size_t) -> window_size_t {
    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let mut window_data = window_data.write().unwrap();
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn window_flags_get(window: window_handle_t) -> window_flag_t {
    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let window_data = window_data.read().unwrap();
    let mut flags: window_flag_t = window_flag_t::WINDOW_FLAG_NONE;
    if window_data.fullscreen {
        flags |= window_flag_t::WINDOW_FLAG_FULLSCREEN;
    }
    if window_data.topmost {
        flags |= window_flag_t::WINDOW_FLAG_ALWAYS_ON_TOP;
    }
    if window_data.undecorated {
        flags |= window_flag_t::WINDOW_FLAG_UNDECORATED;
    }
    flags
}
#[unsafe(no_mangle)]
pub extern "C" fn window_flags_set(window: window_handle_t, flags: window_flag_t) -> window_flag_t {
    let topmost =
        (flags & window_flag_t::WINDOW_FLAG_ALWAYS_ON_TOP) != window_flag_t::WINDOW_FLAG_NONE;
    // Cant change undecorated after creation
    // let undecorated =
    //     (flags & window_flag_t::WINDOW_FLAG_UNDECORATED) != window_flag_t::WINDOW_FLAG_NONE;
    let fullscreen =
        (flags & window_flag_t::WINDOW_FLAG_FULLSCREEN) != window_flag_t::WINDOW_FLAG_NONE;

    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let mut window_data = window_data.write().unwrap();

    window_data.topmost = topmost;
    window_data.fullscreen = fullscreen;
    window_data.window.topmost(topmost);
    // TODO: Implement fullscreen
    return flags;
}
#[unsafe(no_mangle)]
pub extern "C" fn window_framebuffer_size_get(window: window_handle_t) -> window_size_t {
    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let window_data = window_data.read().unwrap();
    return window_size_t {
        w: window_data.framebuffer.w as i32,
        h: window_data.framebuffer.h as i32,
    };
}
#[unsafe(no_mangle)]
pub extern "C" fn window_framebuffer_size_set(
    window: window_handle_t,
    size: window_size_t,
) -> window_size_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn window_framebuffer_format_get(window: window_handle_t) -> pixel_format_t {
    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let window_data = window_data.read().unwrap();
    return window_data.framebuffer.format;
}
#[unsafe(no_mangle)]
pub extern "C" fn window_framebuffer_get(window: window_handle_t) -> *mut framebuffer_t {
    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let mut window_data = window_data.write().unwrap();
    let framebuffer: *mut framebuffer_t = &mut window_data.framebuffer as *mut framebuffer_t;
    return framebuffer;
}
#[unsafe(no_mangle)]
pub extern "C" fn window_present(
    window: window_handle_t,
    _block: bool,
    _rects: *mut window_rect_t,
    _num_rects: ::core::ffi::c_int,
) {
    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let mut window_data = window_data.write().unwrap();
    let window_data_2: &mut WindowData = &mut window_data;

    let WindowData {
        window,
        buffer,
        buffer565,
        framebuffer,
        ..
    } = window_data_2;

    for (pixel_565, pixel_8888) in buffer565.iter_mut().zip(buffer.iter_mut()) {
        *pixel_8888 = convert_565_to_8888(*pixel_565);
    }

    window
        .update_with_buffer(&buffer, framebuffer.w as usize, framebuffer.h as usize)
        .unwrap();
}
#[unsafe(no_mangle)]
pub extern "C" fn window_event_poll(
    window: window_handle_t,
    block: bool,
    timeout_msec: u32,
) -> event_t {
    let windows = WINDOWS.read().unwrap();
    let window_data = windows.get(window as usize).unwrap().as_ref().unwrap();
    let window_data = window_data.read().unwrap();

    let mut time: timeval = timeval {
        tv_sec: 0,
        tv_usec: 0, // Convert milliseconds to microseconds
        __bindgen_padding_0: [0; 4],
    };
    let nullptr: *mut ::core::ffi::c_void = std::ptr::null_mut();
    unsafe { gettimeofday(&mut time as *mut timeval, nullptr) };
    let timestamp = (time.tv_sec as u64) * 1_000_000_000 + (time.tv_usec as u64) * 1_000;

    let native_events = window_data.window.get_keys_pressed(minifb::KeyRepeat::No);
    let Some(key) = native_events.first() else {
        let empty_event = keyboard_event_t {
            timestamp: 0,
            scancode: keyboard_scancode_t::KEY_SCANCODE_UNKNOWN,
            key: 0,
            mod_: 0,
            text: '0' as ::core::ffi::c_char,
            down: false,
            repeat: false,
            __bindgen_padding_0: [0u8; 3],
        };
        let empty_union = event_t__bindgen_ty_1 {
            keyboard: empty_event,
        };
        return event_t {
            type_: event_type_t::EVENT_NONE,
            __bindgen_padding_0: [0; 4],
            __bindgen_anon_1: empty_union,
        };
    };
    let scancode = minifb_key_to_scancode(key);
    let key_code = minifb_key_to_keycode(key);
    let text = minifb_key_to_char(key).unwrap_or('\0' as ::core::ffi::c_char);

    let keyboard_event = keyboard_event_t {
        timestamp: timestamp,
        scancode: scancode,
        key: key_code,
        mod_: 0,
        text: text,
        down: true,
        repeat: false,
        __bindgen_padding_0: [0u8; 3],
    };
    let event_union = event_t__bindgen_ty_1 {
        keyboard: keyboard_event,
    };
    let event = event_t {
        type_: event_type_t::EVENT_KEY_DOWN,
        __bindgen_padding_0: [0; 4],
        __bindgen_anon_1: event_union,
    };
    event
}

#[unsafe(no_mangle)]
pub extern "C" fn get_screen_info(
    width: *mut ::core::ffi::c_int,
    height: *mut ::core::ffi::c_int,
    format: *mut pixel_format_t,
    refresh_rate: *mut f32,
) {
    // TODO: Use real information
    unsafe {
        if !width.is_null() {
            *width = 700; // Width of the badge
        }
        if !height.is_null() {
            *height = 700; // Height of the badge
        }
        if !format.is_null() {
            *format = pixel_format_t::BADGEVMS_PIXELFORMAT_RGB565; // Only supported color format
        }
        if !refresh_rate.is_null() {
            *refresh_rate = 60.0; // Example refresh rate
        }
    }
}
