use crate::types::*;

#[unsafe(no_mangle)]
pub extern "C" fn process_create(
    path: *const ::core::ffi::c_char,
    stack_size: size_t,
    argc: ::core::ffi::c_int,
    argv: *mut *mut ::core::ffi::c_char,
) -> pid_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn thread_create(
    thread_entry: ::core::option::Option<unsafe extern "C" fn(user_data: *mut ::core::ffi::c_void)>,
    user_data: *mut ::core::ffi::c_void,
    stack_size: u16,
) -> pid_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn die(reason: *const ::core::ffi::c_char) {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn task_priority_lower() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn task_priority_restore() {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn get_num_tasks() -> u32 {
    unimplemented!("Implement this yourself if you need it");
}

#[unsafe(no_mangle)]
pub extern "C" fn device_get(name: *const ::core::ffi::c_char) -> *mut device_t {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn vaddr_to_paddr(vaddr: u32) -> u32 {
    unimplemented!("Implement this yourself if you need it");
}
