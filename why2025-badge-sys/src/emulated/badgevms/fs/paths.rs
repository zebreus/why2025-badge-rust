use crate::{malloc, types::*};
use core::ffi::{CStr, c_char};
use std::{fmt::Display, path::PathBuf};

const BASE_DIRECTORY: &str = "~/.why2025-badge/data";

#[derive(Debug)]
pub struct ParsedPath {
    pub device: String,
    pub directory: Option<String>,
    pub filename: String,
}

pub enum PathParseError<'a> {
    EmptyDevice {
        path: &'a CStr,
    },
    NoDevice {
        path: &'a CStr,
    },
    InvalidDeviceChar {
        path: &'a CStr,
    },
    UnclosedDirectory {
        path: &'a CStr,
    },
    InvalidDirChar {
        path: &'a CStr,
    },
    InvalidFileChar {
        path: &'a CStr,
        directory_index: Option<usize>,
    },
    EmptyPath {},
}
impl Into<path_parse_result_t> for &PathParseError<'_> {
    fn into(self) -> path_parse_result_t {
        match *self {
            PathParseError::EmptyPath {} => path_parse_result_t::PATH_PARSE_EMPTY_PATH,
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
    fn result_path_set(&self) -> Option<&'_ CStr> {
        match self {
            Self::EmptyPath {} => None,
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
            Self::EmptyPath {}
            | Self::InvalidDeviceChar { .. }
            | Self::EmptyDevice { .. }
            | Self::NoDevice { .. } => None,
        }
    }
    /// Checks if the directory part was set in the result.
    ///
    /// Returns the index in the path string where the directory part starts.
    fn result_directory_index(&self) -> Option<usize> {
        match self {
            Self::InvalidFileChar {
                directory_index, ..
            } => *directory_index,
            _ => None,
        }
    }
    /// Populates a path_t structure with as much information as possible from this error.
    ///
    /// The fields that will be populated depend on how far the parsing got before the error was encountered.
    /// This behaviour tries to match the badgevms implementation as closely as possible.
    pub fn populate_path_t(&self, result: &mut path_t) {
        let Some(path) = self.result_path_set() else {
            return;
        };

        let bytes = path.to_bytes_with_nul();
        let boxed: Box<[u8]> = bytes.to_owned().into_boxed_slice();
        result.len = boxed.len() - 1;
        result.buffer = Box::leak(boxed) as *mut [u8] as *mut c_char;
        result.device = std::ptr::null_mut();
        result.directory = std::ptr::null_mut();
        result.filename = std::ptr::null_mut();
        result.unixpath = std::ptr::null_mut();

        // Set device if we got far enough
        let Some(device_index) = self.result_device_set() else {
            return;
        };
        result.device = unsafe { result.buffer.byte_add(device_index) };

        // Set device if we got far enough
        let Some(directory_index) = self.result_directory_index() else {
            return;
        };
        result.directory = unsafe { result.buffer.byte_add(directory_index) };
    }
}

impl Display for ParsedPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(dir) = &self.directory {
            write!(f, "{}:[{}]{}", self.device, dir, self.filename)
        } else {
            write!(f, "{}:{}", self.device, self.filename)
        }
    }
}

impl ParsedPath {
    pub fn to_string(&self) -> String {
        format!("{}", self)
    }
    pub fn populate_path_t(&self, result: &mut path_t) {
        let string = self.to_string();

        let buffer = unsafe {
            let buffer = malloc(string.len() as u32 + 1) as *mut c_char;
            buffer.copy_from_nonoverlapping(string.as_ptr() as *const c_char, string.len());
            *buffer.offset(string.len() as isize) = 0;
            buffer
        };

        // Prepare device pointer
        let device_ptr = buffer;
        unsafe { *device_ptr.offset(self.device.len() as isize) = 0 };

        // Prepare directory pointer
        let directory_ptr = if let Some(directory) = &self.directory
            && directory.len() > 0
        {
            let directory_ptr = unsafe { buffer.byte_add(self.device.len() + 2) };
            unsafe { *directory_ptr.byte_add(directory.len()) = 0 };
            directory_ptr
        } else {
            std::ptr::null_mut()
        };

        // Prepare filename pointer
        let filename_ptr = if self.filename.len() > 0 {
            unsafe { buffer.byte_add(string.len() - self.filename.len()) }
        } else {
            std::ptr::null_mut()
        };

        *result = path_t {
            buffer,
            device: device_ptr,
            directory: directory_ptr,
            filename: filename_ptr,
            unixpath: std::ptr::null_mut(),
            len: string.len(),
        };
    }

    pub fn to_host_directory(&self) -> PathBuf {
        let mut path = PathBuf::from(BASE_DIRECTORY);
        path.push(&self.device);
        for part in self.directory.as_ref().unwrap_or(&String::new()).split(".") {
            if part.is_empty() {
                continue;
            }
            path.push(part);
        }
        path
    }

    pub fn to_host_file(&self) -> PathBuf {
        let mut path = self.to_host_directory();
        if !self.filename.is_empty() {
            path.push(&self.filename);
        }
        path
    }
}

/// A valid parse result.
///
/// If you got this structure from the parser the path is guaranteed to be fully valid.
#[derive(Debug)]
struct ParseResult<'a> {
    device: *const u8,
    /// This points to the byte after the device name.
    /// Needs to be set to 0 to make device a valid C string.
    colon: *const u8,
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
    fn device(&self) -> &'a str {
        debug_assert!(!self.device.is_null());
        debug_assert!(!self.colon.is_null());
        let device_length = self.colon as usize - self.device as usize;
        unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.device, device_length))
        }
    }
    fn directory(&self) -> Option<&'a str> {
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
    fn filename(&self) -> &'a str {
        if self.filename.is_null() {
            return "";
        }
        debug_assert!(!self.terminator.is_null());
        let filename_length = self.terminator as usize - self.filename as usize;
        unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                self.filename,
                filename_length,
            ))
        }
    }
}

impl ParsedPath {
    pub fn new(input: &core::ffi::CStr) -> Result<ParsedPath, PathParseError<'_>> {
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
                    return Err(PathParseError::UnclosedDirectory { path: input });
                }
                (ParseState::Device(_), 0) => {
                    return Err(PathParseError::NoDevice { path: input });
                }
                (ParseState::Device(r), b':') => {
                    if r.device.is_null() {
                        return Err(PathParseError::EmptyDevice { path: input });
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
                    return Err(PathParseError::InvalidDeviceChar { path: input });
                }
                (ParseState::MaybeDirectory(r), b'[') => ParseState::Directory(r),
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
                    return Err(PathParseError::InvalidDirChar { path: input });
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
                (ParseState::Filename(r) | ParseState::MaybeDirectory(r), _) => {
                    let directory_index = if r.directory.is_null() {
                        None
                    } else {
                        Some(r.directory as usize - r.device as usize)
                    };
                    return Err(PathParseError::InvalidFileChar {
                        path: input,
                        directory_index,
                    });
                }
            }
        }

        let ParseState::Done(result) = state else {
            unreachable!("Path parsing did not end in a done state. This is not possible.")
        };
        let result = ParsedPath {
            device: result.device().to_string(),
            directory: result.directory().map(|d| d.to_string()),
            filename: result.filename().to_string(),
        };
        Ok(result)
    }
}
