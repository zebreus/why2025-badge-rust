#![allow(dead_code)]

use core::char;
use core::pin::Pin;
use core::task::{Context, Poll};
use std::thread;
use std::time::Duration;

use futures::Stream;
use ocpncord_tui::{Event, KeyEvent, Modifiers, Scancode};
use why2025_badge_sys_bindings::{
    BADGEVMS_KMOD_ALT, BADGEVMS_KMOD_CTRL, BADGEVMS_KMOD_GUI, BADGEVMS_KMOD_SHIFT, event_t,
    event_type_t, key_mod_t, keyboard_event_t, keyboard_scancode_t, window_event_poll,
    window_handle_t,
};

pub struct BadgeEventStream {
    window: window_handle_t,
    timeout_msec: u32,
}

impl BadgeEventStream {
    pub fn new(window: window_handle_t, timeout_msec: u32) -> Self {
        Self {
            window,
            timeout_msec,
        }
    }
}

impl Stream for BadgeEventStream {
    type Item = Event;

    fn poll_next(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let next_event = poll(self.window).unwrap_or_else(|| {
            thread::sleep(Duration::from_millis(self.timeout_msec as u64));
            Event::Tick
        });
        Poll::Ready(Some(next_event))
    }
}

pub fn poll(window: window_handle_t) -> Option<Event> {
    let event = unsafe { window_event_poll(window, false, 0) };
    translate(event)
}

pub fn translate(event: event_t) -> Option<Event> {
    match event.type_ {
        event_type_t::EVENT_NONE => None,
        event_type_t::EVENT_QUIT => Some(Event::Quit),
        event_type_t::EVENT_WINDOW_RESIZE => Some(Event::Tick),
        event_type_t::EVENT_KEY_DOWN => {
            let keyboard = unsafe { event.__bindgen_anon_1.keyboard };
            translate_key_down(keyboard).map(Event::Key)
        }
        event_type_t::EVENT_KEY_UP => None,
    }
}

fn translate_key_down(event: keyboard_event_t) -> Option<KeyEvent> {
    Some(KeyEvent {
        scancode: translate_scancode(event)?,
        modifiers: translate_modifiers(event.mod_),
    })
}

fn translate_modifiers(modifiers: key_mod_t) -> Modifiers {
    let modifiers = modifiers as u32;
    Modifiers {
        shift: (modifiers & BADGEVMS_KMOD_SHIFT) != 0,
        ctrl: (modifiers & BADGEVMS_KMOD_CTRL) != 0,
        alt: (modifiers & BADGEVMS_KMOD_ALT) != 0,
        meta: (modifiers & BADGEVMS_KMOD_GUI) != 0,
    }
}

fn translate_scancode(event: keyboard_event_t) -> Option<Scancode> {
    match event.scancode {
        keyboard_scancode_t::KEY_SCANCODE_RETURN
        | keyboard_scancode_t::KEY_SCANCODE_RETURN2 => Some(Scancode::Enter),
        keyboard_scancode_t::KEY_SCANCODE_ESCAPE => Some(Scancode::Escape),
        keyboard_scancode_t::KEY_SCANCODE_BACKSPACE => Some(Scancode::Backspace),
        keyboard_scancode_t::KEY_SCANCODE_TAB => Some(Scancode::Tab),
        keyboard_scancode_t::KEY_SCANCODE_UP => Some(Scancode::Up),
        keyboard_scancode_t::KEY_SCANCODE_DOWN => Some(Scancode::Down),
        keyboard_scancode_t::KEY_SCANCODE_LEFT => Some(Scancode::Left),
        keyboard_scancode_t::KEY_SCANCODE_RIGHT => Some(Scancode::Right),
        keyboard_scancode_t::KEY_SCANCODE_HOME => Some(Scancode::Home),
        keyboard_scancode_t::KEY_SCANCODE_END => Some(Scancode::End),
        keyboard_scancode_t::KEY_SCANCODE_PAGEUP => Some(Scancode::PageUp),
        keyboard_scancode_t::KEY_SCANCODE_PAGEDOWN => Some(Scancode::PageDown),
        keyboard_scancode_t::KEY_SCANCODE_DELETE => Some(Scancode::Delete),
        keyboard_scancode_t::KEY_SCANCODE_F1
        | keyboard_scancode_t::KEY_SCANCODE_F2
        | keyboard_scancode_t::KEY_SCANCODE_F3
        | keyboard_scancode_t::KEY_SCANCODE_F4
        | keyboard_scancode_t::KEY_SCANCODE_F5
        | keyboard_scancode_t::KEY_SCANCODE_F6
        | keyboard_scancode_t::KEY_SCANCODE_F7
        | keyboard_scancode_t::KEY_SCANCODE_F8
        | keyboard_scancode_t::KEY_SCANCODE_F9
        | keyboard_scancode_t::KEY_SCANCODE_F10
        | keyboard_scancode_t::KEY_SCANCODE_F11
        | keyboard_scancode_t::KEY_SCANCODE_F12
        | keyboard_scancode_t::KEY_SCANCODE_F13
        | keyboard_scancode_t::KEY_SCANCODE_F14
        | keyboard_scancode_t::KEY_SCANCODE_F15 => Some(Scancode::F(
            (event.scancode as u32 - keyboard_scancode_t::KEY_SCANCODE_F1 as u32 + 1) as u8,
        )),
        _ => char::from_u32(event.text as u8 as u32).map(Scancode::Char),
    }
}
