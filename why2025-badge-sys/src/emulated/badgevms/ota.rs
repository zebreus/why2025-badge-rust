use crate::types::*;

#[unsafe(no_mangle)]
pub extern "C" fn ota_session_open() -> ota_handle_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn ota_write(
    session: ota_handle_t,
    buffer: *mut ::core::ffi::c_void,
    block_size: ::core::ffi::c_int,
) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn ota_session_commit(session: ota_handle_t) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn ota_session_abort(session: ota_handle_t) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn ota_get_running_version(version: *mut ::core::ffi::c_char) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn ota_get_invalid_version(version: *mut ::core::ffi::c_char) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
