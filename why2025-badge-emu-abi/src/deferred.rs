use crate::runtime;
use crate::types::*;
use core::ffi::{c_char, c_long};

macro_rules! aborting_export {
    ($family:literal, fn $name:ident($($arg:ident : $arg_ty:ty),* $(,)?) -> $ret:ty) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name($($arg : $arg_ty),*) -> $ret {
            let _ = ($($arg),*);
            runtime::abort_unimplemented_symbol(stringify!($name), $family)
        }
    };
    ($family:literal, fn $name:ident($($arg:ident : $arg_ty:ty),* $(,)?)) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name($($arg : $arg_ty),*) {
            let _ = ($($arg),*);
            runtime::abort_unimplemented_symbol(stringify!($name), $family)
        }
    };
}

aborting_export!("networking", fn curl_easy_init() -> *mut CURL);
aborting_export!("networking", fn curl_easy_perform(curl: *mut CURL) -> CURLcode);
aborting_export!("networking", fn curl_easy_cleanup(curl: *mut CURL));
aborting_export!("networking", fn curl_easy_strerror(error: CURLcode) -> *const c_char);
aborting_export!("networking", fn curl_slist_append(list: *mut curl_slist, string: *const c_char) -> *mut curl_slist);
aborting_export!("networking", fn curl_slist_free_all(list: *mut curl_slist));
aborting_export!("networking", fn curl_global_init(flags: c_long) -> CURLcode);
aborting_export!("networking", fn curl_global_cleanup());

#[unsafe(no_mangle)]
pub unsafe extern "C" fn curl_easy_setopt(
    curl: *mut CURL,
    option: CURLoption,
    mut _args: ...
) -> CURLcode {
    let _ = (curl, option);
    runtime::abort_unimplemented_symbol("curl_easy_setopt", "networking")
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn curl_easy_getinfo(
    curl: *mut CURL,
    info: curl_easy_info_t,
    mut _args: ...
) -> CURLcode {
    let _ = (curl, info);
    runtime::abort_unimplemented_symbol("curl_easy_getinfo", "networking")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host_forward::socket;

    #[test]
    fn deferred_symbols_have_addresses() {
        assert_ne!(crate::wifi::wifi_get_status as *const (), core::ptr::null());
        assert_ne!(curl_easy_init as *const (), core::ptr::null());
        assert_ne!(socket as *const (), core::ptr::null());
    }
}
