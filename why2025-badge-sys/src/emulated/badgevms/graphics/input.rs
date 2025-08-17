//! Helper functions for dealing with input in the emulated badge environment.

use crate::types::*;
use minifb::{InputCallback, Key};
use std::time::{SystemTime, UNIX_EPOCH};

impl Into<ModifierState> for Key {
    fn into(self) -> ModifierState {
        (&self).into()
    }
}
impl Into<ModifierState> for &Key {
    fn into(self) -> ModifierState {
        match self {
            Key::LeftShift => ModifierState::KMOD_LSHIFT,
            Key::RightShift => ModifierState::KMOD_RSHIFT,
            Key::LeftCtrl => ModifierState::KMOD_LCTRL,
            Key::RightCtrl => ModifierState::KMOD_RCTRL,
            Key::LeftAlt => ModifierState::KMOD_LALT,
            Key::RightAlt => ModifierState::KMOD_RALT,
            Key::LeftSuper => ModifierState::KMOD_LGUI,
            Key::RightSuper => ModifierState::KMOD_RGUI,
            Key::NumLock => ModifierState::KMOD_NUM,
            Key::CapsLock => ModifierState::KMOD_CAPS,
            Key::ScrollLock => ModifierState::KMOD_SCROLL,
            _ => ModifierState::KMOD_NONE,
        }
    }
}
pub fn minifb_key_to_scancode(key: &Key) -> keyboard_scancode_t {
    match key {
        Key::Key0 => keyboard_scancode_t::KEY_SCANCODE_0,
        Key::Key1 => keyboard_scancode_t::KEY_SCANCODE_1,
        Key::Key2 => keyboard_scancode_t::KEY_SCANCODE_2,
        Key::Key3 => keyboard_scancode_t::KEY_SCANCODE_3,
        Key::Key4 => keyboard_scancode_t::KEY_SCANCODE_4,
        Key::Key5 => keyboard_scancode_t::KEY_SCANCODE_5,
        Key::Key6 => keyboard_scancode_t::KEY_SCANCODE_6,
        Key::Key7 => keyboard_scancode_t::KEY_SCANCODE_7,
        Key::Key8 => keyboard_scancode_t::KEY_SCANCODE_8,
        Key::Key9 => keyboard_scancode_t::KEY_SCANCODE_9,
        Key::A => keyboard_scancode_t::KEY_SCANCODE_A,
        Key::B => keyboard_scancode_t::KEY_SCANCODE_B,
        Key::C => keyboard_scancode_t::KEY_SCANCODE_C,
        Key::D => keyboard_scancode_t::KEY_SCANCODE_D,
        Key::E => keyboard_scancode_t::KEY_SCANCODE_E,
        Key::F => keyboard_scancode_t::KEY_SCANCODE_F,
        Key::G => keyboard_scancode_t::KEY_SCANCODE_G,
        Key::H => keyboard_scancode_t::KEY_SCANCODE_H,
        Key::I => keyboard_scancode_t::KEY_SCANCODE_I,
        Key::J => keyboard_scancode_t::KEY_SCANCODE_J,
        Key::K => keyboard_scancode_t::KEY_SCANCODE_K,
        Key::L => keyboard_scancode_t::KEY_SCANCODE_L,
        Key::M => keyboard_scancode_t::KEY_SCANCODE_M,
        Key::N => keyboard_scancode_t::KEY_SCANCODE_N,
        Key::O => keyboard_scancode_t::KEY_SCANCODE_O,
        Key::P => keyboard_scancode_t::KEY_SCANCODE_P,
        Key::Q => keyboard_scancode_t::KEY_SCANCODE_Q,
        Key::R => keyboard_scancode_t::KEY_SCANCODE_R,
        Key::S => keyboard_scancode_t::KEY_SCANCODE_S,
        Key::T => keyboard_scancode_t::KEY_SCANCODE_T,
        Key::U => keyboard_scancode_t::KEY_SCANCODE_U,
        Key::V => keyboard_scancode_t::KEY_SCANCODE_V,
        Key::W => keyboard_scancode_t::KEY_SCANCODE_W,
        Key::X => keyboard_scancode_t::KEY_SCANCODE_X,
        Key::Y => keyboard_scancode_t::KEY_SCANCODE_Y,
        Key::Z => keyboard_scancode_t::KEY_SCANCODE_Z,
        Key::F1 => keyboard_scancode_t::KEY_SCANCODE_ESCAPE,
        Key::F2 => keyboard_scancode_t::KEY_SCANCODE_SQUARE,
        Key::F3 => keyboard_scancode_t::KEY_SCANCODE_TRIANGLE,
        Key::F4 => keyboard_scancode_t::KEY_SCANCODE_CROSS,
        Key::F5 => keyboard_scancode_t::KEY_SCANCODE_CIRCLE,
        Key::F6 => keyboard_scancode_t::KEY_SCANCODE_CLOUD,
        Key::F7 => keyboard_scancode_t::KEY_SCANCODE_DIAMOND,
        Key::F8 => keyboard_scancode_t::KEY_SCANCODE_BACKSPACE,
        Key::F9 => keyboard_scancode_t::KEY_SCANCODE_F9,
        Key::F10 => keyboard_scancode_t::KEY_SCANCODE_F10,
        Key::F11 => keyboard_scancode_t::KEY_SCANCODE_F11,
        Key::F12 => keyboard_scancode_t::KEY_SCANCODE_F12,
        Key::F13 => keyboard_scancode_t::KEY_SCANCODE_F13,
        Key::F14 => keyboard_scancode_t::KEY_SCANCODE_F14,
        Key::F15 => keyboard_scancode_t::KEY_SCANCODE_F15,
        Key::Down => keyboard_scancode_t::KEY_SCANCODE_DOWN,
        Key::Left => keyboard_scancode_t::KEY_SCANCODE_LEFT,
        Key::Right => keyboard_scancode_t::KEY_SCANCODE_RIGHT,
        Key::Up => keyboard_scancode_t::KEY_SCANCODE_UP,
        Key::Apostrophe => keyboard_scancode_t::KEY_SCANCODE_APOSTROPHE,
        Key::Backquote => keyboard_scancode_t::KEY_SCANCODE_GRAVE,
        Key::Backslash => keyboard_scancode_t::KEY_SCANCODE_BACKSLASH,
        Key::Comma => keyboard_scancode_t::KEY_SCANCODE_COMMA,
        Key::Equal => keyboard_scancode_t::KEY_SCANCODE_EQUALS,
        Key::LeftBracket => keyboard_scancode_t::KEY_SCANCODE_LEFTBRACKET,
        Key::Minus => keyboard_scancode_t::KEY_SCANCODE_MINUS,
        Key::Period => keyboard_scancode_t::KEY_SCANCODE_PERIOD,
        Key::RightBracket => keyboard_scancode_t::KEY_SCANCODE_RIGHTBRACKET,
        Key::Semicolon => keyboard_scancode_t::KEY_SCANCODE_SEMICOLON,
        Key::Slash => keyboard_scancode_t::KEY_SCANCODE_SLASH,
        Key::Backspace => keyboard_scancode_t::KEY_SCANCODE_BACKSPACE,
        Key::Delete => keyboard_scancode_t::KEY_SCANCODE_DELETE,
        Key::End => keyboard_scancode_t::KEY_SCANCODE_END,
        Key::Enter => keyboard_scancode_t::KEY_SCANCODE_RETURN,
        Key::Escape => keyboard_scancode_t::KEY_SCANCODE_ESCAPE,
        Key::Home => keyboard_scancode_t::KEY_SCANCODE_HOME,
        Key::Insert => keyboard_scancode_t::KEY_SCANCODE_INSERT,
        Key::Menu => keyboard_scancode_t::KEY_SCANCODE_MENU,
        Key::PageDown => keyboard_scancode_t::KEY_SCANCODE_PAGEDOWN,
        Key::PageUp => keyboard_scancode_t::KEY_SCANCODE_PAGEUP,
        Key::Pause => keyboard_scancode_t::KEY_SCANCODE_PAUSE,
        Key::Space => keyboard_scancode_t::KEY_SCANCODE_SPACE,
        Key::Tab => keyboard_scancode_t::KEY_SCANCODE_TAB,
        Key::NumLock => keyboard_scancode_t::KEY_SCANCODE_NUMLOCKCLEAR,
        Key::CapsLock => keyboard_scancode_t::KEY_SCANCODE_CAPSLOCK,
        Key::ScrollLock => keyboard_scancode_t::KEY_SCANCODE_SCROLLLOCK,
        Key::LeftShift => keyboard_scancode_t::KEY_SCANCODE_LSHIFT,
        Key::RightShift => keyboard_scancode_t::KEY_SCANCODE_RSHIFT,
        Key::LeftCtrl => keyboard_scancode_t::KEY_SCANCODE_LCTRL,
        Key::RightCtrl => keyboard_scancode_t::KEY_SCANCODE_RCTRL,
        Key::NumPad0 => keyboard_scancode_t::KEY_SCANCODE_KP_0,
        Key::NumPad1 => keyboard_scancode_t::KEY_SCANCODE_KP_1,
        Key::NumPad2 => keyboard_scancode_t::KEY_SCANCODE_KP_2,
        Key::NumPad3 => keyboard_scancode_t::KEY_SCANCODE_KP_3,
        Key::NumPad4 => keyboard_scancode_t::KEY_SCANCODE_KP_4,
        Key::NumPad5 => keyboard_scancode_t::KEY_SCANCODE_KP_5,
        Key::NumPad6 => keyboard_scancode_t::KEY_SCANCODE_KP_6,
        Key::NumPad7 => keyboard_scancode_t::KEY_SCANCODE_KP_7,
        Key::NumPad8 => keyboard_scancode_t::KEY_SCANCODE_KP_8,
        Key::NumPad9 => keyboard_scancode_t::KEY_SCANCODE_KP_9,
        Key::NumPadDot => keyboard_scancode_t::KEY_SCANCODE_KP_PERIOD,
        Key::NumPadSlash => keyboard_scancode_t::KEY_SCANCODE_KP_DIVIDE,
        Key::NumPadAsterisk => keyboard_scancode_t::KEY_SCANCODE_KP_MULTIPLY,
        Key::NumPadMinus => keyboard_scancode_t::KEY_SCANCODE_KP_MINUS,
        Key::NumPadPlus => keyboard_scancode_t::KEY_SCANCODE_KP_PLUS,
        Key::NumPadEnter => keyboard_scancode_t::KEY_SCANCODE_KP_ENTER,
        Key::LeftAlt => keyboard_scancode_t::KEY_SCANCODE_LALT,
        Key::RightAlt => keyboard_scancode_t::KEY_SCANCODE_RALT,
        Key::LeftSuper => keyboard_scancode_t::KEY_SCANCODE_LGUI,
        Key::RightSuper => keyboard_scancode_t::KEY_SCANCODE_RGUI,
        Key::Unknown => keyboard_scancode_t::KEY_SCANCODE_UNKNOWN,
        Key::Count => keyboard_scancode_t::KEY_SCANCODE_UNKNOWN,
    }
}

const fn keyboard_layout(scancode: keyboard_scancode_t) -> (std::ffi::c_char, std::ffi::c_char) {
    // Ported from badgevms/include/badgevms/keymap_us.h
    match scancode {
        keyboard_scancode_t::KEY_SCANCODE_UNKNOWN => (0, 0),

        // Letters
        keyboard_scancode_t::KEY_SCANCODE_A => ('a' as std::ffi::c_char, 'A' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_B => ('b' as std::ffi::c_char, 'B' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_C => ('c' as std::ffi::c_char, 'C' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_D => ('d' as std::ffi::c_char, 'D' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_E => ('e' as std::ffi::c_char, 'E' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_F => ('f' as std::ffi::c_char, 'F' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_G => ('g' as std::ffi::c_char, 'G' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_H => ('h' as std::ffi::c_char, 'H' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_I => ('i' as std::ffi::c_char, 'I' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_J => ('j' as std::ffi::c_char, 'J' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_K => ('k' as std::ffi::c_char, 'K' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_L => ('l' as std::ffi::c_char, 'L' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_M => ('m' as std::ffi::c_char, 'M' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_N => ('n' as std::ffi::c_char, 'N' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_O => ('o' as std::ffi::c_char, 'O' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_P => ('p' as std::ffi::c_char, 'P' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_Q => ('q' as std::ffi::c_char, 'Q' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_R => ('r' as std::ffi::c_char, 'R' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_S => ('s' as std::ffi::c_char, 'S' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_T => ('t' as std::ffi::c_char, 'T' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_U => ('u' as std::ffi::c_char, 'U' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_V => ('v' as std::ffi::c_char, 'V' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_W => ('w' as std::ffi::c_char, 'W' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_X => ('x' as std::ffi::c_char, 'X' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_Y => ('y' as std::ffi::c_char, 'Y' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_Z => ('z' as std::ffi::c_char, 'Z' as std::ffi::c_char),

        // Numbers
        keyboard_scancode_t::KEY_SCANCODE_1 => ('1' as std::ffi::c_char, '!' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_2 => ('2' as std::ffi::c_char, '@' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_3 => ('3' as std::ffi::c_char, '#' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_4 => ('4' as std::ffi::c_char, '$' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_5 => ('5' as std::ffi::c_char, '%' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_6 => ('6' as std::ffi::c_char, '^' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_7 => ('7' as std::ffi::c_char, '&' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_8 => ('8' as std::ffi::c_char, '*' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_9 => ('9' as std::ffi::c_char, '(' as std::ffi::c_char),
        keyboard_scancode_t::KEY_SCANCODE_0 => ('0' as std::ffi::c_char, ')' as std::ffi::c_char),

        // Control keys
        keyboard_scancode_t::KEY_SCANCODE_RETURN => {
            ('\r' as std::ffi::c_char, '\r' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_ESCAPE => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_BACKSPACE => {
            ('\x08' as std::ffi::c_char, '\x08' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_TAB => {
            ('\t' as std::ffi::c_char, '\t' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_SPACE => {
            (' ' as std::ffi::c_char, ' ' as std::ffi::c_char)
        }

        // Punctuation
        keyboard_scancode_t::KEY_SCANCODE_MINUS => {
            ('-' as std::ffi::c_char, '_' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_EQUALS => {
            ('=' as std::ffi::c_char, '+' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_LEFTBRACKET => {
            ('[' as std::ffi::c_char, '{' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_RIGHTBRACKET => {
            (']' as std::ffi::c_char, '}' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_BACKSLASH => {
            ('\\' as std::ffi::c_char, '|' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_NONUSHASH => {
            ('\\' as std::ffi::c_char, '|' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_SEMICOLON => {
            (';' as std::ffi::c_char, ':' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_APOSTROPHE => {
            ('\'' as std::ffi::c_char, '"' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_GRAVE => {
            ('`' as std::ffi::c_char, '~' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_COMMA => {
            (',' as std::ffi::c_char, '<' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_PERIOD => {
            ('.' as std::ffi::c_char, '>' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_SLASH => {
            ('/' as std::ffi::c_char, '?' as std::ffi::c_char)
        }

        // Lock keys
        keyboard_scancode_t::KEY_SCANCODE_CAPSLOCK => (0, 0),

        // Function keys
        keyboard_scancode_t::KEY_SCANCODE_F1 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_F2 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_F3 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_F4 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_F5 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_F6 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_F7 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_F8 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_F9 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_F10 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_F11 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_F12 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_F13 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_F14 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_F15 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_F16 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_F17 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_F18 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_F19 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_F20 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_F21 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_F22 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_F23 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_F24 => (0, 0),

        // Navigation keys
        keyboard_scancode_t::KEY_SCANCODE_PRINTSCREEN => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_SCROLLLOCK => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_PAUSE => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_INSERT => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_HOME => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_PAGEUP => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_DELETE => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_END => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_PAGEDOWN => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_RIGHT => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_LEFT => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_DOWN => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_UP => (0, 0),

        // Keypad
        keyboard_scancode_t::KEY_SCANCODE_NUMLOCKCLEAR => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_KP_DIVIDE => {
            ('/' as std::ffi::c_char, '/' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_MULTIPLY => {
            ('*' as std::ffi::c_char, '*' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_MINUS => {
            ('-' as std::ffi::c_char, '-' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_PLUS => {
            ('+' as std::ffi::c_char, '+' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_ENTER => {
            ('\r' as std::ffi::c_char, '\r' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_1 => {
            ('1' as std::ffi::c_char, '1' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_2 => {
            ('2' as std::ffi::c_char, '2' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_3 => {
            ('3' as std::ffi::c_char, '3' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_4 => {
            ('4' as std::ffi::c_char, '4' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_5 => {
            ('5' as std::ffi::c_char, '5' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_6 => {
            ('6' as std::ffi::c_char, '6' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_7 => {
            ('7' as std::ffi::c_char, '7' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_8 => {
            ('8' as std::ffi::c_char, '8' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_9 => {
            ('9' as std::ffi::c_char, '9' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_0 => {
            ('0' as std::ffi::c_char, '0' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_PERIOD => {
            ('.' as std::ffi::c_char, '.' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_COMMA => {
            (',' as std::ffi::c_char, ',' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_EQUALS => {
            ('=' as std::ffi::c_char, '=' as std::ffi::c_char)
        }

        // ISO keyboard additional key
        keyboard_scancode_t::KEY_SCANCODE_NONUSBACKSLASH => {
            ('\\' as std::ffi::c_char, '|' as std::ffi::c_char)
        }

        // Other keys - non-printable
        keyboard_scancode_t::KEY_SCANCODE_APPLICATION => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_POWER => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_EXECUTE => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_HELP => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_MENU => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_SELECT => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_STOP => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_AGAIN => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_UNDO => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_CUT => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_COPY => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_PASTE => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_FIND => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_MUTE => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_VOLUMEUP => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_VOLUMEDOWN => (0, 0),

        // Additional keypad keys and modifiers
        keyboard_scancode_t::KEY_SCANCODE_KP_EQUALSAS400 => {
            ('=' as std::ffi::c_char, '=' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_INTERNATIONAL1 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_INTERNATIONAL2 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_INTERNATIONAL3 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_INTERNATIONAL4 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_INTERNATIONAL5 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_INTERNATIONAL6 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_INTERNATIONAL7 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_INTERNATIONAL8 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_INTERNATIONAL9 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_LANG1 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_LANG2 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_LANG3 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_LANG4 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_LANG5 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_LANG6 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_LANG7 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_LANG8 => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_LANG9 => (0, 0),

        // System keys
        keyboard_scancode_t::KEY_SCANCODE_ALTERASE => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_SYSREQ => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_CANCEL => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_CLEAR => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_PRIOR => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_RETURN2 => {
            ('\r' as std::ffi::c_char, '\r' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_SEPARATOR => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_OUT => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_OPER => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_CLEARAGAIN => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_CRSEL => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_EXSEL => (0, 0),

        // Additional keypad functions
        keyboard_scancode_t::KEY_SCANCODE_KP_00 => {
            ('0' as std::ffi::c_char, '0' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_000 => {
            ('0' as std::ffi::c_char, '0' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_THOUSANDSSEPARATOR => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_DECIMALSEPARATOR => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_CURRENCYUNIT => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_CURRENCYSUBUNIT => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_KP_LEFTPAREN => {
            ('(' as std::ffi::c_char, '(' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_RIGHTPAREN => {
            (')' as std::ffi::c_char, ')' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_LEFTBRACE => {
            ('{' as std::ffi::c_char, '{' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_RIGHTBRACE => {
            ('}' as std::ffi::c_char, '}' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_TAB => {
            ('\t' as std::ffi::c_char, '\t' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_BACKSPACE => {
            ('\x08' as std::ffi::c_char, '\x08' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_A => {
            ('a' as std::ffi::c_char, 'A' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_B => {
            ('b' as std::ffi::c_char, 'B' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_C => {
            ('c' as std::ffi::c_char, 'C' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_D => {
            ('d' as std::ffi::c_char, 'D' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_E => {
            ('e' as std::ffi::c_char, 'E' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_F => {
            ('f' as std::ffi::c_char, 'F' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_XOR => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_KP_POWER => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_KP_PERCENT => {
            ('%' as std::ffi::c_char, '%' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_LESS => {
            ('<' as std::ffi::c_char, '<' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_GREATER => {
            ('>' as std::ffi::c_char, '>' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_AMPERSAND => {
            ('&' as std::ffi::c_char, '&' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_DBLAMPERSAND => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_KP_VERTICALBAR => {
            ('|' as std::ffi::c_char, '|' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_DBLVERTICALBAR => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_KP_COLON => {
            (':' as std::ffi::c_char, ':' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_HASH => {
            ('#' as std::ffi::c_char, '#' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_SPACE => {
            (' ' as std::ffi::c_char, ' ' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_AT => {
            ('@' as std::ffi::c_char, '@' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_EXCLAM => {
            ('!' as std::ffi::c_char, '!' as std::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_KP_MEMSTORE => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_KP_MEMRECALL => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_KP_MEMCLEAR => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_KP_MEMADD => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_KP_MEMSUBTRACT => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_KP_MEMMULTIPLY => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_KP_MEMDIVIDE => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_KP_PLUSMINUS => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_KP_CLEAR => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_KP_CLEARENTRY => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_KP_BINARY => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_KP_OCTAL => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_KP_DECIMAL => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_KP_HEXADECIMAL => (0, 0),

        // Modifier keys
        keyboard_scancode_t::KEY_SCANCODE_LCTRL => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_LSHIFT => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_LALT => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_LGUI => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_RCTRL => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_RSHIFT => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_RALT => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_RGUI => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_MODE => (0, 0),

        // Media keys
        keyboard_scancode_t::KEY_SCANCODE_SLEEP => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_WAKE => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_CHANNEL_INCREMENT => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_CHANNEL_DECREMENT => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_MEDIA_PLAY => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_MEDIA_PAUSE => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_MEDIA_RECORD => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_MEDIA_FAST_FORWARD => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_MEDIA_REWIND => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_MEDIA_NEXT_TRACK => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_MEDIA_PREVIOUS_TRACK => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_MEDIA_STOP => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_MEDIA_EJECT => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_MEDIA_PLAY_PAUSE => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_MEDIA_SELECT => (0, 0),

        // Application control keys
        keyboard_scancode_t::KEY_SCANCODE_AC_NEW => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_AC_OPEN => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_AC_CLOSE => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_AC_EXIT => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_AC_SAVE => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_AC_PRINT => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_AC_PROPERTIES => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_AC_SEARCH => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_AC_HOME => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_AC_BACK => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_AC_FORWARD => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_AC_STOP => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_AC_REFRESH => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_AC_BOOKMARKS => (0, 0),

        // Mobile device keys
        keyboard_scancode_t::KEY_SCANCODE_SOFTLEFT => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_SOFTRIGHT => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_CALL => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_ENDCALL => (0, 0),

        // Special keys
        keyboard_scancode_t::KEY_SCANCODE_FN => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_SQUARE => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_TRIANGLE => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_CROSS => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_CIRCLE => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_CLOUD => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_DIAMOND => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_RESERVED => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_COUNT => (0, 0),
    }
}

const fn resolve_typed_character(
    scancode: keyboard_scancode_t,
    modifiers: ModifierState,
) -> std::ffi::c_char {
    let (lower, upper) = keyboard_layout(scancode);
    if modifiers.0 & ModifierState::KMOD_SHIFT.0 != ModifierState::KMOD_NONE.0 {
        upper
    } else {
        lower
    }
}

const fn scancode_to_keycode(scancode: keyboard_scancode_t) -> key_code_t {
    scancode as u32 | (1 << 30)
}

impl event_t {
    pub fn new_empty() -> Self {
        // If no event is available, return an empty event
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
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct ModifierState(pub u16);
impl ModifierState {
    /// No modifier is applicable.
    pub const KMOD_NONE: ModifierState = ModifierState(BADGEVMS_KMOD_NONE as u16);
    /// The left Shift key is down.
    pub const KMOD_LSHIFT: ModifierState = ModifierState(BADGEVMS_KMOD_LSHIFT as u16);
    /// The right Shift key is down.
    pub const KMOD_RSHIFT: ModifierState = ModifierState(BADGEVMS_KMOD_RSHIFT as u16);
    /// The Level 5 Shift key is down.
    pub const KMOD_LEVEL5: ModifierState = ModifierState(BADGEVMS_KMOD_LEVEL5 as u16);
    /// The left Ctrl (Control) key is down.
    pub const KMOD_LCTRL: ModifierState = ModifierState(BADGEVMS_KMOD_LCTRL as u16);
    /// The right Ctrl (Control) key is down.
    pub const KMOD_RCTRL: ModifierState = ModifierState(BADGEVMS_KMOD_RCTRL as u16);
    /// The left Alt key is down.
    pub const KMOD_LALT: ModifierState = ModifierState(BADGEVMS_KMOD_LALT as u16);
    /// The right Alt key is down.
    pub const KMOD_RALT: ModifierState = ModifierState(BADGEVMS_KMOD_RALT as u16);
    /// The left GUI key (often the Windows key) is down.
    pub const KMOD_LGUI: ModifierState = ModifierState(BADGEVMS_KMOD_LGUI as u16);
    /// The right GUI key (often the Windows key) is down.
    pub const KMOD_RGUI: ModifierState = ModifierState(BADGEVMS_KMOD_RGUI as u16);
    /// The Num Lock key (may be located on an extended keypad) is down.
    pub const KMOD_NUM: ModifierState = ModifierState(BADGEVMS_KMOD_NUM as u16);
    /// The Caps Lock key is down.
    pub const KMOD_CAPS: ModifierState = ModifierState(BADGEVMS_KMOD_CAPS as u16);
    /// The !AltGr key is down.
    pub const KMOD_MODE: ModifierState = ModifierState(BADGEVMS_KMOD_MODE as u16);
    /// The Scroll Lock key is down.
    pub const KMOD_SCROLL: ModifierState = ModifierState(BADGEVMS_KMOD_SCROLL as u16);
    /// Any Ctrl key is down.
    pub const KMOD_CTRL: ModifierState = ModifierState(BADGEVMS_KMOD_CTRL as u16);
    /// Any Shift key is down.
    pub const KMOD_SHIFT: ModifierState = ModifierState(BADGEVMS_KMOD_SHIFT as u16);
    /// Any Alt key is down.
    pub const KMOD_ALT: ModifierState = ModifierState(BADGEVMS_KMOD_ALT as u16);
    /// Any GUI key is down.
    pub const KMOD_GUI: ModifierState = ModifierState(BADGEVMS_KMOD_GUI as u16);
}
impl ::core::ops::BitOr<ModifierState> for ModifierState {
    type Output = Self;
    #[inline]
    fn bitor(self, other: Self) -> Self {
        ModifierState(self.0 | other.0)
    }
}
impl ::core::ops::BitOrAssign for ModifierState {
    #[inline]
    fn bitor_assign(&mut self, rhs: ModifierState) {
        self.0 |= rhs.0;
    }
}
impl ::core::ops::BitAnd<ModifierState> for ModifierState {
    type Output = Self;
    #[inline]
    fn bitand(self, other: Self) -> Self {
        ModifierState(self.0 & other.0)
    }
}
impl ::core::ops::BitAndAssign for ModifierState {
    #[inline]
    fn bitand_assign(&mut self, rhs: ModifierState) {
        self.0 &= rhs.0;
    }
}
impl ::core::ops::Not for ModifierState {
    type Output = Self;
    #[inline]
    fn not(self) -> Self {
        ModifierState(!self.0)
    }
}

pub struct WindowEventCallback {
    sender: std::sync::mpsc::Sender<event_t>,
    modifiers: ModifierState,
}
impl WindowEventCallback {
    pub fn new(sender: std::sync::mpsc::Sender<event_t>) -> Self {
        Self {
            sender,
            modifiers: ModifierState(0),
        }
    }
    fn update_modifier_state(&mut self, key: Key, is_down: bool) {
        let modifier = key.into();
        if is_down {
            self.modifiers |= modifier
        } else {
            self.modifiers &= !modifier
        }
    }
}

impl InputCallback for WindowEventCallback {
    fn add_char(&mut self, _uni_char: u32) {
        // We do our own key state handling in set_key_state.
        //
        // This is because with this function we can only get unicode characters,
        // but that doesn't include control characters like Shift, Ctrl, etc.
        //
        // This comes with the drawback that we might get funny sideeffects, if
        // the user presses shift and then focusses or unfocuses the window.
        // However I think the tradeoff is worth it
    }

    fn set_key_state(&mut self, key: Key, is_down: bool) {
        self.update_modifier_state(key, is_down);

        let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let now = since_epoch.as_micros() as u64;
        let scancode = minifb_key_to_scancode(&key);
        // In badgevms this is always just the processed scancode
        let key_code: key_code_t = scancode_to_keycode(scancode);
        // For button up/down events we never use text.
        let text = resolve_typed_character(scancode, self.modifiers);

        let keyboard_event = keyboard_event_t {
            // Actually in micros even though the documentation says nanos
            timestamp: now,
            scancode: scancode,
            key: key_code,
            mod_: self.modifiers.0,
            text: text,
            down: is_down,
            repeat: false,
            __bindgen_padding_0: [0u8; 3],
        };
        let event_union = event_t__bindgen_ty_1 {
            keyboard: keyboard_event,
        };
        let event_type = if is_down {
            event_type_t::EVENT_KEY_DOWN
        } else {
            event_type_t::EVENT_KEY_UP
        };
        let event = event_t {
            type_: event_type,
            __bindgen_padding_0: [0; 4],
            __bindgen_anon_1: event_union,
        };
        self.sender.send(event).unwrap();
    }
}
