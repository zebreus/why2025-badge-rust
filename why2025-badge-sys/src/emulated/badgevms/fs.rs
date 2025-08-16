use crate::types::*;

#[unsafe(no_mangle)]
pub extern "C" fn parse_path(
    path: *const ::core::ffi::c_char,
    result: *mut path_t,
) -> path_parse_result_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn path_free(path: *mut path_t) {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn mkdir_p(path: *const ::core::ffi::c_char) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn rm_rf(path: *const ::core::ffi::c_char) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn path_dirname(path: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn path_basename(path: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn path_devname(path: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn path_dirconcat(
    path: *const ::core::ffi::c_char,
    subdir: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn path_fileconcat(
    path: *const ::core::ffi::c_char,
    filename: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn path_concat(
    base_path: *const ::core::ffi::c_char,
    append_path: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}
