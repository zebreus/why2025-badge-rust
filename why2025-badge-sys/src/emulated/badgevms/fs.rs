use crate::{malloc, types::*};
use core::ffi::{CStr, c_char};

/// A valid parse result.
///
/// If you got this structure from the parser the path is guaranteed to be fully valid.
#[derive(Debug)]
struct ParseResult<'a> {
    device: *const u8,
    /// This points to the byte after the device name.
    /// Needs to be set to 0 to make device a valid C string.
    colon: *const u8,
    /// If this pointer is indentical to `colon` there is no directory part.
    opening_bracket: *const u8,
    directory: *const u8,
    /// If not null, this points to the byte after the directory name.
    /// Needs to be set to 0 to make directory a valid C string.
    ///
    /// If this is set the path contains a brackets for the directory part.
    /// If the directory part is empty, this will be set, but `directory` will be a nullptr.
    closing_bracket: *const u8,
    filename: *const u8,
    // This points to the byte after the filename.
    // Opposed to directory_end, this is already guaranteed to be 0.
    terminator: *const u8,
    _lifetime: core::marker::PhantomData<&'a ()>,
}

impl<'a> ParseResult<'a> {
    // Debug asserts everywhere, although they should not be needed as the parser should only ever hand out valid ParseResults.
    //
    // We can use from_utf8_unchecked here because the parser only allows valid UTF-8 characters in paths.

    pub fn device(&self) -> &'a str {
        debug_assert!(!self.device.is_null());
        debug_assert!(!self.colon.is_null());
        let device_length = self.colon as usize - self.device as usize;
        unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.device, device_length))
        }
    }
    pub fn colon(&self) -> &'a [u8] {
        debug_assert!(!self.colon.is_null());
        std::array::from_ref(unsafe { &*self.colon })
    }
    pub fn opening_bracket(&self) -> Option<&'a u8> {
        if self.opening_bracket.is_null() {
            return None;
        }
        Some(unsafe { &*self.opening_bracket })
    }
    pub fn closing_bracket(&self) -> Option<&'a u8> {
        if self.closing_bracket.is_null() {
            return None;
        }
        Some(unsafe { &*self.closing_bracket })
    }
    pub fn directory(&self) -> Option<&'a str> {
        if self.directory.is_null() || self.closing_bracket.is_null() {
            return None;
        }
        let directory_length = self.closing_bracket as usize - self.directory as usize;
        unsafe {
            Some(std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                self.directory,
                directory_length,
            )))
        }
    }
    pub fn filename(&self) -> Option<&'a str> {
        if self.filename.is_null() {
            return None;
        }
        debug_assert!(!self.terminator.is_null());
        let filename_length = self.terminator as usize - self.filename as usize;
        Some(unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                self.filename,
                filename_length,
            ))
        })
    }
    pub fn terminator(&self) -> &'a u8 {
        debug_assert!(!self.terminator.is_null());
        unsafe { &*self.terminator }
    }

    /// Set the null terminators in the original string to sure that `device`, `directory` and `filename` are valid C strings.
    ///
    /// This is unsafe because it modifies the original string even if it is not mutable.
    unsafe fn finalize(&self) {
        unsafe {
            if !self.colon.is_null() {
                *(self.colon as *mut u8) = 0;
            }
            if !self.closing_bracket.is_null() {
                *(self.closing_bracket as *mut u8) = 0;
            }
        }
    }
}

// impl ParseResult {
//     fn into_better_format(&self) {
//         debug_assert!(!self.device.is_null());
//         let device_len = self.device.

//     }
// }

// struct BetterDirectoryPart<'a> {
//     opening_bracket: &'a [u8; 1],
//     directory: &'a [u8],
//     closing_bracket: &'a [u8; 1],
// }
// struct BetterParseResult<'a> {
//     device: &'a [u8],
//     colon: &'a [u8; 1],
//     // Possibly a directory part with brackets
//     directory: Option<BetterDirectoryPart<'a>>,
//     // A character sequence of 0 or more bytes.
//     filename: &'a [u8],
//     // Guaranteed to be a single null byte
//     terminator: &'a [u8; 1],
// }

fn parse_path_internal<'a>(
    input: &'a core::ffi::CStr,
) -> Result<ParseResult<'a>, path_parse_result_t> {
    enum ParseState<'a> {
        Device(ParseResult<'a>),
        Filename(ParseResult<'a>),
        MaybeDirectory(ParseResult<'a>),
        Directory(ParseResult<'a>),
        Done(ParseResult<'a>),
    }
    let mut state = ParseState::Device(ParseResult {
        device: std::ptr::null_mut(),
        colon: std::ptr::null_mut(),
        opening_bracket: std::ptr::null_mut(),
        directory: std::ptr::null_mut(),
        closing_bracket: std::ptr::null_mut(),
        filename: std::ptr::null_mut(),
        terminator: std::ptr::null_mut(),
        _lifetime: core::marker::PhantomData,
    });
    for c in input.to_bytes_with_nul().iter() {
        state = match (state, *c) {
            (ParseState::Done(_), _) => {
                unreachable!(
                    "Encountered another byte after null terminator. This should not be possible here."
                );
            }
            (ParseState::Filename(r) | ParseState::MaybeDirectory(r), 0) => {
                ParseState::Done(ParseResult { terminator: c, ..r })
            }
            (ParseState::Directory(_), 0) => {
                return Err(path_parse_result_t::PATH_PARSE_UNCLOSED_DIRECTORY);
            }
            (ParseState::Device(_), 0) => {
                return Err(path_parse_result_t::PATH_PARSE_NO_DEVICE);
            }
            (ParseState::Device(r), b':') => {
                if r.device.is_null() {
                    return Err(path_parse_result_t::PATH_PARSE_EMPTY_DEVICE);
                }
                ParseState::MaybeDirectory(ParseResult { colon: c, ..r })
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
            (ParseState::MaybeDirectory(r), b'[') => ParseState::Directory(ParseResult {
                opening_bracket: c,
                ..r
            }),
            (ParseState::Directory(r), b']') => ParseState::Filename(ParseResult {
                closing_bracket: c,
                ..r
            }),
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

    let ParseState::Done(result) = state else {
        unreachable!("Path parsing did not end in a done state. This is not possible.")
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
pub extern "C" fn parse_path(path: *const c_char, result: *mut path_t) -> path_parse_result_t {
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
        buffer: copied_buffer.as_ptr() as *mut c_char,
        unixpath: std::ptr::null_mut(),
        len: c_str.to_bytes().len(),
    };

    let parse_result = match parse_path_internal(&copied_buffer) {
        Ok(r) => r,
        Err(e) => return e,
    };
    unsafe { parse_result.finalize() };

    *result = path_t {
        device: parse_result.device as *mut c_char,
        directory: parse_result.directory as *mut c_char,
        filename: parse_result.filename as *mut c_char,
        buffer: copied_buffer.as_ptr() as *mut c_char,
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
pub extern "C" fn mkdir_p(_path: *const c_char) -> bool {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn rm_rf(_path: *const c_char) -> bool {
    unimplemented!("Implement this yourself if you need it");
}

/// Get the device name from a path
///
/// Keep in mind that you must free the returned string with `free` when you are done with it.
#[unsafe(no_mangle)]
pub extern "C" fn path_dirname(path: *const c_char) -> *mut c_char {
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
    let path_end = if !parse_result.closing_bracket.is_null() {
        parse_result.closing_bracket
    } else {
        parse_result.colon
    };

    let path_len = (path_end as u32 - path_start as u32) + 1;
    let result_buffer = unsafe { malloc(path_len + 1) as *mut u8 };
    unsafe { core::ptr::copy_nonoverlapping(path_start, result_buffer, path_len as usize) };
    unsafe { *result_buffer.byte_add(path_len as usize) = 0 };
    result_buffer as *mut c_char
}

/// Get the device name from a path
///
/// Keep in mind that you must free the returned string with `free` when you are done with it.
#[unsafe(no_mangle)]
pub extern "C" fn path_basename(path: *const c_char) -> *mut c_char {
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
    let path_start = if !parse_result.closing_bracket.is_null() {
        unsafe { parse_result.closing_bracket.byte_add(1) }
    } else {
        unsafe { parse_result.colon.byte_add(1) }
    };

    let path_len = (path_end as u32 - path_start as u32) + 1;
    let result_buffer = unsafe { malloc(path_len + 1) as *mut u8 };
    unsafe { core::ptr::copy_nonoverlapping(path_start, result_buffer, path_len as usize) };
    unsafe { *result_buffer.byte_add(path_len as usize) = 0 };
    result_buffer as *mut c_char
}

/// Get the device name from a path
///
/// Keep in mind that you must free the returned string with `free` when you are done with it.
#[unsafe(no_mangle)]
pub extern "C" fn path_devname(path: *const c_char) -> *mut c_char {
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
    let path_end = parse_result.colon;

    let path_len = path_end as u32 - path_start as u32;
    let result_buffer = unsafe { malloc(path_len + 1) as *mut u8 };
    unsafe { core::ptr::copy_nonoverlapping(path_start, result_buffer, path_len as usize) };
    unsafe { *result_buffer.byte_add(path_len as usize) = 0 };
    result_buffer as *mut c_char
}

/// Takes a path (in the form described in the documentation of `parse_path`) and a subdirectory name
///
/// If the path already contains a directory and returns a newly allocated string with the result.
///
/// If you input something like "DEV:file.ext" and "subdir", you will get "DEV:[subdir]file.ext"
/// If you input "DEV:[dir]file.ext" and "subdir", you will get "DEV:[dir.subdir]file.ext"
///
/// Keep in mind that you must free the returned string with `free` when you are done with it.
#[unsafe(no_mangle)]
pub extern "C" fn path_dirconcat(path: *const c_char, subdir: *const c_char) -> *mut c_char {
    if path.is_null() {
        return core::ptr::null_mut();
    }
    let path_c_str = unsafe { ::core::ffi::CStr::from_ptr(path) };
    if path_c_str.to_bytes().len() == 0 {
        return core::ptr::null_mut();
    }

    if subdir.is_null() {
        return core::ptr::null_mut();
    }
    let subdir_c_str = unsafe { ::core::ffi::CStr::from_ptr(subdir) };
    let subdir = std::str::from_utf8(subdir_c_str.to_bytes()).expect("Subdir is not valid UTF-8. While you can call this function with any c_str like thing on badgevms, it will cause errors down the line. If you need this function to work with non UTF-8 subdirectories, implement it yourself.");
    if subdir.len() == 0 {
        return core::ptr::null_mut();
    }

    let parse_result = match parse_path_internal(path_c_str) {
        Ok(r) => r,
        Err(_) => return core::ptr::null_mut(),
    };

    // I first wrote had a low level implemetation without format and additional allocations.
    // Removed it because I need to remember that I am not writing C code for embedded and that we have enough power to do this on a high level
    let old_directory = if let Some(d) = parse_result.directory()
        && !d.is_empty()
    {
        format!("{}.", d)
    } else {
        String::new()
    };
    let result_string = format!(
        "{}:[{}{}]{}",
        parse_result.device(),
        old_directory,
        subdir,
        parse_result.filename().unwrap_or(""),
    );

    unsafe {
        let result_buffer = malloc(result_string.len() as u32 + 1) as *mut u8;
        std::ptr::copy_nonoverlapping(result_string.as_ptr(), result_buffer, result_string.len());
        *result_buffer.add(result_string.len()) = 0;
        result_buffer as *mut c_char
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn path_fileconcat(_path: *const c_char, _filename: *const c_char) -> *mut c_char {
    unimplemented!("Implement this yourself if you need it");
}
#[unsafe(no_mangle)]
pub extern "C" fn path_concat(
    _base_path: *const c_char,
    _append_path: *const c_char,
) -> *mut c_char {
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
            b"DEV:[dir.subdir]file.ext\0".as_ptr() as *const c_char,
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
        let res = parse_path(b"DEV:file.ext\0".as_ptr() as *const c_char, &mut result);
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
        let res = parse_path(
            b"DEV:[dir]file.ext\0".as_ptr() as *const c_char,
            &mut result,
        );
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
        let res = parse_path(b"DEV:[dir.subdir]\0".as_ptr() as *const c_char, &mut result);
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
        let res = parse_path(b"DEV:\0".as_ptr() as *const c_char, &mut result);
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
        let dirname_ptr = path_dirname(input.as_ptr() as *const c_char);
        assert!(!dirname_ptr.is_null());
        let dirname = unsafe { CStr::from_ptr(dirname_ptr) }.to_str().unwrap();
        assert_eq!(dirname, "DEV:[dir.subdir]");
        unsafe { libc::free(dirname_ptr as *mut libc::c_void) };

        // "DEV:file.ext" should return "DEV:"
        let input = b"DEV:file.ext\0";
        let dirname_ptr = path_dirname(input.as_ptr() as *const c_char);
        assert!(!dirname_ptr.is_null());
        let dirname = unsafe { CStr::from_ptr(dirname_ptr) }.to_str().unwrap();
        assert_eq!(dirname, "DEV:");
        unsafe { libc::free(dirname_ptr as *mut libc::c_void) };

        // "DEV:[dir]" should return "DEV:[dir]"
        let input = b"DEV:[dir]\0";
        let dirname_ptr = path_dirname(input.as_ptr() as *const c_char);
        assert!(!dirname_ptr.is_null());
        let dirname = unsafe { CStr::from_ptr(dirname_ptr) }.to_str().unwrap();
        assert_eq!(dirname, "DEV:[dir]");
        unsafe { libc::free(dirname_ptr as *mut libc::c_void) };

        // "DEV:" should return "DEV:"
        let input = b"DEV:\0";
        let dirname_ptr = path_dirname(input.as_ptr() as *const c_char);
        assert!(!dirname_ptr.is_null());
        let dirname = unsafe { CStr::from_ptr(dirname_ptr) }.to_str().unwrap();
        assert_eq!(dirname, "DEV:");
        unsafe { libc::free(dirname_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_basename() {
        // "DEV:[dir.subdir]file.ext" should return "file.ext"
        let input = b"DEV:[dir.subdir]file.ext\0";
        let basename_ptr = path_basename(input.as_ptr() as *const c_char);
        assert!(!basename_ptr.is_null());
        let basename = unsafe { CStr::from_ptr(basename_ptr) }.to_str().unwrap();
        assert_eq!(basename, "file.ext");
        unsafe { libc::free(basename_ptr as *mut libc::c_void) };

        // "DEV:file.ext" should return "file.ext"
        let input = b"DEV:file.ext\0";
        let basename_ptr = path_basename(input.as_ptr() as *const c_char);
        assert!(!basename_ptr.is_null());
        let basename = unsafe { CStr::from_ptr(basename_ptr) }.to_str().unwrap();
        assert_eq!(basename, "file.ext");
        unsafe { libc::free(basename_ptr as *mut libc::c_void) };

        // "DEV:[dir]" should return "" (empty string, no filename)
        let input = b"DEV:[dir]\0";
        let basename_ptr = path_basename(input.as_ptr() as *const c_char);
        assert!(!basename_ptr.is_null());
        let basename = unsafe { CStr::from_ptr(basename_ptr) }.to_str().unwrap();
        assert_eq!(basename, "");
        unsafe { libc::free(basename_ptr as *mut libc::c_void) };

        // "DEV:" should return "" (empty string, no filename)
        let input = b"DEV:\0";
        let basename_ptr = path_basename(input.as_ptr() as *const c_char);
        assert!(!basename_ptr.is_null());
        let basename = unsafe { CStr::from_ptr(basename_ptr) }.to_str().unwrap();
        assert_eq!(basename, "");
        unsafe { libc::free(basename_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_basename_full() {
        // "DEV:[dir.subdir]file.ext" should return "file.ext"
        let input = b"DEV:[dir.subdir]file.ext\0";
        let basename_ptr = path_basename(input.as_ptr() as *const c_char);
        assert!(!basename_ptr.is_null());
        let basename = unsafe { CStr::from_ptr(basename_ptr) }.to_str().unwrap();
        assert_eq!(basename, "file.ext");
        unsafe { libc::free(basename_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_basename_device_and_file() {
        // "DEV:file.ext" should return "file.ext"
        let input = b"DEV:file.ext\0";
        let basename_ptr = path_basename(input.as_ptr() as *const c_char);
        assert!(!basename_ptr.is_null());
        let basename = unsafe { CStr::from_ptr(basename_ptr) }.to_str().unwrap();
        assert_eq!(basename, "file.ext");
        unsafe { libc::free(basename_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_basename_device_directory_only() {
        // "DEV:[dir]" should return "" (empty string, no filename)
        let input = b"DEV:[dir]\0";
        let basename_ptr = path_basename(input.as_ptr() as *const c_char);
        assert!(!basename_ptr.is_null());
        let basename = unsafe { CStr::from_ptr(basename_ptr) }.to_str().unwrap();
        assert_eq!(basename, "");
        unsafe { libc::free(basename_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_basename_device_only() {
        // "DEV:" should return "" (empty string, no filename)
        let input = b"DEV:\0";
        let basename_ptr = path_basename(input.as_ptr() as *const c_char);
        assert!(!basename_ptr.is_null());
        let basename = unsafe { CStr::from_ptr(basename_ptr) }.to_str().unwrap();
        assert_eq!(basename, "");
        unsafe { libc::free(basename_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_devname_full() {
        // "DEV:[dir.subdir]file.ext" should return "DEV"
        let input = b"DEV:[dir.subdir]file.ext\0";
        let devname_ptr = path_devname(input.as_ptr() as *const c_char);
        assert!(!devname_ptr.is_null());
        let devname = unsafe { CStr::from_ptr(devname_ptr) }.to_str().unwrap();
        assert_eq!(devname, "DEV");
        unsafe { libc::free(devname_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_devname_device_and_file() {
        // "DEV:file.ext" should return "DEV"
        let input = b"DEV:file.ext\0";
        let devname_ptr = path_devname(input.as_ptr() as *const c_char);
        assert!(!devname_ptr.is_null());
        let devname = unsafe { CStr::from_ptr(devname_ptr) }.to_str().unwrap();
        assert_eq!(devname, "DEV");
        unsafe { libc::free(devname_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_devname_device_directory_only() {
        // "DEV:[dir]" should return "DEV"
        let input = b"DEV:[dir]\0";
        let devname_ptr = path_devname(input.as_ptr() as *const c_char);
        assert!(!devname_ptr.is_null());
        let devname = unsafe { CStr::from_ptr(devname_ptr) }.to_str().unwrap();
        assert_eq!(devname, "DEV");
        unsafe { libc::free(devname_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_devname_device_only() {
        // "DEV:" should return "DEV"
        let input = b"DEV:\0";
        let devname_ptr = path_devname(input.as_ptr() as *const c_char);
        assert!(!devname_ptr.is_null());
        let devname = unsafe { CStr::from_ptr(devname_ptr) }.to_str().unwrap();
        assert_eq!(devname, "DEV");
        unsafe { libc::free(devname_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_dirconcat_add_to_file_only() {
        // "DEV:file.ext" + "subdir" -> "DEV:[subdir]file.ext"
        let input = b"DEV:file.ext\0";
        let subdir = b"subdir\0";
        let out_ptr = path_dirconcat(
            input.as_ptr() as *const c_char,
            subdir.as_ptr() as *const c_char,
        );
        assert!(!out_ptr.is_null());
        let out = unsafe { CStr::from_ptr(out_ptr) }.to_str().unwrap();
        assert_eq!(out, "DEV:[subdir]file.ext");
        unsafe { libc::free(out_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_dirconcat_add_to_existing_dir() {
        // "DEV:[dir]file.ext" + "subdir" -> "DEV:[dir.subdir]file.ext"
        let input = b"DEV:[dir]file.ext\0";
        let subdir = b"subdir\0";
        let out_ptr = path_dirconcat(
            input.as_ptr() as *const c_char,
            subdir.as_ptr() as *const c_char,
        );
        assert!(!out_ptr.is_null());
        let out = unsafe { CStr::from_ptr(out_ptr) }.to_str().unwrap();
        assert_eq!(out, "DEV:[dir.subdir]file.ext");
        unsafe { libc::free(out_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_dirconcat_add_to_existing_dir_with_subdir() {
        // "DEV:[dir.sub]file.ext" + "subdir" -> "DEV:[dir.sub.subdir]file.ext"
        let input = b"DEV:[dir.sub]file.ext\0";
        let subdir = b"subdir\0";
        let out_ptr = path_dirconcat(
            input.as_ptr() as *const c_char,
            subdir.as_ptr() as *const c_char,
        );
        assert!(!out_ptr.is_null());
        let out = unsafe { CStr::from_ptr(out_ptr) }.to_str().unwrap();
        assert_eq!(out, "DEV:[dir.sub.subdir]file.ext");
        unsafe { libc::free(out_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_dirconcat_add_to_dir_only() {
        // "DEV:[dir]\0" + "subdir" -> "DEV:[dir.subdir]"
        let input = b"DEV:[dir]\0";
        let subdir = b"subdir\0";
        let out_ptr = path_dirconcat(
            input.as_ptr() as *const c_char,
            subdir.as_ptr() as *const c_char,
        );
        assert!(!out_ptr.is_null());
        let out = unsafe { CStr::from_ptr(out_ptr) }.to_str().unwrap();
        assert_eq!(out, "DEV:[dir.subdir]");
        unsafe { libc::free(out_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_dirconcat_add_to_device_only() {
        // "DEV:\0" + "subdir" -> "DEV:[subdir]"
        let input = b"DEV:\0";
        let subdir = b"subdir\0";
        let out_ptr = path_dirconcat(
            input.as_ptr() as *const c_char,
            subdir.as_ptr() as *const c_char,
        );
        assert!(!out_ptr.is_null());
        let out = unsafe { CStr::from_ptr(out_ptr) }.to_str().unwrap();
        assert_eq!(out, "DEV:[subdir]");
        unsafe { libc::free(out_ptr as *mut libc::c_void) };
    }
}
