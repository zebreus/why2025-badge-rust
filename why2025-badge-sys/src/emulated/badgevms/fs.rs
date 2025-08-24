use crate::{malloc, types::*};
use std::ffi::CStr;

#[derive(Debug)]
struct ParseResult {
    device: *const u8,
    filename: *const u8,
    directory: *const u8,
    /// This points to the byte after the device name.
    /// Needs to be set to 0 to make device a valid C string.
    device_end: *const u8,
    /// If not null, this points to the byte after the directory name.
    /// Needs to be set to 0 to make directory a valid C string.
    directory_end: *const u8,
}

impl ParseResult {
    /// Set the null terminators in the original string to sure that `device`, `directory` and `filename` are valid C strings.
    ///
    /// This is unsafe because it modifies the original string even if it is not mutable.
    unsafe fn finalize(&self) {
        unsafe {
            if !self.device_end.is_null() {
                *(self.device_end as *mut u8) = 0;
            }
            if !self.directory_end.is_null() {
                *(self.directory_end as *mut u8) = 0;
            }
        }
    }
}

fn parse_path_internal(input: &core::ffi::CStr) -> Result<ParseResult, path_parse_result_t> {
    enum ParseState {
        Device(ParseResult),
        Filename(ParseResult),
        MaybeDirectory(ParseResult),
        Directory(ParseResult),
    }
    let mut state = ParseState::Device(ParseResult {
        device: std::ptr::null_mut(),
        filename: std::ptr::null_mut(),
        directory: std::ptr::null_mut(),
        device_end: std::ptr::null_mut(),
        directory_end: std::ptr::null_mut(),
    });
    for c in input.to_bytes().iter() {
        state = match (state, *c) {
            (_, 0) => {
                unreachable!(
                    "Encountered null terminator in string. This should not be possible here."
                );
            }
            (ParseState::Device(r), b':') => {
                if r.device.is_null() {
                    return Err(path_parse_result_t::PATH_PARSE_EMPTY_DEVICE);
                }
                ParseState::MaybeDirectory(ParseResult { device_end: c, ..r })
            }
            (
                ParseState::Device(r),
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'$',
            ) => {
                if r.device.is_null() {
                    ParseState::Device(ParseResult { device: c, ..r })
                } else {
                    ParseState::Device(r)
                }
            }
            (ParseState::Device(_), _) => {
                return Err(path_parse_result_t::PATH_PARSE_INVALID_DEVICE_CHAR);
            }
            (ParseState::MaybeDirectory(r), b'[') => ParseState::Directory(r),
            (ParseState::Directory(r), b']') => {
                if r.directory.is_null() {
                    ParseState::Filename(r)
                } else {
                    ParseState::Filename(ParseResult {
                        directory_end: c,
                        ..r
                    })
                }
            }
            (
                ParseState::Directory(r),
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'$' | b'.',
            ) => {
                if r.directory.is_null() {
                    ParseState::Directory(ParseResult { directory: c, ..r })
                } else {
                    ParseState::Directory(r)
                }
            }
            (ParseState::Directory(_), _) => {
                return Err(path_parse_result_t::PATH_PARSE_INVALID_DIR_CHAR);
            }
            (
                ParseState::Filename(r) | ParseState::MaybeDirectory(r),
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'$' | b'.',
            ) => {
                if r.filename.is_null() {
                    ParseState::Filename(ParseResult { filename: c, ..r })
                } else {
                    ParseState::Filename(r)
                }
            }
            (ParseState::Filename(_) | ParseState::MaybeDirectory(_), _) => {
                return Err(path_parse_result_t::PATH_PARSE_INVALID_FILE_CHAR);
            }
        }
    }

    let result = match state {
        ParseState::Filename(r) | ParseState::MaybeDirectory(r) => r,
        ParseState::Directory(_) => {
            return Err(path_parse_result_t::PATH_PARSE_UNCLOSED_DIRECTORY);
        }
        ParseState::Device(_) => {
            return Err(path_parse_result_t::PATH_PARSE_NO_DEVICE);
        }
    };
    Ok(result)
}

/// Parse a path into a path structure
///
/// UNIX paths do not work! Paths are in the form of `DEVICE:[directory.subdirectory]filename.ext`
///
/// The structure will hold allocated memory for the path, which must be freed with `path_free`
/// after use.
///
/// Returns a `path_parse_result_t` indicating success or failure reason.
///
/// You must free the path with `path_free` even if this function indicates an error, as it will still have allocated memory for the path buffer.
#[unsafe(no_mangle)]
pub extern "C" fn parse_path(
    path: *const ::core::ffi::c_char,
    result: *mut path_t,
) -> path_parse_result_t {
    if result.is_null() {
        panic!("Result pointer cannot be a nullptr");
    }
    let result = unsafe { &mut *result };

    if path.is_null() {
        return path_parse_result_t::PATH_PARSE_EMPTY_PATH;
    }
    let c_str = unsafe { ::core::ffi::CStr::from_ptr(path) };
    if c_str.to_bytes().len() == 0 {
        return path_parse_result_t::PATH_PARSE_EMPTY_PATH;
    }

    let copied_buffer: Box<CStr> = c_str.into();
    let copied_buffer = Box::leak(copied_buffer);

    *result = path_t {
        device: std::ptr::null_mut(),
        directory: std::ptr::null_mut(),
        filename: std::ptr::null_mut(),
        buffer: copied_buffer.as_ptr() as *mut i8,
        unixpath: std::ptr::null_mut(),
        len: c_str.to_bytes().len(),
    };

    let parse_result = match parse_path_internal(&copied_buffer) {
        Ok(r) => r,
        Err(e) => return e,
    };
    unsafe { parse_result.finalize() };

    *result = path_t {
        device: parse_result.device as *mut i8,
        directory: parse_result.directory as *mut i8,
        filename: parse_result.filename as *mut i8,
        buffer: copied_buffer.as_ptr() as *mut i8,
        unixpath: std::ptr::null_mut(),
        len: c_str.to_bytes().len(),
    };
    return path_parse_result_t::PATH_PARSE_OK;
}

/// Free a path structure previously allocated by `parse_path`
#[unsafe(no_mangle)]
pub extern "C" fn path_free(path: *mut path_t) {
    if path.is_null() {
        panic!("Result pointer cannot be a nullptr");
    }
    let path = unsafe { &mut *path };
    unsafe {
        if !path.buffer.is_null() {
            drop(Box::from_raw(path.buffer));
        }
        if !path.unixpath.is_null() {
            drop(Box::from_raw(path.unixpath));
        }
    };
}
#[unsafe(no_mangle)]
pub extern "C" fn mkdir_p(_path: *const ::core::ffi::c_char) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn rm_rf(_path: *const ::core::ffi::c_char) -> bool {
    unimplemented!("Implement this yourself if you need it");
}

/// Get the device name from a path
///
/// Keep in mind that you must free the returned string with `free` when you are done with it.
#[unsafe(no_mangle)]
pub extern "C" fn path_dirname(path: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    if path.is_null() {
        return core::ptr::null_mut();
    }
    let c_str = unsafe { ::core::ffi::CStr::from_ptr(path) };
    if c_str.to_bytes().len() == 0 {
        return core::ptr::null_mut();
    }
    let parse_result = match parse_path_internal(c_str) {
        Ok(r) => r,
        Err(_) => return core::ptr::null_mut(),
    };
    let path_start = parse_result.device;
    let path_end = if !parse_result.directory_end.is_null() {
        parse_result.directory_end
    } else {
        parse_result.device_end
    };

    let path_len = (path_end as u32 - path_start as u32) + 1;
    let result_buffer = unsafe { malloc(path_len + 1) as *mut u8 };
    unsafe { core::ptr::copy_nonoverlapping(path_start, result_buffer, path_len as usize) };
    unsafe { *result_buffer.byte_add(path_len as usize) = 0 };
    result_buffer as *mut i8
}

/// Get the device name from a path
///
/// Keep in mind that you must free the returned string with `free` when you are done with it.
#[unsafe(no_mangle)]
pub extern "C" fn path_basename(path: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    if path.is_null() {
        return core::ptr::null_mut();
    }
    let c_str = unsafe { ::core::ffi::CStr::from_ptr(path) };
    if c_str.to_bytes().len() == 0 {
        return core::ptr::null_mut();
    }
    let parse_result = match parse_path_internal(c_str) {
        Ok(r) => r,
        Err(_) => return core::ptr::null_mut(),
    };
    let path_end = c_str.to_bytes_with_nul().last().unwrap() as *const u8;
    let path_start = if !parse_result.directory_end.is_null() {
        unsafe { parse_result.directory_end.byte_add(1) }
    } else {
        unsafe { parse_result.device_end.byte_add(1) }
    };

    let path_len = (path_end as u32 - path_start as u32) + 1;
    let result_buffer = unsafe { malloc(path_len + 1) as *mut u8 };
    unsafe { core::ptr::copy_nonoverlapping(path_start, result_buffer, path_len as usize) };
    unsafe { *result_buffer.byte_add(path_len as usize) = 0 };
    result_buffer as *mut i8
}

/// Get the device name from a path
///
/// Keep in mind that you must free the returned string with `free` when you are done with it.
#[unsafe(no_mangle)]
pub extern "C" fn path_devname(path: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    if path.is_null() {
        return core::ptr::null_mut();
    }
    let c_str = unsafe { ::core::ffi::CStr::from_ptr(path) };
    if c_str.to_bytes().len() == 0 {
        return core::ptr::null_mut();
    }
    let parse_result = match parse_path_internal(c_str) {
        Ok(r) => r,
        Err(_) => return core::ptr::null_mut(),
    };
    let path_start = parse_result.device;
    let path_end = parse_result.device_end;

    let path_len = path_end as u32 - path_start as u32;
    let result_buffer = unsafe { malloc(path_len + 1) as *mut u8 };
    unsafe { core::ptr::copy_nonoverlapping(path_start, result_buffer, path_len as usize) };
    unsafe { *result_buffer.byte_add(path_len as usize) = 0 };
    result_buffer as *mut i8
}

#[unsafe(no_mangle)]
pub extern "C" fn path_dirconcat(
    _path: *const ::core::ffi::c_char,
    _subdir: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn path_fileconcat(
    _path: *const ::core::ffi::c_char,
    _filename: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn path_concat(
    _base_path: *const ::core::ffi::c_char,
    _append_path: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unimplemented!("Implement this yourself if you need it");
}

#[cfg(test)]
mod tests {
    use std::ffi::CStr;

    use super::*;

    #[test]
    fn test_parse_path_full() {
        let mut result = path_t {
            device: std::ptr::null_mut(),
            directory: std::ptr::null_mut(),
            filename: std::ptr::null_mut(),
            buffer: std::ptr::null_mut(),
            unixpath: std::ptr::null_mut(),
            len: 0,
        };
        let res = parse_path(
            b"DEV:[dir.subdir]file.ext\0".as_ptr() as *const i8,
            &mut result,
        );
        assert_eq!(res, path_parse_result_t::PATH_PARSE_OK);
        assert_eq!(
            unsafe { CStr::from_ptr(result.device).to_str().unwrap() },
            "DEV"
        );
        assert_eq!(
            unsafe { CStr::from_ptr(result.directory).to_str().unwrap() },
            "dir.subdir"
        );
        assert_eq!(
            unsafe { CStr::from_ptr(result.filename).to_str().unwrap() },
            "file.ext"
        );
        path_free(&mut result);
    }

    #[test]
    fn test_parse_path_device_and_file() {
        let mut result = path_t {
            device: std::ptr::null_mut(),
            directory: std::ptr::null_mut(),
            filename: std::ptr::null_mut(),
            buffer: std::ptr::null_mut(),
            unixpath: std::ptr::null_mut(),
            len: 0,
        };
        let res = parse_path(b"DEV:file.ext\0".as_ptr() as *const i8, &mut result);
        assert_eq!(res, path_parse_result_t::PATH_PARSE_OK);
        assert_eq!(
            unsafe { CStr::from_ptr(result.device).to_str().unwrap() },
            "DEV"
        );
        assert!(result.directory.is_null());
        assert_eq!(
            unsafe { CStr::from_ptr(result.filename).to_str().unwrap() },
            "file.ext"
        );
        path_free(&mut result);
    }

    #[test]
    fn test_parse_path_device_directory_file() {
        let mut result = path_t {
            device: std::ptr::null_mut(),
            directory: std::ptr::null_mut(),
            filename: std::ptr::null_mut(),
            buffer: std::ptr::null_mut(),
            unixpath: std::ptr::null_mut(),
            len: 0,
        };
        let res = parse_path(b"DEV:[dir]file.ext\0".as_ptr() as *const i8, &mut result);
        assert_eq!(res, path_parse_result_t::PATH_PARSE_OK);
        assert_eq!(
            unsafe { CStr::from_ptr(result.device).to_str().unwrap() },
            "DEV"
        );
        assert_eq!(
            unsafe { CStr::from_ptr(result.directory).to_str().unwrap() },
            "dir"
        );
        assert_eq!(
            unsafe { CStr::from_ptr(result.filename).to_str().unwrap() },
            "file.ext"
        );
        path_free(&mut result);
    }

    #[test]
    fn test_parse_path_device_directory_only() {
        let mut result = path_t {
            device: std::ptr::null_mut(),
            directory: std::ptr::null_mut(),
            filename: std::ptr::null_mut(),
            buffer: std::ptr::null_mut(),
            unixpath: std::ptr::null_mut(),
            len: 0,
        };
        let res = parse_path(b"DEV:[dir.subdir]\0".as_ptr() as *const i8, &mut result);
        assert_eq!(res, path_parse_result_t::PATH_PARSE_OK);
        assert_eq!(
            unsafe { CStr::from_ptr(result.device).to_str().unwrap() },
            "DEV"
        );
        assert_eq!(
            unsafe { CStr::from_ptr(result.directory).to_str().unwrap() },
            "dir.subdir"
        );
        assert!(result.filename.is_null());
        path_free(&mut result);
    }

    #[test]
    fn test_parse_path_device_only() {
        let mut result = path_t {
            device: std::ptr::null_mut(),
            directory: std::ptr::null_mut(),
            filename: std::ptr::null_mut(),
            buffer: std::ptr::null_mut(),
            unixpath: std::ptr::null_mut(),
            len: 0,
        };
        let res = parse_path(b"DEV:\0".as_ptr() as *const i8, &mut result);
        assert_eq!(res, path_parse_result_t::PATH_PARSE_OK);
        assert_eq!(
            unsafe { CStr::from_ptr(result.device).to_str().unwrap() },
            "DEV"
        );
        assert!(result.directory.is_null());
        assert!(result.filename.is_null());
        path_free(&mut result);
    }

    #[test]
    fn test_path_dirname() {
        // "DEV:[dir.subdir]file.ext" should return "DEV:[dir.subdir]"
        let input = b"DEV:[dir.subdir]file.ext\0";
        let dirname_ptr = path_dirname(input.as_ptr() as *const i8);
        assert!(!dirname_ptr.is_null());
        let dirname = unsafe { CStr::from_ptr(dirname_ptr) }.to_str().unwrap();
        assert_eq!(dirname, "DEV:[dir.subdir]");
        unsafe { libc::free(dirname_ptr as *mut libc::c_void) };

        // "DEV:file.ext" should return "DEV:"
        let input = b"DEV:file.ext\0";
        let dirname_ptr = path_dirname(input.as_ptr() as *const i8);
        assert!(!dirname_ptr.is_null());
        let dirname = unsafe { CStr::from_ptr(dirname_ptr) }.to_str().unwrap();
        assert_eq!(dirname, "DEV:");
        unsafe { libc::free(dirname_ptr as *mut libc::c_void) };

        // "DEV:[dir]" should return "DEV:[dir]"
        let input = b"DEV:[dir]\0";
        let dirname_ptr = path_dirname(input.as_ptr() as *const i8);
        assert!(!dirname_ptr.is_null());
        let dirname = unsafe { CStr::from_ptr(dirname_ptr) }.to_str().unwrap();
        assert_eq!(dirname, "DEV:[dir]");
        unsafe { libc::free(dirname_ptr as *mut libc::c_void) };

        // "DEV:" should return "DEV:"
        let input = b"DEV:\0";
        let dirname_ptr = path_dirname(input.as_ptr() as *const i8);
        assert!(!dirname_ptr.is_null());
        let dirname = unsafe { CStr::from_ptr(dirname_ptr) }.to_str().unwrap();
        assert_eq!(dirname, "DEV:");
        unsafe { libc::free(dirname_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_basename() {
        // "DEV:[dir.subdir]file.ext" should return "file.ext"
        let input = b"DEV:[dir.subdir]file.ext\0";
        let basename_ptr = path_basename(input.as_ptr() as *const i8);
        assert!(!basename_ptr.is_null());
        let basename = unsafe { CStr::from_ptr(basename_ptr) }.to_str().unwrap();
        assert_eq!(basename, "file.ext");
        unsafe { libc::free(basename_ptr as *mut libc::c_void) };

        // "DEV:file.ext" should return "file.ext"
        let input = b"DEV:file.ext\0";
        let basename_ptr = path_basename(input.as_ptr() as *const i8);
        assert!(!basename_ptr.is_null());
        let basename = unsafe { CStr::from_ptr(basename_ptr) }.to_str().unwrap();
        assert_eq!(basename, "file.ext");
        unsafe { libc::free(basename_ptr as *mut libc::c_void) };

        // "DEV:[dir]" should return "" (empty string, no filename)
        let input = b"DEV:[dir]\0";
        let basename_ptr = path_basename(input.as_ptr() as *const i8);
        assert!(!basename_ptr.is_null());
        let basename = unsafe { CStr::from_ptr(basename_ptr) }.to_str().unwrap();
        assert_eq!(basename, "");
        unsafe { libc::free(basename_ptr as *mut libc::c_void) };

        // "DEV:" should return "" (empty string, no filename)
        let input = b"DEV:\0";
        let basename_ptr = path_basename(input.as_ptr() as *const i8);
        assert!(!basename_ptr.is_null());
        let basename = unsafe { CStr::from_ptr(basename_ptr) }.to_str().unwrap();
        assert_eq!(basename, "");
        unsafe { libc::free(basename_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_basename_full() {
        // "DEV:[dir.subdir]file.ext" should return "file.ext"
        let input = b"DEV:[dir.subdir]file.ext\0";
        let basename_ptr = path_basename(input.as_ptr() as *const i8);
        assert!(!basename_ptr.is_null());
        let basename = unsafe { CStr::from_ptr(basename_ptr) }.to_str().unwrap();
        assert_eq!(basename, "file.ext");
        unsafe { libc::free(basename_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_basename_device_and_file() {
        // "DEV:file.ext" should return "file.ext"
        let input = b"DEV:file.ext\0";
        let basename_ptr = path_basename(input.as_ptr() as *const i8);
        assert!(!basename_ptr.is_null());
        let basename = unsafe { CStr::from_ptr(basename_ptr) }.to_str().unwrap();
        assert_eq!(basename, "file.ext");
        unsafe { libc::free(basename_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_basename_device_directory_only() {
        // "DEV:[dir]" should return "" (empty string, no filename)
        let input = b"DEV:[dir]\0";
        let basename_ptr = path_basename(input.as_ptr() as *const i8);
        assert!(!basename_ptr.is_null());
        let basename = unsafe { CStr::from_ptr(basename_ptr) }.to_str().unwrap();
        assert_eq!(basename, "");
        unsafe { libc::free(basename_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_basename_device_only() {
        // "DEV:" should return "" (empty string, no filename)
        let input = b"DEV:\0";
        let basename_ptr = path_basename(input.as_ptr() as *const i8);
        assert!(!basename_ptr.is_null());
        let basename = unsafe { CStr::from_ptr(basename_ptr) }.to_str().unwrap();
        assert_eq!(basename, "");
        unsafe { libc::free(basename_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_devname_full() {
        // "DEV:[dir.subdir]file.ext" should return "DEV"
        let input = b"DEV:[dir.subdir]file.ext\0";
        let devname_ptr = path_devname(input.as_ptr() as *const i8);
        assert!(!devname_ptr.is_null());
        let devname = unsafe { CStr::from_ptr(devname_ptr) }.to_str().unwrap();
        assert_eq!(devname, "DEV");
        unsafe { libc::free(devname_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_devname_device_and_file() {
        // "DEV:file.ext" should return "DEV"
        let input = b"DEV:file.ext\0";
        let devname_ptr = path_devname(input.as_ptr() as *const i8);
        assert!(!devname_ptr.is_null());
        let devname = unsafe { CStr::from_ptr(devname_ptr) }.to_str().unwrap();
        assert_eq!(devname, "DEV");
        unsafe { libc::free(devname_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_devname_device_directory_only() {
        // "DEV:[dir]" should return "DEV"
        let input = b"DEV:[dir]\0";
        let devname_ptr = path_devname(input.as_ptr() as *const i8);
        assert!(!devname_ptr.is_null());
        let devname = unsafe { CStr::from_ptr(devname_ptr) }.to_str().unwrap();
        assert_eq!(devname, "DEV");
        unsafe { libc::free(devname_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_devname_device_only() {
        // "DEV:" should return "DEV"
        let input = b"DEV:\0";
        let devname_ptr = path_devname(input.as_ptr() as *const i8);
        assert!(!devname_ptr.is_null());
        let devname = unsafe { CStr::from_ptr(devname_ptr) }.to_str().unwrap();
        assert_eq!(devname, "DEV");
        unsafe { libc::free(devname_ptr as *mut libc::c_void) };
    }
}
