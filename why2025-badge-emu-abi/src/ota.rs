use crate::types::*;

mod runtime;

pub(crate) use runtime::abort_task_owned_ota_session;
use runtime::{
    ota_get_invalid_version_inner, ota_get_running_version_inner, ota_session_abort_inner,
    ota_session_commit_inner, ota_session_open_inner, ota_write_inner,
};

#[unsafe(no_mangle)]
extern "C" fn ota_session_open() -> ota_handle_t {
    ota_session_open_inner()
}

#[unsafe(no_mangle)]
extern "C" fn ota_write(
    session: ota_handle_t,
    buffer: *mut ::core::ffi::c_void,
    block_size: ::core::ffi::c_int,
) -> bool {
    ota_write_inner(session, buffer, block_size)
}

#[unsafe(no_mangle)]
extern "C" fn ota_session_commit(session: ota_handle_t) -> bool {
    ota_session_commit_inner(session)
}

#[unsafe(no_mangle)]
extern "C" fn ota_session_abort(session: ota_handle_t) -> bool {
    ota_session_abort_inner(session)
}

#[unsafe(no_mangle)]
extern "C" fn ota_get_running_version(version: *mut *mut ::core::ffi::c_char) -> bool {
    ota_get_running_version_inner(version)
}

#[unsafe(no_mangle)]
extern "C" fn ota_get_invalid_version(version: *mut *mut ::core::ffi::c_char) -> bool {
    ota_get_invalid_version_inner(version)
}