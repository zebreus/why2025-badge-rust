use crate::{emulated::badgevms::fs::paths::ParsedPath, malloc, types::*};
use core::ffi::c_char;
use std::fs;

pub(crate) mod paths;

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
///
/// MISMATCH: The badgevms implementation may return a half-populated path_t on error, while in this implementation it will either be fully populated or filled with nullptrs.
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

    let parse_result = match ParsedPath::new(&c_str) {
        Ok(r) => r,
        Err(e) => {
            e.populate_path_t(result);
            return (&e).into();
        }
    };

    parse_result.populate_path_t(result);
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

/// Create a directory and all parent directories if they do not exist
///
/// Returns true on success, false on failure (including if the path is invalid)
#[unsafe(no_mangle)]
pub extern "C" fn mkdir_p(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }
    let c_str = unsafe { ::core::ffi::CStr::from_ptr(path) };
    if c_str.to_bytes().len() == 0 {
        return false;
    }

    let parsed_path = match ParsedPath::new(&c_str) {
        Ok(parsed_path) => parsed_path,
        Err(_) => return false,
    };

    fs::create_dir_all(parsed_path.to_host_directory()).is_ok()
}

#[unsafe(no_mangle)]
pub extern "C" fn rm_rf(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }
    let c_str = unsafe { ::core::ffi::CStr::from_ptr(path) };
    if c_str.to_bytes().len() == 0 {
        return false;
    }

    let parsed_path = match ParsedPath::new(&c_str) {
        Ok(parsed_path) => parsed_path,
        Err(_) => return true,
    };

    let host_path = parsed_path.to_host_file();
    let metadata = match fs::metadata(&host_path) {
        Ok(metadata) => metadata,
        Err(error) => return error.kind() == std::io::ErrorKind::NotFound,
    };

    if metadata.is_dir() {
        fs::remove_dir_all(host_path).is_ok()
    } else {
        fs::remove_file(host_path).is_ok()
    }
}

/// Get the filename name from a path
///
/// Keep in mind that you must free the returned string with `free` when you are done with it.
///
/// Returns a nullptr if an error occurs
#[unsafe(no_mangle)]
pub extern "C" fn path_dirname(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return core::ptr::null_mut();
    }
    let c_str = unsafe { ::core::ffi::CStr::from_ptr(path) };
    if c_str.to_bytes().len() == 0 {
        return core::ptr::null_mut();
    }

    let mut parse_result = match ParsedPath::new(c_str) {
        Ok(r) => r,
        Err(_) => return core::ptr::null_mut(),
    };
    parse_result.filename = String::new();
    let dirname = format!("{}", parse_result);

    unsafe {
        // Allocate a buffer with `malloc` so the caller can free it with `free`
        let buffer = malloc(dirname.len() as u32 + 1) as *mut c_char;
        core::ptr::copy_nonoverlapping(dirname.as_ptr() as *mut c_char, buffer, dirname.len());
        *buffer.add(dirname.len()) = 0;
        buffer
    }
}

/// Get the device name from a path
///
/// Keep in mind that you must free the returned string with `free` when you are done with it.
///
/// Returns a nullptr if an error occurs or if the path does not contain a filename.
#[unsafe(no_mangle)]
pub extern "C" fn path_basename(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return core::ptr::null_mut();
    }
    let c_str = unsafe { ::core::ffi::CStr::from_ptr(path) };
    if c_str.to_bytes().len() == 0 {
        return core::ptr::null_mut();
    }

    let parse_result = match ParsedPath::new(c_str) {
        Ok(r) => r,
        Err(_) => return core::ptr::null_mut(),
    };
    let basename = parse_result.filename;
    if basename.is_empty() {
        return core::ptr::null_mut();
    }

    unsafe {
        // Allocate a buffer with `malloc` so the caller can free it with `free`
        let buffer = malloc(basename.len() as u32 + 1) as *mut c_char;
        core::ptr::copy_nonoverlapping(basename.as_ptr() as *mut c_char, buffer, basename.len());
        *buffer.add(basename.len()) = 0;
        buffer
    }
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

    let mut parse_result = match ParsedPath::new(c_str) {
        Ok(r) => r,
        Err(_) => return core::ptr::null_mut(),
    };
    parse_result.filename = String::new();
    parse_result.directory = None;
    let devname = format!("{}", parse_result);

    unsafe {
        // Allocate a buffer with `malloc` so the caller can free it with `free`
        let buffer = malloc(devname.len() as u32 + 1) as *mut c_char;
        core::ptr::copy_nonoverlapping(devname.as_ptr() as *mut c_char, buffer, devname.len());
        *buffer.add(devname.len()) = 0;
        buffer
    }
}

/// Takes a path (in the form described in the documentation of `parse_path`) and a subdirectory name
///
/// If the path already contains a directory and returns a newly allocated string with the result.
///
/// If you input something like "DEV:file.ext" and "subdir", you will get "DEV:[subdir]file.ext"
/// If you input "DEV:[dir]file.ext" and "subdir", you will get "DEV:[dir.subdir]file.ext"
///
/// Keep in mind that you must free the returned string with `free` when you are done with it.
///
/// Returns a nullptr if an error occurs
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

    let mut parse_result = match ParsedPath::new(path_c_str) {
        Ok(r) => r,
        Err(_) => return core::ptr::null_mut(),
    };

    if let Some(d) = parse_result.directory.filter(|d| d.len() > 0) {
        parse_result.directory = Some(format!("{}.{}", d, subdir));
    } else {
        parse_result.directory = Some(subdir.to_string());
    }

    // I first wrote had a low level implemetation without format and additional allocations.
    // Removed it because I need to remember that I am not writing C code for embedded and that we have enough power to do this on a high level

    let result = format! {"{}", parse_result};

    unsafe {
        // Allocate a buffer with `malloc`` so the caller can free it with `free``
        let buffer = malloc(result.len() as u32 + 1) as *mut c_char;
        core::ptr::copy_nonoverlapping(result.as_ptr() as *mut c_char, buffer, result.len());
        *buffer.add(result.len()) = 0;
        buffer
    }
}

/// Replace the filename of a path with a new filename
///
/// Keep in mind that you must free the returned string with `free` when you are done with it.
///
/// Returns a nullptr if an error occurs
// TODO: Better document error conditions
#[unsafe(no_mangle)]
pub extern "C" fn path_fileconcat(path: *const c_char, filename: *const c_char) -> *mut c_char {
    if path.is_null() {
        return core::ptr::null_mut();
    }
    let path_c_str = unsafe { ::core::ffi::CStr::from_ptr(path) };
    if path_c_str.to_bytes().len() == 0 {
        return core::ptr::null_mut();
    }

    if filename.is_null() {
        return core::ptr::null_mut();
    }
    let filename_c_str = unsafe { ::core::ffi::CStr::from_ptr(filename) };
    let filename = std::str::from_utf8(filename_c_str.to_bytes()).expect("Filename is not valid UTF-8. While you can call this function with any c_str like thing on badgevms, it will cause errors down the line. If you need this function to work with non UTF-8 filenameectories, implement it yourself.");
    if filename.len() == 0 {
        return core::ptr::null_mut();
    }

    let mut parse_result = match ParsedPath::new(path_c_str) {
        Ok(r) => r,
        Err(_) => return core::ptr::null_mut(),
    };
    parse_result.filename = filename.to_string();

    let result = format! {"{}", parse_result};

    unsafe {
        // Allocate a buffer with `malloc`` so the caller can free it with `free``
        let buffer = malloc(result.len() as u32 + 1) as *mut c_char;
        core::ptr::copy_nonoverlapping(result.as_ptr() as *mut c_char, buffer, result.len());
        *buffer.add(result.len()) = 0;
        buffer
    }
}

/// Concatenate two paths
///
/// The `base_path` may not have a filename.
/// The `append_path` may not have a device name. It must start with either a `[` for a directory or a filename directly.
///
/// Keep in mind that you must free the returned string with `free` when you are done with it.
///
/// Returns a nullptr if an error occurs
#[unsafe(no_mangle)]
pub extern "C" fn path_concat(base_path: *const c_char, append_path: *const c_char) -> *mut c_char {
    if base_path.is_null() {
        return core::ptr::null_mut();
    }
    let base_path_c_str = unsafe { ::core::ffi::CStr::from_ptr(base_path) };
    if base_path_c_str.to_bytes().len() == 0 {
        return core::ptr::null_mut();
    }

    if append_path.is_null() {
        return core::ptr::null_mut();
    }
    let append_path_c_str = unsafe { ::core::ffi::CStr::from_ptr(append_path) };
    if append_path_c_str.to_bytes().len() == 0 {
        return core::ptr::null_mut();
    }

    let base_parsed = match ParsedPath::new(base_path_c_str) {
        Ok(r) => r,
        Err(_) => return core::ptr::null_mut(),
    };
    if base_parsed.filename.len() > 0 {
        // Base path must not have a filename
        return core::ptr::null_mut();
    }

    // Append a fake device name so we can parse it. This also ensures that the append path does not have a device name as two devices would make the path invalid.
    let mut append_path_with_device: Vec<u8> = Vec::from(b"X:");
    append_path_with_device.extend_from_slice(append_path_c_str.to_bytes_with_nul());
    let append_path_c_str =
        ::core::ffi::CStr::from_bytes_with_nul(&append_path_with_device).unwrap();
    let append_parsed = match ParsedPath::new(append_path_c_str) {
        Ok(r) => r,
        Err(_) => return core::ptr::null_mut(),
    };

    let maybe_base_dir = base_parsed.directory.filter(|d| d.len() > 0);
    let maybe_append_dir = append_parsed.directory.filter(|d| d.len() > 0);

    let new_directory = match (maybe_base_dir, maybe_append_dir) {
        (Some(b), Some(a)) => Some(format!("{}.{}", b, a)),
        (Some(d), None) => Some(d),
        (None, Some(d)) => Some(d),
        (None, None) => None,
    };
    let parsed_path = ParsedPath {
        device: base_parsed.device,
        directory: new_directory,
        filename: append_parsed.filename,
    };

    let result = format! {"{}", parsed_path};

    unsafe {
        // Allocate a buffer with `malloc`` so the caller can free it with `free``
        let buffer = malloc(result.len() as u32 + 1) as *mut c_char;
        core::ptr::copy_nonoverlapping(result.as_ptr() as *mut c_char, buffer, result.len());
        *buffer.add(result.len()) = 0;
        buffer
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::CStr, fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

    use super::*;

    struct TemporaryTestDirectory {
        path: PathBuf,
    }

    impl TemporaryTestDirectory {
        fn new(test_name: &str) -> Self {
            let unique_suffix = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after the unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "why2025-badge-sys-{test_name}-{}-{unique_suffix}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&path);
            Self { path }
        }

        fn path(&self) -> &PathBuf {
            &self.path
        }
    }

    impl Drop for TemporaryTestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn test_mkdir_p_creates_nested_host_directories() {
        let temporary_directory = TemporaryTestDirectory::new("mkdir-p-nested");
        let _base_directory = super::paths::set_base_directory_for_tests(
            temporary_directory.path().clone(),
        );

        assert!(mkdir_p(b"DEV:[dir.subdir]file.ext\0".as_ptr() as *const c_char));
        assert!(temporary_directory.path().join("DEV").join("dir").join("subdir").is_dir());

        assert!(mkdir_p(b"DEV:[dir.subdir]file.ext\0".as_ptr() as *const c_char));
    }

    #[test]
    fn test_mkdir_p_rejects_invalid_paths() {
        let temporary_directory = TemporaryTestDirectory::new("mkdir-p-invalid");
        let _base_directory = super::paths::set_base_directory_for_tests(
            temporary_directory.path().clone(),
        );

        assert!(!mkdir_p(core::ptr::null()));
        assert!(!mkdir_p(b"[dir.subdir]file.ext\0".as_ptr() as *const c_char));
        assert!(!temporary_directory.path().exists());
    }

    #[test]
    fn test_rm_rf_removes_file_and_missing_target_is_success() {
        let temporary_directory = TemporaryTestDirectory::new("rm-rf-file");
        let _base_directory = super::paths::set_base_directory_for_tests(
            temporary_directory.path().clone(),
        );

        let file_path = CStr::from_bytes_with_nul(b"DEV:[dir.subdir]file.ext\0").unwrap();
        let parsed_path = match ParsedPath::new(file_path) {
            Ok(parsed_path) => parsed_path,
            Err(_) => panic!("test setup should use a valid badge file path"),
        };
        fs::create_dir_all(parsed_path.to_host_directory()).unwrap();
        fs::write(parsed_path.to_host_file(), b"hello").unwrap();

        assert!(rm_rf(file_path.as_ptr()));
        assert!(!parsed_path.to_host_file().exists());

        assert!(rm_rf(file_path.as_ptr()));
    }

    #[test]
    fn test_rm_rf_removes_directories_recursively() {
        let temporary_directory = TemporaryTestDirectory::new("rm-rf-directory");
        let _base_directory = super::paths::set_base_directory_for_tests(
            temporary_directory.path().clone(),
        );

        let directory_path = CStr::from_bytes_with_nul(b"DEV:[dir.subdir]\0").unwrap();
        let parsed_path = match ParsedPath::new(directory_path) {
            Ok(parsed_path) => parsed_path,
            Err(_) => panic!("test setup should use a valid badge directory path"),
        };
        let host_directory = parsed_path.to_host_directory();
        fs::create_dir_all(host_directory.join("nested")).unwrap();
        fs::write(host_directory.join("child.txt"), b"child").unwrap();
        fs::write(
            host_directory.join("nested").join("grandchild.txt"),
            b"grandchild",
        )
        .unwrap();

        assert!(rm_rf(directory_path.as_ptr()));
        assert!(!host_directory.exists());
    }

    #[test]
    fn test_rm_rf_rejects_empty_input_and_ignores_invalid_paths() {
        assert!(!rm_rf(core::ptr::null()));
        assert!(!rm_rf(b"\0".as_ptr() as *const c_char));
        assert!(rm_rf(b"[dir.subdir]file.ext\0".as_ptr() as *const c_char));
    }

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
        assert!(basename_ptr.is_null());
        unsafe { libc::free(basename_ptr as *mut libc::c_void) };

        // "DEV:" should return "" (empty string, no filename)
        let input = b"DEV:\0";
        let basename_ptr = path_basename(input.as_ptr() as *const c_char);
        assert!(basename_ptr.is_null());
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
        assert!(basename_ptr.is_null());
        unsafe { libc::free(basename_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_basename_device_only() {
        // "DEV:" should return a nullptr
        let input = b"DEV:\0";
        let basename_ptr = path_basename(input.as_ptr() as *const c_char);
        assert!(basename_ptr.is_null());
        unsafe { libc::free(basename_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_devname_full() {
        // "DEV:[dir.subdir]file.ext" should return "DEV:"
        let input = b"DEV:[dir.subdir]file.ext\0";
        let devname_ptr = path_devname(input.as_ptr() as *const c_char);
        assert!(!devname_ptr.is_null());
        let devname = unsafe { CStr::from_ptr(devname_ptr) }.to_str().unwrap();
        assert_eq!(devname, "DEV:");
        unsafe { libc::free(devname_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_devname_device_and_file() {
        // "DEV:file.ext" should return "DEV:"
        let input = b"DEV:file.ext\0";
        let devname_ptr = path_devname(input.as_ptr() as *const c_char);
        assert!(!devname_ptr.is_null());
        let devname = unsafe { CStr::from_ptr(devname_ptr) }.to_str().unwrap();
        assert_eq!(devname, "DEV:");
        unsafe { libc::free(devname_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_devname_device_directory_only() {
        // "DEV:[dir]" should return "DEV:"
        let input = b"DEV:[dir]\0";
        let devname_ptr = path_devname(input.as_ptr() as *const c_char);
        assert!(!devname_ptr.is_null());
        let devname = unsafe { CStr::from_ptr(devname_ptr) }.to_str().unwrap();
        assert_eq!(devname, "DEV:");
        unsafe { libc::free(devname_ptr as *mut libc::c_void) };
    }

    #[test]
    fn test_path_devname_device_only() {
        // "DEV:" should return "DEV:"
        let input = b"DEV:\0";
        let devname_ptr = path_devname(input.as_ptr() as *const c_char);
        assert!(!devname_ptr.is_null());
        let devname = unsafe { CStr::from_ptr(devname_ptr) }.to_str().unwrap();
        assert_eq!(devname, "DEV:");
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
