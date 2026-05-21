use crate::types::*;
use alloc::{
    ffi::CString,
    format,
    string::{String, ToString},
    vec::Vec,
};
use core::{
    ffi::{c_char, c_int, CStr},
    ptr,
};

mod paths;

use paths::ParsedPath;

struct DirectoryHandle(*mut libc::DIR);

impl Drop for DirectoryHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = libc::closedir(self.0);
        }
    }
}

fn current_errno() -> c_int {
    unsafe { *libc::__errno_location() }
}

fn empty_path() -> path_t {
    path_t {
        buffer: ptr::null_mut(),
        device: ptr::null_mut(),
        directory: ptr::null_mut(),
        filename: ptr::null_mut(),
        unixpath: ptr::null_mut(),
        len: 0,
    }
}

fn allocate_result_string(bytes: &[u8]) -> *mut c_char {
    let Some(allocation_size) = bytes.len().checked_add(1) else {
        return ptr::null_mut();
    };

    let buffer = unsafe { libc::malloc(allocation_size) }.cast::<c_char>();
    if buffer.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr().cast::<c_char>(), buffer, bytes.len());
        *buffer.add(bytes.len()) = 0;
    }

    buffer
}

fn to_c_string(bytes: &[u8]) -> Result<CString, c_int> {
    CString::new(bytes.to_vec()).map_err(|_| libc::EINVAL)
}

fn create_dir_all(path: &[u8]) -> Result<(), c_int> {
    if path.is_empty() {
        return Ok(());
    }

    let mut current = Vec::new();
    let mut index = 0usize;
    if path.first() == Some(&b'/') {
        current.push(b'/');
        index = 1;
    }

    while index < path.len() {
        while index < path.len() && path[index] == b'/' {
            index += 1;
        }
        if index >= path.len() {
            break;
        }

        let end = path[index..]
            .iter()
            .position(|byte| *byte == b'/')
            .map(|offset| index + offset)
            .unwrap_or(path.len());

        if !current.is_empty() && current.last() != Some(&b'/') {
            current.push(b'/');
        }
        current.extend_from_slice(&path[index..end]);

        let current_path = to_c_string(&current)?;
        let result = unsafe { libc::mkdir(current_path.as_ptr(), 0o755 as libc::mode_t) };
        if result != 0 {
            let errno = current_errno();
            if errno != libc::EEXIST {
                return Err(errno);
            }
        }

        index = end.saturating_add(1);
    }

    Ok(())
}

fn remove_tree(path: &[u8]) -> Result<(), c_int> {
    let path_c = to_c_string(path)?;
    let directory = unsafe { libc::opendir(path_c.as_ptr()) };
    if !directory.is_null() {

        let directory_handle = DirectoryHandle(directory);
        loop {
            unsafe {
                *libc::__errno_location() = 0;
            }

            let entry = unsafe { libc::readdir(directory) };
            if entry.is_null() {
                let errno = current_errno();
                if errno == 0 {
                    break;
                }
                return Err(errno);
            }

            let name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) }.to_bytes();
            if name == b"." || name == b".." {
                continue;
            }

            let mut child = Vec::with_capacity(path.len() + 1 + name.len());
            child.extend_from_slice(path);
            if !child.is_empty() && child.last() != Some(&b'/') {
                child.push(b'/');
            }
            child.extend_from_slice(name);
            remove_tree(&child)?;
        }

        drop(directory_handle);

        if unsafe { libc::rmdir(path_c.as_ptr()) } != 0 {
            let errno = current_errno();
            return if errno == libc::ENOENT {
                Ok(())
            } else {
                Err(errno)
            };
        }

        return Ok(());
    }

    let opendir_errno = current_errno();
    if opendir_errno == libc::ENOENT {
        return Ok(());
    }

    if opendir_errno != libc::ENOTDIR {
        return Err(opendir_errno);
    }

    if unsafe { libc::remove(path_c.as_ptr()) } != 0 {
        let errno = current_errno();
        return if errno == libc::ENOENT {
            Ok(())
        } else {
            Err(errno)
        };
    }

    Ok(())
}

#[unsafe(no_mangle)]
extern "C" fn parse_path(path: *const c_char, result: *mut path_t) -> path_parse_result_t {
    assert!(!result.is_null(), "Result pointer cannot be a nullptr");
    let result = unsafe { &mut *result };
    *result = empty_path();

    if path.is_null() {
        return path_parse_result_t::PATH_PARSE_EMPTY_PATH;
    }
    let c_str = unsafe { CStr::from_ptr(path) };
    if c_str.to_bytes().is_empty() {
        return path_parse_result_t::PATH_PARSE_EMPTY_PATH;
    }

    let parse_result = match ParsedPath::new(c_str) {
        Ok(parse_result) => parse_result,
        Err(error) => {
            error.populate_path_t(result);
            return path_parse_result_t::from(&error);
        }
    };

    parse_result.populate_path_t(result);
    path_parse_result_t::PATH_PARSE_OK
}

#[unsafe(no_mangle)]
extern "C" fn path_free(path: *mut path_t) {
    assert!(!path.is_null(), "Result pointer cannot be a nullptr");
    let path = unsafe { &mut *path };

    unsafe {
        if !path.buffer.is_null() {
            libc::free(path.buffer.cast());
        }
        if !path.unixpath.is_null() {
            libc::free(path.unixpath.cast());
        }
    }

    *path = empty_path();
}

#[unsafe(no_mangle)]
extern "C" fn mkdir_p(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }
    let c_str = unsafe { CStr::from_ptr(path) };
    if c_str.to_bytes().is_empty() {
        return false;
    }

    let parsed_path = match ParsedPath::new(c_str) {
        Ok(parsed_path) => parsed_path,
        Err(_) => return false,
    };

    create_dir_all(&parsed_path.to_host_directory_bytes()).is_ok()
}

#[unsafe(no_mangle)]
extern "C" fn rm_rf(path: *const c_char) -> bool {
    if path.is_null() {
        return false;
    }
    let c_str = unsafe { CStr::from_ptr(path) };
    if c_str.to_bytes().is_empty() {
        return false;
    }

    let parsed_path = match ParsedPath::new(c_str) {
        Ok(parsed_path) => parsed_path,
        Err(_) => return true,
    };

    remove_tree(&parsed_path.to_host_file_bytes()).is_ok()
}

#[unsafe(no_mangle)]
extern "C" fn path_dirname(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return ptr::null_mut();
    }
    let c_str = unsafe { CStr::from_ptr(path) };
    if c_str.to_bytes().is_empty() {
        return ptr::null_mut();
    }

    let mut parse_result = match ParsedPath::new(c_str) {
        Ok(parse_result) => parse_result,
        Err(_) => return ptr::null_mut(),
    };
    parse_result.filename = String::new();
    let dirname = format!("{}", parse_result);

    allocate_result_string(dirname.as_bytes())
}

#[unsafe(no_mangle)]
extern "C" fn path_basename(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return ptr::null_mut();
    }
    let c_str = unsafe { CStr::from_ptr(path) };
    if c_str.to_bytes().is_empty() {
        return ptr::null_mut();
    }

    let parse_result = match ParsedPath::new(c_str) {
        Ok(parse_result) => parse_result,
        Err(_) => return ptr::null_mut(),
    };
    if parse_result.filename.is_empty() {
        return ptr::null_mut();
    }

    allocate_result_string(parse_result.filename.as_bytes())
}

#[unsafe(no_mangle)]
extern "C" fn path_devname(path: *const c_char) -> *mut c_char {
    if path.is_null() {
        return ptr::null_mut();
    }
    let c_str = unsafe { CStr::from_ptr(path) };
    if c_str.to_bytes().is_empty() {
        return ptr::null_mut();
    }

    let mut parse_result = match ParsedPath::new(c_str) {
        Ok(parse_result) => parse_result,
        Err(_) => return ptr::null_mut(),
    };
    parse_result.filename = String::new();
    parse_result.directory = None;
    let devname = format!("{}", parse_result);

    allocate_result_string(devname.as_bytes())
}

#[unsafe(no_mangle)]
extern "C" fn path_dirconcat(path: *const c_char, subdir: *const c_char) -> *mut c_char {
    if path.is_null() {
        return ptr::null_mut();
    }
    let path_c_str = unsafe { CStr::from_ptr(path) };
    if path_c_str.to_bytes().is_empty() {
        return ptr::null_mut();
    }

    if subdir.is_null() {
        return ptr::null_mut();
    }
    let subdir_c_str = unsafe { CStr::from_ptr(subdir) };
    let subdir = core::str::from_utf8(subdir_c_str.to_bytes()).expect("Subdir is not valid UTF-8. While you can call this function with any c_str like thing on badgevms, it will cause errors down the line. If you need this function to work with non UTF-8 subdirectories, implement it yourself.");
    if subdir.is_empty() {
        return ptr::null_mut();
    }

    let mut parse_result = match ParsedPath::new(path_c_str) {
        Ok(parse_result) => parse_result,
        Err(_) => return ptr::null_mut(),
    };

    if let Some(directory) = parse_result.directory.filter(|directory| !directory.is_empty()) {
        parse_result.directory = Some(format!("{}.{}", directory, subdir));
    } else {
        parse_result.directory = Some(subdir.to_string());
    }

    let result = format!("{}", parse_result);
    allocate_result_string(result.as_bytes())
}

#[unsafe(no_mangle)]
extern "C" fn path_fileconcat(path: *const c_char, filename: *const c_char) -> *mut c_char {
    if path.is_null() {
        return ptr::null_mut();
    }
    let path_c_str = unsafe { CStr::from_ptr(path) };
    if path_c_str.to_bytes().is_empty() {
        return ptr::null_mut();
    }

    if filename.is_null() {
        return ptr::null_mut();
    }
    let filename_c_str = unsafe { CStr::from_ptr(filename) };
    let filename = core::str::from_utf8(filename_c_str.to_bytes()).expect("Filename is not valid UTF-8. While you can call this function with any c_str like thing on badgevms, it will cause errors down the line. If you need this function to work with non UTF-8 directories, implement it yourself.");
    if filename.is_empty() {
        return ptr::null_mut();
    }

    let mut parse_result = match ParsedPath::new(path_c_str) {
        Ok(parse_result) => parse_result,
        Err(_) => return ptr::null_mut(),
    };
    parse_result.filename = filename.to_string();

    let result = format!("{}", parse_result);
    allocate_result_string(result.as_bytes())
}

#[unsafe(no_mangle)]
extern "C" fn path_concat(base_path: *const c_char, append_path: *const c_char) -> *mut c_char {
    if base_path.is_null() {
        return ptr::null_mut();
    }
    let base_path_c_str = unsafe { CStr::from_ptr(base_path) };
    if base_path_c_str.to_bytes().is_empty() {
        return ptr::null_mut();
    }

    if append_path.is_null() {
        return ptr::null_mut();
    }
    let append_path_c_str = unsafe { CStr::from_ptr(append_path) };
    if append_path_c_str.to_bytes().is_empty() {
        return ptr::null_mut();
    }

    let base_parsed = match ParsedPath::new(base_path_c_str) {
        Ok(parse_result) => parse_result,
        Err(_) => return ptr::null_mut(),
    };
    if !base_parsed.filename.is_empty() {
        return ptr::null_mut();
    }

    let mut append_path_with_device = Vec::from(&b"X:"[..]);
    append_path_with_device.extend_from_slice(append_path_c_str.to_bytes_with_nul());
    let append_path_c_str = CStr::from_bytes_with_nul(&append_path_with_device).unwrap();
    let append_parsed = match ParsedPath::new(append_path_c_str) {
        Ok(parse_result) => parse_result,
        Err(_) => return ptr::null_mut(),
    };

    let maybe_base_dir = base_parsed.directory.filter(|directory| !directory.is_empty());
    let maybe_append_dir = append_parsed.directory.filter(|directory| !directory.is_empty());
    let new_directory = match (maybe_base_dir, maybe_append_dir) {
        (Some(base), Some(append)) => Some(format!("{}.{}", base, append)),
        (Some(directory), None) | (None, Some(directory)) => Some(directory),
        (None, None) => None,
    };

    let result = ParsedPath {
        device: base_parsed.device,
        directory: new_directory,
        filename: append_parsed.filename,
    };

    allocate_result_string(format!("{}", result).as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        ffi::CStr,
        fs,
        os::unix::ffi::OsStringExt,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

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
                "why2025-badge-emu-abi-fs-{test_name}-{}-{unique_suffix}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&path);
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TemporaryTestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn path_buffer() -> path_t {
        empty_path()
    }

    fn host_path(bytes: Vec<u8>) -> PathBuf {
        PathBuf::from(std::ffi::OsString::from_vec(bytes))
    }

    #[test]
    fn parse_path_full_populates_all_components() {
        let mut result = path_buffer();
        let parsed = parse_path(
            c"DEV:[dir.subdir]file.ext".as_ptr(),
            &mut result,
        );

        assert_eq!(parsed, path_parse_result_t::PATH_PARSE_OK);
        assert_eq!(unsafe { CStr::from_ptr(result.device) }.to_str().unwrap(), "DEV");
        assert_eq!(
            unsafe { CStr::from_ptr(result.directory) }.to_str().unwrap(),
            "dir.subdir"
        );
        assert_eq!(
            unsafe { CStr::from_ptr(result.filename) }.to_str().unwrap(),
            "file.ext"
        );

        path_free(&mut result);
    }

    #[test]
    fn parse_path_device_only_keeps_filename_empty() {
        let mut result = path_buffer();
        let parsed = parse_path(c"DEV:".as_ptr(), &mut result);

        assert_eq!(parsed, path_parse_result_t::PATH_PARSE_OK);
        assert_eq!(unsafe { CStr::from_ptr(result.device) }.to_str().unwrap(), "DEV");
        assert!(result.directory.is_null());
        assert!(result.filename.is_null());

        path_free(&mut result);
    }

    #[test]
    fn mkdir_p_creates_nested_host_directories() {
        let temporary_directory = TemporaryTestDirectory::new("mkdir-p-nested");
        let _base_directory = paths::set_base_directory_for_tests(temporary_directory.path());

        assert!(mkdir_p(c"DEV:[dir.subdir]file.ext".as_ptr()));
        assert!(
            temporary_directory
                .path()
                .join("DEV")
                .join("dir")
                .join("subdir")
                .is_dir()
        );
    }

    #[test]
    fn rm_rf_removes_files_and_nested_directories() {
        let temporary_directory = TemporaryTestDirectory::new("rm-rf");
        let _base_directory = paths::set_base_directory_for_tests(temporary_directory.path());
        let host_file = host_path(
            ParsedPath::new(c"DEV:[dir.subdir]file.ext")
                .unwrap()
                .to_host_file_bytes(),
        );

        fs::create_dir_all(host_file.parent().unwrap()).unwrap();
        fs::write(&host_file, b"payload").unwrap();

        assert!(rm_rf(c"DEV:[dir.subdir]file.ext".as_ptr()));
        assert!(!host_file.exists());

        let host_directory = host_path(
            ParsedPath::new(c"DEV:[dir.subdir]")
                .unwrap()
                .to_host_file_bytes(),
        );
        fs::create_dir_all(host_directory.join("child")).unwrap();
        fs::write(host_directory.join("child").join("note.txt"), b"payload").unwrap();

        assert!(rm_rf(c"DEV:[dir.subdir]".as_ptr()));
        assert!(!host_directory.exists());
    }

    #[test]
    fn path_dirname_returns_directory_component() {
        let dirname_ptr = path_dirname(c"DEV:[dir.subdir]file.ext".as_ptr());
        assert!(!dirname_ptr.is_null());

        let dirname = unsafe { CStr::from_ptr(dirname_ptr) }.to_str().unwrap();
        assert_eq!(dirname, "DEV:[dir.subdir]");
        unsafe { libc::free(dirname_ptr.cast()) };
    }

    #[test]
    fn path_basename_returns_null_without_filename() {
        let basename_ptr = path_basename(c"DEV:[dir]".as_ptr());
        assert!(basename_ptr.is_null());
    }

    #[test]
    fn path_devname_returns_device_only_path() {
        let devname_ptr = path_devname(c"DEV:[dir.subdir]file.ext".as_ptr());
        assert!(!devname_ptr.is_null());

        let devname = unsafe { CStr::from_ptr(devname_ptr) }.to_str().unwrap();
        assert_eq!(devname, "DEV:");
        unsafe { libc::free(devname_ptr.cast()) };
    }

    #[test]
    fn path_dirconcat_appends_subdirectory() {
        let path_ptr = path_dirconcat(c"DEV:[dir]file.ext".as_ptr(), c"subdir".as_ptr());
        assert!(!path_ptr.is_null());

        let path = unsafe { CStr::from_ptr(path_ptr) }.to_str().unwrap();
        assert_eq!(path, "DEV:[dir.subdir]file.ext");
        unsafe { libc::free(path_ptr.cast()) };
    }

    #[test]
    fn path_fileconcat_replaces_filename() {
        let path_ptr = path_fileconcat(c"DEV:[dir]old.ext".as_ptr(), c"new.bin".as_ptr());
        assert!(!path_ptr.is_null());

        let path = unsafe { CStr::from_ptr(path_ptr) }.to_str().unwrap();
        assert_eq!(path, "DEV:[dir]new.bin");
        unsafe { libc::free(path_ptr.cast()) };
    }

    #[test]
    fn path_concat_joins_base_directory_with_append_path() {
        let path_ptr = path_concat(c"DEV:[dir.sub]".as_ptr(), c"[more]file.ext".as_ptr());
        assert!(!path_ptr.is_null());

        let path = unsafe { CStr::from_ptr(path_ptr) }.to_str().unwrap();
        assert_eq!(path, "DEV:[dir.sub.more]file.ext");
        unsafe { libc::free(path_ptr.cast()) };
    }
}