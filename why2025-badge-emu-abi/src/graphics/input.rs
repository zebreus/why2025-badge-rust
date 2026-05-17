use core::ffi::c_ulong;

use x11_dl::keysym::*;

use crate::types::*;

const fn keyboard_layout(
    scancode: keyboard_scancode_t,
) -> (::core::ffi::c_char, ::core::ffi::c_char) {
    match scancode {
        keyboard_scancode_t::KEY_SCANCODE_UNKNOWN => (0, 0),
        keyboard_scancode_t::KEY_SCANCODE_A => {
            ('a' as ::core::ffi::c_char, 'A' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_B => {
            ('b' as ::core::ffi::c_char, 'B' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_C => {
            ('c' as ::core::ffi::c_char, 'C' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_D => {
            ('d' as ::core::ffi::c_char, 'D' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_E => {
            ('e' as ::core::ffi::c_char, 'E' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_F => {
            ('f' as ::core::ffi::c_char, 'F' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_G => {
            ('g' as ::core::ffi::c_char, 'G' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_H => {
            ('h' as ::core::ffi::c_char, 'H' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_I => {
            ('i' as ::core::ffi::c_char, 'I' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_J => {
            ('j' as ::core::ffi::c_char, 'J' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_K => {
            ('k' as ::core::ffi::c_char, 'K' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_L => {
            ('l' as ::core::ffi::c_char, 'L' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_M => {
            ('m' as ::core::ffi::c_char, 'M' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_N => {
            ('n' as ::core::ffi::c_char, 'N' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_O => {
            ('o' as ::core::ffi::c_char, 'O' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_P => {
            ('p' as ::core::ffi::c_char, 'P' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_Q => {
            ('q' as ::core::ffi::c_char, 'Q' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_R => {
            ('r' as ::core::ffi::c_char, 'R' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_S => {
            ('s' as ::core::ffi::c_char, 'S' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_T => {
            ('t' as ::core::ffi::c_char, 'T' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_U => {
            ('u' as ::core::ffi::c_char, 'U' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_V => {
            ('v' as ::core::ffi::c_char, 'V' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_W => {
            ('w' as ::core::ffi::c_char, 'W' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_X => {
            ('x' as ::core::ffi::c_char, 'X' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_Y => {
            ('y' as ::core::ffi::c_char, 'Y' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_Z => {
            ('z' as ::core::ffi::c_char, 'Z' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_1 => {
            ('1' as ::core::ffi::c_char, '!' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_2 => {
            ('2' as ::core::ffi::c_char, '@' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_3 => {
            ('3' as ::core::ffi::c_char, '#' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_4 => {
            ('4' as ::core::ffi::c_char, '$' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_5 => {
            ('5' as ::core::ffi::c_char, '%' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_6 => {
            ('6' as ::core::ffi::c_char, '^' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_7 => {
            ('7' as ::core::ffi::c_char, '&' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_8 => {
            ('8' as ::core::ffi::c_char, '*' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_9 => {
            ('9' as ::core::ffi::c_char, '(' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_0 => {
            ('0' as ::core::ffi::c_char, ')' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_RETURN => {
            ('\r' as ::core::ffi::c_char, '\r' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_BACKSPACE => {
            ('\x08' as ::core::ffi::c_char, '\x08' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_TAB => {
            ('\t' as ::core::ffi::c_char, '\t' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_SPACE => {
            (' ' as ::core::ffi::c_char, ' ' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_MINUS => {
            ('-' as ::core::ffi::c_char, '_' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_EQUALS => {
            ('=' as ::core::ffi::c_char, '+' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_LEFTBRACKET => {
            ('[' as ::core::ffi::c_char, '{' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_RIGHTBRACKET => {
            (']' as ::core::ffi::c_char, '}' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_BACKSLASH => {
            ('\\' as ::core::ffi::c_char, '|' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_SEMICOLON => {
            (';' as ::core::ffi::c_char, ':' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_APOSTROPHE => {
            ('\'' as ::core::ffi::c_char, '"' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_GRAVE => {
            ('`' as ::core::ffi::c_char, '~' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_COMMA => {
            (',' as ::core::ffi::c_char, '<' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_PERIOD => {
            ('.' as ::core::ffi::c_char, '>' as ::core::ffi::c_char)
        }
        keyboard_scancode_t::KEY_SCANCODE_SLASH => {
            ('/' as ::core::ffi::c_char, '?' as ::core::ffi::c_char)
        }
        _ => (0, 0),
    }
}

pub(crate) const fn resolve_typed_character(
    scancode: keyboard_scancode_t,
    modifiers: ModifierState,
) -> ::core::ffi::c_char {
    let (lower, upper) = keyboard_layout(scancode);
    if modifiers.0 & ModifierState::KMOD_SHIFT.0 != ModifierState::KMOD_NONE.0 {
        upper
    } else {
        lower
    }
}

pub(crate) const fn scancode_to_keycode(scancode: keyboard_scancode_t) -> key_code_t {
    scancode as u32 | (1 << 30)
}

pub(crate) const fn scancode_from_x11_keysym(keysym: c_ulong) -> Option<keyboard_scancode_t> {
    match keysym as u32 {
        XK_0 => Some(keyboard_scancode_t::KEY_SCANCODE_0),
        XK_1 => Some(keyboard_scancode_t::KEY_SCANCODE_1),
        XK_2 => Some(keyboard_scancode_t::KEY_SCANCODE_2),
        XK_3 => Some(keyboard_scancode_t::KEY_SCANCODE_3),
        XK_4 => Some(keyboard_scancode_t::KEY_SCANCODE_4),
        XK_5 => Some(keyboard_scancode_t::KEY_SCANCODE_5),
        XK_6 => Some(keyboard_scancode_t::KEY_SCANCODE_6),
        XK_7 => Some(keyboard_scancode_t::KEY_SCANCODE_7),
        XK_8 => Some(keyboard_scancode_t::KEY_SCANCODE_8),
        XK_9 => Some(keyboard_scancode_t::KEY_SCANCODE_9),

        XK_a => Some(keyboard_scancode_t::KEY_SCANCODE_A),
        XK_b => Some(keyboard_scancode_t::KEY_SCANCODE_B),
        XK_c => Some(keyboard_scancode_t::KEY_SCANCODE_C),
        XK_d => Some(keyboard_scancode_t::KEY_SCANCODE_D),
        XK_e => Some(keyboard_scancode_t::KEY_SCANCODE_E),
        XK_f => Some(keyboard_scancode_t::KEY_SCANCODE_F),
        XK_g => Some(keyboard_scancode_t::KEY_SCANCODE_G),
        XK_h => Some(keyboard_scancode_t::KEY_SCANCODE_H),
        XK_i => Some(keyboard_scancode_t::KEY_SCANCODE_I),
        XK_j => Some(keyboard_scancode_t::KEY_SCANCODE_J),
        XK_k => Some(keyboard_scancode_t::KEY_SCANCODE_K),
        XK_l => Some(keyboard_scancode_t::KEY_SCANCODE_L),
        XK_m => Some(keyboard_scancode_t::KEY_SCANCODE_M),
        XK_n => Some(keyboard_scancode_t::KEY_SCANCODE_N),
        XK_o => Some(keyboard_scancode_t::KEY_SCANCODE_O),
        XK_p => Some(keyboard_scancode_t::KEY_SCANCODE_P),
        XK_q => Some(keyboard_scancode_t::KEY_SCANCODE_Q),
        XK_r => Some(keyboard_scancode_t::KEY_SCANCODE_R),
        XK_s => Some(keyboard_scancode_t::KEY_SCANCODE_S),
        XK_t => Some(keyboard_scancode_t::KEY_SCANCODE_T),
        XK_u => Some(keyboard_scancode_t::KEY_SCANCODE_U),
        XK_v => Some(keyboard_scancode_t::KEY_SCANCODE_V),
        XK_w => Some(keyboard_scancode_t::KEY_SCANCODE_W),
        XK_x => Some(keyboard_scancode_t::KEY_SCANCODE_X),
        XK_y => Some(keyboard_scancode_t::KEY_SCANCODE_Y),
        XK_z => Some(keyboard_scancode_t::KEY_SCANCODE_Z),

        XK_apostrophe => Some(keyboard_scancode_t::KEY_SCANCODE_APOSTROPHE),
        XK_grave => Some(keyboard_scancode_t::KEY_SCANCODE_GRAVE),
        XK_backslash => Some(keyboard_scancode_t::KEY_SCANCODE_BACKSLASH),
        XK_comma => Some(keyboard_scancode_t::KEY_SCANCODE_COMMA),
        XK_equal => Some(keyboard_scancode_t::KEY_SCANCODE_EQUALS),
        XK_bracketleft => Some(keyboard_scancode_t::KEY_SCANCODE_LEFTBRACKET),
        XK_minus => Some(keyboard_scancode_t::KEY_SCANCODE_MINUS),
        XK_period => Some(keyboard_scancode_t::KEY_SCANCODE_PERIOD),
        XK_bracketright => Some(keyboard_scancode_t::KEY_SCANCODE_RIGHTBRACKET),
        XK_semicolon => Some(keyboard_scancode_t::KEY_SCANCODE_SEMICOLON),
        XK_slash => Some(keyboard_scancode_t::KEY_SCANCODE_SLASH),
        XK_space => Some(keyboard_scancode_t::KEY_SCANCODE_SPACE),

        XK_F1 => Some(keyboard_scancode_t::KEY_SCANCODE_ESCAPE),
        XK_F2 => Some(keyboard_scancode_t::KEY_SCANCODE_SQUARE),
        XK_F3 => Some(keyboard_scancode_t::KEY_SCANCODE_TRIANGLE),
        XK_F4 => Some(keyboard_scancode_t::KEY_SCANCODE_CROSS),
        XK_F5 => Some(keyboard_scancode_t::KEY_SCANCODE_CIRCLE),
        XK_F6 => Some(keyboard_scancode_t::KEY_SCANCODE_CLOUD),
        XK_F7 => Some(keyboard_scancode_t::KEY_SCANCODE_DIAMOND),
        XK_F8 => Some(keyboard_scancode_t::KEY_SCANCODE_BACKSPACE),
        XK_F9 => Some(keyboard_scancode_t::KEY_SCANCODE_F9),
        XK_F10 => Some(keyboard_scancode_t::KEY_SCANCODE_F10),
        XK_F11 => Some(keyboard_scancode_t::KEY_SCANCODE_F11),
        XK_F12 => Some(keyboard_scancode_t::KEY_SCANCODE_F12),
        XK_F13 => Some(keyboard_scancode_t::KEY_SCANCODE_F13),
        XK_F14 => Some(keyboard_scancode_t::KEY_SCANCODE_F14),
        XK_F15 => Some(keyboard_scancode_t::KEY_SCANCODE_F15),

        XK_Down => Some(keyboard_scancode_t::KEY_SCANCODE_DOWN),
        XK_Left => Some(keyboard_scancode_t::KEY_SCANCODE_LEFT),
        XK_Right => Some(keyboard_scancode_t::KEY_SCANCODE_RIGHT),
        XK_Up => Some(keyboard_scancode_t::KEY_SCANCODE_UP),
        XK_Escape => Some(keyboard_scancode_t::KEY_SCANCODE_ESCAPE),
        XK_BackSpace => Some(keyboard_scancode_t::KEY_SCANCODE_BACKSPACE),
        XK_Delete => Some(keyboard_scancode_t::KEY_SCANCODE_DELETE),
        XK_End => Some(keyboard_scancode_t::KEY_SCANCODE_END),
        XK_Return => Some(keyboard_scancode_t::KEY_SCANCODE_RETURN),
        XK_Home => Some(keyboard_scancode_t::KEY_SCANCODE_HOME),
        XK_Insert => Some(keyboard_scancode_t::KEY_SCANCODE_INSERT),
        XK_Menu => Some(keyboard_scancode_t::KEY_SCANCODE_MENU),
        XK_Page_Down => Some(keyboard_scancode_t::KEY_SCANCODE_PAGEDOWN),
        XK_Page_Up => Some(keyboard_scancode_t::KEY_SCANCODE_PAGEUP),
        XK_Pause => Some(keyboard_scancode_t::KEY_SCANCODE_PAUSE),
        XK_Tab => Some(keyboard_scancode_t::KEY_SCANCODE_TAB),
        XK_Num_Lock => Some(keyboard_scancode_t::KEY_SCANCODE_NUMLOCKCLEAR),
        XK_Caps_Lock => Some(keyboard_scancode_t::KEY_SCANCODE_CAPSLOCK),
        XK_Scroll_Lock => Some(keyboard_scancode_t::KEY_SCANCODE_SCROLLLOCK),
        XK_Shift_L => Some(keyboard_scancode_t::KEY_SCANCODE_LSHIFT),
        XK_Shift_R => Some(keyboard_scancode_t::KEY_SCANCODE_RSHIFT),
        XK_Alt_L => Some(keyboard_scancode_t::KEY_SCANCODE_LALT),
        XK_Alt_R => Some(keyboard_scancode_t::KEY_SCANCODE_RALT),
        XK_Control_L => Some(keyboard_scancode_t::KEY_SCANCODE_LCTRL),
        XK_Control_R => Some(keyboard_scancode_t::KEY_SCANCODE_RCTRL),
        XK_Super_L => Some(keyboard_scancode_t::KEY_SCANCODE_LGUI),
        XK_Super_R => Some(keyboard_scancode_t::KEY_SCANCODE_RGUI),

        XK_KP_0 => Some(keyboard_scancode_t::KEY_SCANCODE_KP_0),
        XK_KP_1 => Some(keyboard_scancode_t::KEY_SCANCODE_KP_1),
        XK_KP_2 => Some(keyboard_scancode_t::KEY_SCANCODE_KP_2),
        XK_KP_3 => Some(keyboard_scancode_t::KEY_SCANCODE_KP_3),
        XK_KP_4 => Some(keyboard_scancode_t::KEY_SCANCODE_KP_4),
        XK_KP_5 => Some(keyboard_scancode_t::KEY_SCANCODE_KP_5),
        XK_KP_6 => Some(keyboard_scancode_t::KEY_SCANCODE_KP_6),
        XK_KP_7 => Some(keyboard_scancode_t::KEY_SCANCODE_KP_7),
        XK_KP_8 => Some(keyboard_scancode_t::KEY_SCANCODE_KP_8),
        XK_KP_9 => Some(keyboard_scancode_t::KEY_SCANCODE_KP_9),
        XK_KP_Decimal => Some(keyboard_scancode_t::KEY_SCANCODE_KP_PERIOD),
        XK_KP_Divide => Some(keyboard_scancode_t::KEY_SCANCODE_KP_DIVIDE),
        XK_KP_Multiply => Some(keyboard_scancode_t::KEY_SCANCODE_KP_MULTIPLY),
        XK_KP_Subtract => Some(keyboard_scancode_t::KEY_SCANCODE_KP_MINUS),
        XK_KP_Add => Some(keyboard_scancode_t::KEY_SCANCODE_KP_PLUS),
        XK_KP_Enter => Some(keyboard_scancode_t::KEY_SCANCODE_KP_ENTER),
        _ => None,
    }
}

fn modifier_for_scancode(scancode: keyboard_scancode_t) -> ModifierState {
    match scancode {
        keyboard_scancode_t::KEY_SCANCODE_LSHIFT => ModifierState::KMOD_LSHIFT,
        keyboard_scancode_t::KEY_SCANCODE_RSHIFT => ModifierState::KMOD_RSHIFT,
        keyboard_scancode_t::KEY_SCANCODE_LCTRL => ModifierState::KMOD_LCTRL,
        keyboard_scancode_t::KEY_SCANCODE_RCTRL => ModifierState::KMOD_RCTRL,
        keyboard_scancode_t::KEY_SCANCODE_LALT => ModifierState::KMOD_LALT,
        keyboard_scancode_t::KEY_SCANCODE_RALT => ModifierState::KMOD_RALT,
        keyboard_scancode_t::KEY_SCANCODE_LGUI => ModifierState::KMOD_LGUI,
        keyboard_scancode_t::KEY_SCANCODE_RGUI => ModifierState::KMOD_RGUI,
        keyboard_scancode_t::KEY_SCANCODE_NUMLOCKCLEAR => ModifierState::KMOD_NUM,
        keyboard_scancode_t::KEY_SCANCODE_CAPSLOCK => ModifierState::KMOD_CAPS,
        keyboard_scancode_t::KEY_SCANCODE_SCROLLLOCK => ModifierState::KMOD_SCROLL,
        _ => ModifierState::KMOD_NONE,
    }
}

pub(crate) fn update_modifier_state(
    modifiers: &mut ModifierState,
    scancode: keyboard_scancode_t,
    is_down: bool,
) {
    let modifier = modifier_for_scancode(scancode);
    if modifier == ModifierState::KMOD_NONE {
        return;
    }

    if is_down {
        *modifiers |= modifier;
    } else {
        *modifiers &= !modifier;
    }
}

pub(crate) fn empty_event() -> event_t {
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
    event_t {
        type_: event_type_t::EVENT_NONE,
        __bindgen_padding_0: [0; 4],
        __bindgen_anon_1: empty_union,
    }
}

pub(crate) fn key_event(
    scancode: keyboard_scancode_t,
    modifiers: ModifierState,
    down: bool,
    timestamp: u64,
) -> event_t {
    let keyboard_event = keyboard_event_t {
        timestamp,
        scancode,
        key: scancode_to_keycode(scancode),
        mod_: modifiers.0,
        text: resolve_typed_character(scancode, modifiers),
        down,
        repeat: false,
        __bindgen_padding_0: [0u8; 3],
    };
    let event_union = event_t__bindgen_ty_1 {
        keyboard: keyboard_event,
    };
    event_t {
        type_: if down {
            event_type_t::EVENT_KEY_DOWN
        } else {
            event_type_t::EVENT_KEY_UP
        },
        __bindgen_padding_0: [0; 4],
        __bindgen_anon_1: event_union,
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct ModifierState(pub u16);

impl ModifierState {
    pub const KMOD_NONE: ModifierState = ModifierState(BADGEVMS_KMOD_NONE as u16);
    pub const KMOD_LSHIFT: ModifierState = ModifierState(BADGEVMS_KMOD_LSHIFT as u16);
    pub const KMOD_RSHIFT: ModifierState = ModifierState(BADGEVMS_KMOD_RSHIFT as u16);
    pub const KMOD_LEVEL5: ModifierState = ModifierState(BADGEVMS_KMOD_LEVEL5 as u16);
    pub const KMOD_LCTRL: ModifierState = ModifierState(BADGEVMS_KMOD_LCTRL as u16);
    pub const KMOD_RCTRL: ModifierState = ModifierState(BADGEVMS_KMOD_RCTRL as u16);
    pub const KMOD_LALT: ModifierState = ModifierState(BADGEVMS_KMOD_LALT as u16);
    pub const KMOD_RALT: ModifierState = ModifierState(BADGEVMS_KMOD_RALT as u16);
    pub const KMOD_LGUI: ModifierState = ModifierState(BADGEVMS_KMOD_LGUI as u16);
    pub const KMOD_RGUI: ModifierState = ModifierState(BADGEVMS_KMOD_RGUI as u16);
    pub const KMOD_NUM: ModifierState = ModifierState(BADGEVMS_KMOD_NUM as u16);
    pub const KMOD_CAPS: ModifierState = ModifierState(BADGEVMS_KMOD_CAPS as u16);
    pub const KMOD_MODE: ModifierState = ModifierState(BADGEVMS_KMOD_MODE as u16);
    pub const KMOD_SCROLL: ModifierState = ModifierState(BADGEVMS_KMOD_SCROLL as u16);
    pub const KMOD_CTRL: ModifierState = ModifierState(BADGEVMS_KMOD_CTRL as u16);
    pub const KMOD_SHIFT: ModifierState = ModifierState(BADGEVMS_KMOD_SHIFT as u16);
    pub const KMOD_ALT: ModifierState = ModifierState(BADGEVMS_KMOD_ALT as u16);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_event_is_none() {
        assert_eq!(empty_event().type_, event_type_t::EVENT_NONE);
    }

    #[test]
    fn shifted_text_uses_upper_variant() {
        assert_eq!(
            resolve_typed_character(
                keyboard_scancode_t::KEY_SCANCODE_A,
                ModifierState::KMOD_SHIFT
            ),
            'A' as ::core::ffi::c_char,
        );
    }

    #[test]
    fn x11_letter_keysym_maps_to_scancode() {
        assert_eq!(
            scancode_from_x11_keysym(XK_a as c_ulong),
            Some(keyboard_scancode_t::KEY_SCANCODE_A),
        );
    }

    #[test]
    fn x11_f13_keysym_maps_to_scancode() {
        assert_eq!(
            scancode_from_x11_keysym(XK_F13 as c_ulong),
            Some(keyboard_scancode_t::KEY_SCANCODE_F13),
        );
    }

    #[test]
    fn key_release_clears_modifier_bit() {
        let mut modifiers = ModifierState::KMOD_LSHIFT;
        update_modifier_state(
            &mut modifiers,
            keyboard_scancode_t::KEY_SCANCODE_LSHIFT,
            false,
        );
        assert_eq!(modifiers, ModifierState::KMOD_NONE);
    }

    #[test]
    fn key_press_sets_modifier_bit() {
        let mut modifiers = ModifierState::KMOD_NONE;
        update_modifier_state(
            &mut modifiers,
            keyboard_scancode_t::KEY_SCANCODE_LSHIFT,
            true,
        );
        assert_eq!(modifiers, ModifierState::KMOD_LSHIFT);
    }

    #[test]
    fn key_event_marks_key_down() {
        let event = key_event(
            keyboard_scancode_t::KEY_SCANCODE_A,
            ModifierState::KMOD_SHIFT,
            true,
            123,
        );
        assert_eq!(event.type_, event_type_t::EVENT_KEY_DOWN);
        unsafe {
            assert_eq!(
                event.__bindgen_anon_1.keyboard.text,
                'A' as ::core::ffi::c_char
            );
        }
    }
}
