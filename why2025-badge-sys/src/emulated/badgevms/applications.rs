use crate::types::*;

#[unsafe(no_mangle)]
pub extern "C" fn application_launch(unique_identifier: *const ::core::ffi::c_char) -> pid_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_create(
    unique_identifier: *const ::core::ffi::c_char,
    name: *const ::core::ffi::c_char,
    author: *const ::core::ffi::c_char,
    version: *const ::core::ffi::c_char,
    interpreter: *const ::core::ffi::c_char,
    source: application_source_t,
) -> *mut application_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_set_metadata(
    application: *mut application_t,
    metadata_file: *const ::core::ffi::c_char,
) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_set_binary_path(
    application: *mut application_t,
    binary_path: *const ::core::ffi::c_char,
) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_set_version(
    application: *mut application_t,
    version: *const ::core::ffi::c_char,
) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_set_author(
    application: *mut application_t,
    author: *const ::core::ffi::c_char,
) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_set_name(
    application: *mut application_t,
    name: *const ::core::ffi::c_char,
) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_set_interpreter(
    application: *mut application_t,
    interpreter: *const ::core::ffi::c_char,
) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_destroy(application: *mut application_t) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_create_file(
    application: *mut application_t,
    file_path: *const ::core::ffi::c_char,
) -> *mut FILE {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_create_file_string(
    application: *mut application_t,
    file_path: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_list(out: *mut *mut application_t) -> application_list_handle {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_list_get_next(list: application_list_handle) -> *mut application_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_list_close(list: application_list_handle) {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_get(
    unique_identifier: *const ::core::ffi::c_char,
) -> *mut application_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn application_free(application: *mut application_t) {
    unimplemented!("Implement this yourself if you need it");
}
