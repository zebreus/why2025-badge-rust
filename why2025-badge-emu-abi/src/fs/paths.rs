use crate::{host_forward, types::*};
use alloc::{string::{String, ToString}, vec::Vec};
use core::{
    ffi::{c_char, CStr},
    fmt::{self, Display},
    marker::PhantomData,
    ptr,
};

const BASE_DIRECTORY_SUFFIX: &str = ".why2025-badge/data";
const BASE_DIRECTORY_ENV_VAR: &[u8] = b"WHY2025_BADGE_EMULATED_BASE_DIRECTORY\0";
const HOME_ENV_VAR: &[u8] = b"HOME\0";

#[cfg(test)]
use std::{
    os::unix::ffi::OsStrExt,
    path::Path,
    sync::{LazyLock, Mutex},
};

#[cfg(test)]
static TEST_BASE_DIRECTORY: LazyLock<Mutex<Option<Vec<u8>>>> = LazyLock::new(|| Mutex::new(None));

#[derive(Debug)]
pub(crate) struct ParsedPath {
    pub(crate) device: String,
    pub(crate) directory: Option<String>,
    pub(crate) filename: String,
}

#[derive(Debug)]
pub(crate) enum PathParseError<'a> {
    EmptyDevice { path: &'a CStr },
    NoDevice { path: &'a CStr },
    InvalidDeviceChar { path: &'a CStr },
    UnclosedDirectory { path: &'a CStr },
    InvalidDirChar { path: &'a CStr },
    InvalidFileChar {
        path: &'a CStr,
        directory_index: Option<usize>,
    },
    EmptyPath,
}

impl From<&PathParseError<'_>> for path_parse_result_t {
    fn from(value: &PathParseError<'_>) -> Self {
        match value {
            PathParseError::EmptyPath => path_parse_result_t::PATH_PARSE_EMPTY_PATH,
            PathParseError::InvalidDeviceChar { .. } => {
                path_parse_result_t::PATH_PARSE_INVALID_DEVICE_CHAR
            }
            PathParseError::InvalidDirChar { .. } => {
                path_parse_result_t::PATH_PARSE_INVALID_DIR_CHAR
            }
            PathParseError::InvalidFileChar { .. } => {
                path_parse_result_t::PATH_PARSE_INVALID_FILE_CHAR
            }
            PathParseError::EmptyDevice { .. } => path_parse_result_t::PATH_PARSE_EMPTY_DEVICE,
            PathParseError::UnclosedDirectory { .. } => {
                path_parse_result_t::PATH_PARSE_UNCLOSED_DIRECTORY
            }
            PathParseError::NoDevice { .. } => path_parse_result_t::PATH_PARSE_NO_DEVICE,
        }
    }
}

impl PathParseError<'_> {
    fn result_path_set(&self) -> Option<&CStr> {
        match self {
            Self::EmptyPath => None,
            Self::InvalidDeviceChar { path }
            | Self::InvalidDirChar { path }
            | Self::InvalidFileChar { path, .. }
            | Self::EmptyDevice { path }
            | Self::UnclosedDirectory { path }
            | Self::NoDevice { path } => Some(path),
        }
    }

    fn result_device_set(&self) -> Option<usize> {
        match self {
            Self::InvalidDirChar { .. }
            | Self::UnclosedDirectory { .. }
            | Self::InvalidFileChar { .. } => Some(0),
            Self::EmptyPath
            | Self::InvalidDeviceChar { .. }
            | Self::EmptyDevice { .. }
            | Self::NoDevice { .. } => None,
        }
    }

    fn result_directory_index(&self) -> Option<usize> {
        match self {
            Self::InvalidFileChar {
                directory_index, ..
            } => *directory_index,
            _ => None,
        }
    }

    pub(crate) fn populate_path_t(&self, result: &mut path_t) {
        *result = empty_path();

        let Some(path) = self.result_path_set() else {
            return;
        };

        let Some(buffer) = allocate_path_buffer(path.to_bytes_with_nul()) else {
            return;
        };

        result.len = path.to_bytes().len();
        result.buffer = buffer;

        let Some(device_index) = self.result_device_set() else {
            return;
        };
        result.device = unsafe { result.buffer.add(device_index) };

        let Some(directory_index) = self.result_directory_index() else {
            return;
        };
        result.directory = unsafe { result.buffer.add(directory_index) };
    }
}

impl Display for ParsedPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(directory) = &self.directory {
            write!(f, "{}:[{}]{}", self.device, directory, self.filename)
        } else {
            write!(f, "{}:{}", self.device, self.filename)
        }
    }
}

impl ParsedPath {
    pub(crate) fn new(input: &CStr) -> Result<Self, PathParseError<'_>> {
        enum ParseState<'a> {
            Device(ParseResult<'a>),
            Filename(ParseResult<'a>),
            MaybeDirectory(ParseResult<'a>),
            Directory(ParseResult<'a>),
            Done(ParseResult<'a>),
        }

        if input.to_bytes().is_empty() {
            return Err(PathParseError::EmptyPath);
        }

        let mut state = ParseState::Device(ParseResult {
            device: ptr::null(),
            colon: ptr::null(),
            directory: ptr::null(),
            closing_bracket: ptr::null(),
            filename: ptr::null(),
            terminator: ptr::null(),
            _lifetime: PhantomData,
        });

        for byte in input.to_bytes_with_nul().iter() {
            state = match (state, *byte) {
                (ParseState::Done(_), _) => {
                    unreachable!("encountered another byte after null terminator")
                }
                (ParseState::Filename(result) | ParseState::MaybeDirectory(result), 0) => {
                    ParseState::Done(ParseResult {
                        terminator: byte,
                        ..result
                    })
                }
                (ParseState::Directory(_), 0) => {
                    return Err(PathParseError::UnclosedDirectory { path: input });
                }
                (ParseState::Device(_), 0) => {
                    return Err(PathParseError::NoDevice { path: input });
                }
                (ParseState::Device(result), b':') => {
                    if result.device.is_null() {
                        return Err(PathParseError::EmptyDevice { path: input });
                    }
                    ParseState::MaybeDirectory(ParseResult {
                        colon: byte,
                        ..result
                    })
                }
                (
                    ParseState::Device(result),
                    b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'$',
                ) => {
                    if result.device.is_null() {
                        ParseState::Device(ParseResult {
                            device: byte,
                            ..result
                        })
                    } else {
                        ParseState::Device(result)
                    }
                }
                (ParseState::Device(_), _) => {
                    return Err(PathParseError::InvalidDeviceChar { path: input });
                }
                (ParseState::MaybeDirectory(result), b'[') => ParseState::Directory(result),
                (ParseState::Directory(result), b']') => ParseState::Filename(ParseResult {
                    closing_bracket: byte,
                    ..result
                }),
                (
                    ParseState::Directory(result),
                    b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'$' | b'.',
                ) => {
                    if result.directory.is_null() {
                        ParseState::Directory(ParseResult {
                            directory: byte,
                            ..result
                        })
                    } else {
                        ParseState::Directory(result)
                    }
                }
                (ParseState::Directory(_), _) => {
                    return Err(PathParseError::InvalidDirChar { path: input });
                }
                (
                    ParseState::Filename(result) | ParseState::MaybeDirectory(result),
                    b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'$' | b'.',
                ) => {
                    if result.filename.is_null() {
                        ParseState::Filename(ParseResult {
                            filename: byte,
                            ..result
                        })
                    } else {
                        ParseState::Filename(result)
                    }
                }
                (ParseState::Filename(result) | ParseState::MaybeDirectory(result), _) => {
                    let directory_index = if result.directory.is_null() {
                        None
                    } else {
                        Some(result.directory as usize - result.device as usize)
                    };
                    return Err(PathParseError::InvalidFileChar {
                        path: input,
                        directory_index,
                    });
                }
            };
        }

        let ParseState::Done(result) = state else {
            unreachable!("path parsing did not end in a done state")
        };

        Ok(Self {
            device: result.device().to_string(),
            directory: result.directory().map(ToString::to_string),
            filename: result.filename().to_string(),
        })
    }

    pub(crate) fn populate_path_t(&self, result: &mut path_t) {
        *result = empty_path();

        let string = self.to_string();
        let Some(buffer) = allocate_path_buffer(string.as_bytes()) else {
            return;
        };

        let device_ptr = buffer;
        unsafe { *device_ptr.add(self.device.len()) = 0 };

        let directory_ptr = if let Some(directory) = &self.directory {
            if directory.is_empty() {
                ptr::null_mut()
            } else {
                let directory_ptr = unsafe { buffer.add(self.device.len() + 2) };
                unsafe { *directory_ptr.add(directory.len()) = 0 };
                directory_ptr
            }
        } else {
            ptr::null_mut()
        };

        let filename_ptr = if self.filename.is_empty() {
            ptr::null_mut()
        } else {
            unsafe { buffer.add(string.len() - self.filename.len()) }
        };

        *result = path_t {
            buffer,
            device: device_ptr,
            directory: directory_ptr,
            filename: filename_ptr,
            unixpath: ptr::null_mut(),
            len: string.len(),
        };
    }

    pub(crate) fn to_host_directory_bytes(&self) -> Vec<u8> {
        let mut path = base_directory_bytes();
        append_component(&mut path, self.device.as_bytes());
        if let Some(directory) = self.directory.as_deref() {
            for component in directory.split('.') {
                if component.is_empty() {
                    continue;
                }
                append_component(&mut path, component.as_bytes());
            }
        }
        path
    }

    pub(crate) fn to_host_file_bytes(&self) -> Vec<u8> {
        let mut path = self.to_host_directory_bytes();
        if !self.filename.is_empty() {
            append_component(&mut path, self.filename.as_bytes());
        }
        path
    }
}

#[derive(Debug)]
struct ParseResult<'a> {
    device: *const u8,
    colon: *const u8,
    directory: *const u8,
    closing_bracket: *const u8,
    filename: *const u8,
    terminator: *const u8,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> ParseResult<'a> {
    fn device(&self) -> &'a str {
        debug_assert!(!self.device.is_null());
        debug_assert!(!self.colon.is_null());
        let device_length = self.colon as usize - self.device as usize;
        unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(self.device, device_length))
        }
    }

    fn directory(&self) -> Option<&'a str> {
        if self.directory.is_null() || self.closing_bracket.is_null() {
            return None;
        }

        let directory_length = self.closing_bracket as usize - self.directory as usize;
        unsafe {
            Some(core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                self.directory,
                directory_length,
            )))
        }
    }

    fn filename(&self) -> &'a str {
        if self.filename.is_null() {
            return "";
        }

        debug_assert!(!self.terminator.is_null());
        let filename_length = self.terminator as usize - self.filename as usize;
        unsafe {
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                self.filename,
                filename_length,
            ))
        }
    }
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

    if let Some(base_directory) = host_env_bytes(BASE_DIRECTORY_ENV_VAR) {
        return base_directory;
    }

    if let Some(home_directory) = host_env_bytes(HOME_ENV_VAR) {
        let mut path = home_directory;
        append_component(&mut path, BASE_DIRECTORY_SUFFIX.as_bytes());
        return path;
    }

    BASE_DIRECTORY_SUFFIX.as_bytes().to_vec()
}

fn host_env_bytes(name: &[u8]) -> Option<Vec<u8>> {
    let value = unsafe { host_forward::getenv(name.as_ptr().cast::<c_char>()) };
    if value.is_null() {
        return None;
    }

    let bytes = unsafe { CStr::from_ptr(value) }.to_bytes();
    (!bytes.is_empty()).then(|| bytes.to_vec())
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

fn allocate_path_buffer(bytes: &[u8]) -> Option<*mut c_char> {
    let allocation_size = bytes.len().checked_add(1)?;
    let buffer = unsafe { libc::malloc(allocation_size) }.cast::<c_char>();
    if buffer.is_null() {
        return None;
    }

    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr().cast::<c_char>(), buffer, bytes.len());
        *buffer.add(bytes.len()) = 0;
    }

    Some(buffer)
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