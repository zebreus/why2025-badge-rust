use super::input::{ModifierState, key_event, scancode_from_x11_keysym, update_modifier_state};

use crate::types::event_t;

use alloc::vec::Vec;
use core::convert::TryFrom;
use core::ffi::{CStr, c_char, c_int, c_uchar, c_ulong, c_void};
use core::mem;
use core::ptr::{self, NonNull};

use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WindowHandle, XlibDisplayHandle, XlibWindowHandle,
};
use x11_dl::xlib::{self, XEvent};

#[derive(Default)]
pub(crate) struct PumpResult {
    pub(crate) position: Option<(usize, usize)>,
    pub(crate) size: Option<(usize, usize)>,
    pub(crate) events: Vec<event_t>,
}

#[repr(C)]
struct MwmHints {
    flags: c_ulong,
    functions: c_ulong,
    decorations: c_ulong,
    input_mode: libc::c_long,
    status: c_ulong,
}

pub(crate) struct X11Window {
    lib: x11_dl::xlib::Xlib,
    display: *mut xlib::Display,
    screen: c_int,
    visual: *mut xlib::Visual,
    depth: c_int,
    gc: xlib::GC,
    handle: xlib::Window,
    wm_delete_window: xlib::Atom,
    ximage: *mut xlib::XImage,
    image_size: (usize, usize),
    draw_buffer: Vec<u32>,
    modifiers: ModifierState,
}

impl HasWindowHandle for X11Window {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let raw_handle = RawWindowHandle::Xlib(XlibWindowHandle::new(self.handle));
        unsafe { Ok(WindowHandle::borrow_raw(raw_handle)) }
    }
}

impl HasDisplayHandle for X11Window {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let display = NonNull::new(self.display.cast::<c_void>());
        let raw_handle = RawDisplayHandle::Xlib(XlibDisplayHandle::new(display, self.screen));
        unsafe { Ok(DisplayHandle::borrow_raw(raw_handle)) }
    }
}

impl X11Window {
    pub(crate) fn try_open(
        title: &[u8; 128],
        size: (usize, usize),
        undecorated: bool,
    ) -> Option<Self> {
        if cfg!(test) {
            return None;
        }

        unsafe {
            let lib = x11_dl::xlib::Xlib::open().ok()?;
            let _ = (lib.XInitThreads)();

            let display = (lib.XOpenDisplay)(ptr::null());
            if display.is_null() {
                return None;
            }

            let screen = (lib.XDefaultScreen)(display);
            let visual = (lib.XDefaultVisual)(display, screen);
            let depth = (lib.XDefaultDepth)(display, screen);

            let root = (lib.XDefaultRootWindow)(display);

            let mut attributes: xlib::XSetWindowAttributes = mem::zeroed();
            attributes.border_pixel = (lib.XBlackPixel)(display, screen);
            attributes.background_pixel = attributes.border_pixel;
            attributes.backing_store = xlib::NotUseful;

            let handle = (lib.XCreateWindow)(
                display,
                root,
                0,
                0,
                size.0 as u32,
                size.1 as u32,
                0,
                depth,
                xlib::InputOutput as u32,
                visual,
                xlib::CWBackingStore | xlib::CWBackPixel | xlib::CWBorderPixel,
                &mut attributes,
            );

            if handle == 0 {
                (lib.XCloseDisplay)(display);
                return None;
            }

            let gc = (lib.XCreateGC)(display, handle, 0, ptr::null_mut());
            if gc.is_null() {
                (lib.XDestroyWindow)(display, handle);
                (lib.XCloseDisplay)(display);
                return None;
            }

            let wm_delete_window = (lib.XInternAtom)(
                display,
                b"WM_DELETE_WINDOW\0".as_ptr() as *const c_char,
                xlib::False,
            );

            (lib.XSelectInput)(
                display,
                handle,
                xlib::StructureNotifyMask
                    | xlib::ExposureMask
                    | xlib::KeyPressMask
                    | xlib::KeyReleaseMask,
            );

            if undecorated {
                let hints_property = (lib.XInternAtom)(
                    display,
                    b"_MOTIF_WM_HINTS\0".as_ptr() as *const c_char,
                    xlib::False,
                );
                if hints_property != 0 {
                    let hints = MwmHints {
                        flags: 2,
                        functions: 0,
                        decorations: 0,
                        input_mode: 0,
                        status: 0,
                    };
                    (lib.XChangeProperty)(
                        display,
                        handle,
                        hints_property,
                        hints_property,
                        32,
                        xlib::PropModeReplace,
                        &hints as *const _ as *const c_uchar,
                        5,
                    );
                }
            }

            let mut wm_delete_window_protocol = wm_delete_window;
            (lib.XSetWMProtocols)(display, handle, &mut wm_delete_window_protocol, 1);
            (lib.XClearWindow)(display, handle);
            (lib.XMapRaised)(display, handle);

            let mut window = Self {
                lib,
                display,
                screen,
                visual,
                depth,
                gc,
                handle,
                wm_delete_window,
                ximage: ptr::null_mut(),
                image_size: (0, 0),
                draw_buffer: Vec::new(),
                modifiers: ModifierState::KMOD_NONE,
            };

            window.set_title(title);
            if !window.recreate_image(size) {
                return None;
            }

            (window.lib.XFlush)(window.display);
            Some(window)
        }
    }

    pub(crate) fn set_title(&mut self, title: &[u8; 128]) {
        let Ok(title) = CStr::from_bytes_until_nul(title) else {
            return;
        };

        unsafe {
            (self.lib.XStoreName)(self.display, self.handle, title.as_ptr());

            if let Ok(title_len) = c_int::try_from(title.to_bytes().len()) {
                let net_wm_name = (self.lib.XInternAtom)(
                    self.display,
                    b"_NET_WM_NAME\0".as_ptr() as *const c_char,
                    xlib::False,
                );
                let utf8_string = (self.lib.XInternAtom)(
                    self.display,
                    b"UTF8_STRING\0".as_ptr() as *const c_char,
                    xlib::False,
                );

                if net_wm_name != 0 && utf8_string != 0 {
                    (self.lib.XChangeProperty)(
                        self.display,
                        self.handle,
                        net_wm_name,
                        utf8_string,
                        8,
                        xlib::PropModeReplace,
                        title.as_ptr() as *const c_uchar,
                        title_len,
                    );
                }
            }

            (self.lib.XFlush)(self.display);
        }
    }

    pub(crate) fn set_position(&mut self, position: (usize, usize)) {
        unsafe {
            (self.lib.XMoveWindow)(
                self.display,
                self.handle,
                position.0 as c_int,
                position.1 as c_int,
            );
            (self.lib.XFlush)(self.display);
        }
    }

    pub(crate) fn set_size(&mut self, size: (usize, usize)) {
        unsafe {
            (self.lib.XResizeWindow)(self.display, self.handle, size.0 as u32, size.1 as u32);
            let _ = self.recreate_image(size);
            (self.lib.XFlush)(self.display);
        }
    }

    pub(crate) fn present(&mut self, pixels: &[u32], size: (usize, usize)) {
        if self.image_size != size && !self.recreate_image(size) {
            return;
        }

        if self.draw_buffer.len() != pixels.len() {
            return;
        }

        self.draw_buffer.copy_from_slice(pixels);

        unsafe {
            (self.lib.XPutImage)(
                self.display,
                self.handle,
                self.gc,
                self.ximage,
                0,
                0,
                0,
                0,
                size.0 as u32,
                size.1 as u32,
            );
            (self.lib.XFlush)(self.display);
        }
    }

    pub(crate) fn pump_events(&mut self) -> PumpResult {
        let mut result = PumpResult::default();

        unsafe {
            let count = (self.lib.XPending)(self.display);
            for _ in 0..count {
                let mut event: XEvent = mem::zeroed();
                (self.lib.XNextEvent)(self.display, &mut event);

                if event.any.window != self.handle {
                    continue;
                }

                match event.type_ {
                    xlib::ClientMessage => {
                        let _ = self.wm_delete_window;
                    }
                    xlib::ConfigureNotify => {
                        let width = event.configure.width.max(0) as usize;
                        let height = event.configure.height.max(0) as usize;
                        let size = (width, height);
                        result.position = Some((
                            event.configure.x.max(0) as usize,
                            event.configure.y.max(0) as usize,
                        ));
                        result.size = Some(size);
                        if self.image_size != size {
                            let _ = self.recreate_image(size);
                        }
                    }
                    xlib::KeyPress | xlib::KeyRelease => {
                        let keysym = (self.lib.XLookupKeysym)(&mut event.key, 0);
                        if let Some(scancode) = scancode_from_x11_keysym(keysym) {
                            let is_down = event.type_ == xlib::KeyPress;
                            update_modifier_state(&mut self.modifiers, scancode, is_down);
                            result.events.push(key_event(
                                scancode,
                                self.modifiers,
                                is_down,
                                current_time_micros(),
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }

        result
    }

    pub(crate) fn wait_for_event(&self, timeout_msec: Option<u32>) -> bool {
        let fd = unsafe { (self.lib.XConnectionNumber)(self.display) };
        if fd < 0 {
            return false;
        }

        let timeout = timeout_msec
            .map(|timeout_msec| timeout_msec.min(i32::MAX as u32) as c_int)
            .unwrap_or(-1);
        let mut poll_fd = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };

        unsafe { libc::poll(&mut poll_fd, 1, timeout) > 0 }
    }

    fn recreate_image(&mut self, size: (usize, usize)) -> bool {
        unsafe {
            self.destroy_image();

            let bytes_per_line = size.0 as c_int * 4;
            self.draw_buffer.resize(size.0 * size.1, 0);
            let image = (self.lib.XCreateImage)(
                self.display,
                self.visual,
                self.depth as u32,
                xlib::ZPixmap,
                0,
                self.draw_buffer.as_mut_ptr() as *mut c_char,
                size.0 as u32,
                size.1 as u32,
                32,
                bytes_per_line,
            );

            if image.is_null() {
                self.image_size = (0, 0);
                false
            } else {
                self.ximage = image;
                self.image_size = size;
                true
            }
        }
    }

    unsafe fn destroy_image(&mut self) {
        if self.ximage.is_null() {
            return;
        }

        unsafe {
            (*self.ximage).data = ptr::null_mut();
            (self.lib.XDestroyImage)(self.ximage);
        }
        self.ximage = ptr::null_mut();
    }
}

fn current_time_micros() -> u64 {
    unsafe {
        let mut time: libc::timeval = mem::zeroed();
        if libc::gettimeofday(&mut time, ptr::null_mut()) != 0 {
            return 0;
        }

        (time.tv_sec as u64)
            .saturating_mul(1_000_000)
            .saturating_add(time.tv_usec as u64)
    }
}

impl Drop for X11Window {
    fn drop(&mut self) {
        unsafe {
            self.destroy_image();
            if !self.gc.is_null() {
                (self.lib.XFreeGC)(self.display, self.gc);
            }
            if self.handle != 0 {
                (self.lib.XDestroyWindow)(self.display, self.handle);
            }
            if !self.display.is_null() {
                (self.lib.XCloseDisplay)(self.display);
            }
        }
    }
}
