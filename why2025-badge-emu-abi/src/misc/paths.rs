use alloc::ffi::CString;
use alloc::vec::Vec;
use core::ffi::{CStr, c_char};

use crate::runtime as crate_runtime;

const BASE_DIRECTORY_ENV_VAR: &[u8] = b"WHY2025_BADGE_EMULATED_BASE_DIRECTORY";
const DEFAULT_BASE_DIRECTORY_SUFFIX: &[u8] = b".why2025-badge/data";

#[cfg(test)]
use std::{
    os::unix::ffi::OsStrExt,
    path::Path,
    sync::{LazyLock, Mutex},
};

#[cfg(test)]
static TEST_BASE_DIRECTORY: LazyLock<Mutex<Option<Vec<u8>>>> = LazyLock::new(|| Mutex::new(None));

pub(crate) struct ParsedPath {
    device: Vec<u8>,
    directory: Option<Vec<u8>>,
    filename: Vec<u8>,
}

impl ParsedPath {
    pub(crate) fn new(path: &CStr) -> Option<Self> {
        let bytes = path.to_bytes();
        if bytes.is_empty() {
            return None;
        }

        let colon_index = bytes.iter().position(|byte| *byte == b':')?;
        if colon_index == 0 {
            return None;
        }

        let device = &bytes[..colon_index];
        if device.iter().any(|byte| !is_device_char(*byte)) {
            return None;
        }

        let mut cursor = colon_index + 1;
        let directory = if bytes.get(cursor) == Some(&b'[') {
            cursor += 1;
            let closing_index = bytes[cursor..]
                .iter()
                .position(|byte| *byte == b']')?
                + cursor;
            let directory = &bytes[cursor..closing_index];
            if directory.iter().any(|byte| !is_directory_char(*byte)) {
                return None;
            }
            cursor = closing_index + 1;
            Some(directory.to_vec())
        } else {
            None
        };

        let filename = &bytes[cursor..];
        if filename.is_empty() || filename.iter().any(|byte| !is_file_char(*byte)) {
            return None;
        }

        Some(Self {
            device: device.to_vec(),
            directory,
            filename: filename.to_vec(),
        })
    }

    pub(crate) fn to_host_file(&self) -> Option<CString> {
        let mut path = base_directory_bytes();
        append_component(&mut path, &self.device);

        if let Some(directory) = &self.directory {
            for component in directory.split(|byte| *byte == b'.') {
                if component.is_empty() {
                    continue;
                }
                append_component(&mut path, component);
            }
        }

        append_component(&mut path, &self.filename);
        CString::new(path).ok()
    }
}

pub(crate) fn base_directory_env_var_name() -> &'static [u8] {
    BASE_DIRECTORY_ENV_VAR
}

pub(crate) fn base_directory_cstring() -> Option<CString> {
    CString::new(base_directory_bytes()).ok()
}

fn base_directory_bytes() -> Vec<u8> {
    #[cfg(test)]
    if let Some(base_directory) = TEST_BASE_DIRECTORY
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
    {
        return base_directory;
    }

    if let Some(base_directory) = environment_value_bytes(BASE_DIRECTORY_ENV_VAR) {
        return base_directory;
    }

    if let Some(home_directory) = environment_value_bytes(b"HOME") {
        let mut path = home_directory;
        append_component(&mut path, DEFAULT_BASE_DIRECTORY_SUFFIX);
        return path;
    }

    DEFAULT_BASE_DIRECTORY_SUFFIX.to_vec()
}

fn append_component(path: &mut Vec<u8>, component: &[u8]) {
    if component.is_empty() {
        return;
    }

    if !path.is_empty() && path.last() != Some(&b'/') {
        path.push(b'/');
    }
    path.extend_from_slice(component);
}

fn environment_value_bytes(name: &[u8]) -> Option<Vec<u8>> {
    let environ = unsafe {
        crate_runtime::resolve_next_object_value::<*mut *mut c_char>(b"environ\0")
    };
    if environ.is_null() {
        return None;
    }

    let mut index = 0usize;
    loop {
        let entry_ptr = unsafe { *environ.add(index) };
        if entry_ptr.is_null() {
            return None;
        }

        let entry = unsafe { CStr::from_ptr(entry_ptr) }.to_bytes();
        if let Some(value) = match_environment_entry(entry, name) {
            if value.is_empty() {
                return None;
            }
            return Some(value.to_vec());
        }

        index += 1;
    }
}

fn match_environment_entry<'a>(entry: &'a [u8], name: &[u8]) -> Option<&'a [u8]> {
    if entry.len() <= name.len() || &entry[..name.len()] != name || entry[name.len()] != b'=' {
        return None;
    }

    Some(&entry[name.len() + 1..])
}

fn is_device_char(byte: u8) -> bool {
    !matches!(byte, 0..=31 | b':' | b'[' | b']' | b'/')
}

fn is_directory_char(byte: u8) -> bool {
    !matches!(byte, 0..=31 | b':' | b'[' | b']' | b'/')
}

fn is_file_char(byte: u8) -> bool {
    !matches!(byte, 0..=31 | b':' | b'[' | b']' | b'/')
}

#[cfg(test)]
pub(crate) fn set_base_directory_for_tests(base_directory: &Path) -> TestBaseDirectoryGuard {
    let mut value = TEST_BASE_DIRECTORY
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = value.replace(base_directory.as_os_str().as_bytes().to_vec());
    TestBaseDirectoryGuard { previous }
}

#[cfg(test)]
pub(crate) struct TestBaseDirectoryGuard {
    previous: Option<Vec<u8>>,
}

#[cfg(test)]
impl Drop for TestBaseDirectoryGuard {
    fn drop(&mut self) {
        let mut value = TEST_BASE_DIRECTORY
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *value = self.previous.take();
    }
}