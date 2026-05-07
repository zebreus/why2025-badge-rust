use crate::{
    emulated::badgevms::{
        fs::{mkdir_p, path_concat, path_dirconcat, path_dirname, path_fileconcat, rm_rf},
        fs::paths::ParsedPath,
        misc::process_create,
    },
    free, malloc,
    types::*,
};
use core::ffi::{CStr, c_char, c_void};
use serde::Serialize;
use serde_json::Value;
use std::{
    ffi::CString,
    fs,
    mem,
    os::unix::ffi::OsStrExt,
    path::PathBuf,
    ptr,
};

const APPLICATIONS_BASE_DIR: &[u8] = b"APPS:\0";

#[derive(Serialize)]
struct PersistedApplication {
    unique_identifier: String,
    name: String,
    author: String,
    version: String,
    interpreter: String,
    metadata_file: String,
    binary_path: String,
    source: u32,
}

#[derive(Default)]
struct LoadedApplication {
    unique_identifier: Option<String>,
    name: Option<String>,
    author: Option<String>,
    version: Option<String>,
    interpreter: Option<String>,
    metadata_file: Option<String>,
    binary_path: Option<String>,
    source: Option<u32>,
}

struct ApplicationListState {
    applications: Vec<*mut application_t>,
    current_index: usize,
}

struct OwnedMallocCString {
    ptr: *mut c_char,
}

impl OwnedMallocCString {
    unsafe fn from_raw(ptr: *mut c_char) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self { ptr })
        }
    }

    fn as_c_str(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.ptr) }
    }

    fn as_ptr(&self) -> *const c_char {
        self.ptr
    }

    fn into_raw(mut self) -> *mut c_char {
        let ptr = self.ptr;
        self.ptr = ptr::null_mut();
        mem::forget(self);
        ptr
    }
}

impl Drop for OwnedMallocCString {
    fn drop(&mut self) {
        if self.ptr.is_null() {
            return;
        }

        unsafe {
            free(self.ptr.cast::<c_void>());
        }
    }
}

fn applications_base_dir() -> &'static CStr {
    unsafe { CStr::from_bytes_with_nul_unchecked(APPLICATIONS_BASE_DIR) }
}

fn applications_root_host_directory() -> Option<PathBuf> {
    Some(ParsedPath::new(applications_base_dir()).ok()?.to_host_directory())
}

fn ensure_applications_root() -> bool {
    mkdir_p(applications_base_dir().as_ptr())
}

fn allocate_c_string(value: &str) -> *mut c_char {
    let Some(allocation_size) = value
        .len()
        .checked_add(1)
        .and_then(|size| u32::try_from(size).ok())
    else {
        return ptr::null_mut();
    };

    let buffer = unsafe { malloc(allocation_size) as *mut c_char };
    if buffer.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        buffer.copy_from_nonoverlapping(value.as_ptr().cast::<c_char>(), value.len());
        *buffer.add(value.len()) = 0;
    }

    buffer
}

unsafe fn free_guest_string(value: *const c_char) {
    if value.is_null() {
        return;
    }

    unsafe {
        free(value.cast_mut().cast::<c_void>());
    }
}

fn duplicate_input_string(value: *const c_char) -> *mut c_char {
    if value.is_null() {
        return ptr::null_mut();
    }

    let value = unsafe { CStr::from_ptr(value) }.to_string_lossy();
    allocate_c_string(value.as_ref())
}

fn field_to_string(value: *const c_char) -> String {
    if value.is_null() {
        return String::new();
    }

    unsafe { CStr::from_ptr(value) }
        .to_string_lossy()
        .into_owned()
}

fn source_from_raw(raw: u32) -> application_source_t {
    match raw {
        x if x == application_source_t::APPLICATION_SOURCE_BADGEHUB as u32 => {
            application_source_t::APPLICATION_SOURCE_BADGEHUB
        }
        x if x == application_source_t::APPLICATION_SOURCE_MAX as u32 => {
            application_source_t::APPLICATION_SOURCE_MAX
        }
        _ => application_source_t::APPLICATION_SOURCE_UNKNOWN,
    }
}

fn metadata_badge_path(unique_identifier: &CStr) -> Option<OwnedMallocCString> {
    let mut filename = unique_identifier.to_bytes().to_vec();
    filename.extend_from_slice(b".json");
    let filename = CString::new(filename).ok()?;

    unsafe { OwnedMallocCString::from_raw(path_fileconcat(applications_base_dir().as_ptr(), filename.as_ptr())) }
}

fn application_dir_badge_path(unique_identifier: &CStr) -> Option<OwnedMallocCString> {
    unsafe { OwnedMallocCString::from_raw(path_dirconcat(applications_base_dir().as_ptr(), unique_identifier.as_ptr())) }
}

fn badge_path_to_host_file(path: &CStr) -> Option<PathBuf> {
    Some(ParsedPath::new(path).ok()?.to_host_file())
}

fn parse_loaded_application(json: Value) -> LoadedApplication {
    let Some(object) = json.as_object() else {
        return LoadedApplication::default();
    };

    LoadedApplication {
        unique_identifier: object
            .get("unique_identifier")
            .and_then(Value::as_str)
            .map(str::to_owned),
        name: object.get("name").and_then(Value::as_str).map(str::to_owned),
        author: object.get("author").and_then(Value::as_str).map(str::to_owned),
        version: object.get("version").and_then(Value::as_str).map(str::to_owned),
        interpreter: object
            .get("interpreter")
            .and_then(Value::as_str)
            .map(str::to_owned),
        metadata_file: object
            .get("metadata_file")
            .and_then(Value::as_str)
            .map(str::to_owned),
        binary_path: object
            .get("binary_path")
            .and_then(Value::as_str)
            .map(str::to_owned),
        source: object
            .get("source")
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok()),
    }
}

fn load_metadata(unique_identifier: &CStr) -> Option<LoadedApplication> {
    let metadata_badge_path = metadata_badge_path(unique_identifier)?;
    let host_path = badge_path_to_host_file(metadata_badge_path.as_c_str())?;
    let content = fs::read(host_path).ok()?;
    let json = serde_json::from_slice(&content).ok()?;
    Some(parse_loaded_application(json))
}

fn save_metadata(application: *const application_t) -> bool {
    if application.is_null() {
        return false;
    }

    let application = unsafe { &*application };
    if application.unique_identifier.is_null() {
        return false;
    }

    let unique_identifier = unsafe { CStr::from_ptr(application.unique_identifier) };
    let persisted = PersistedApplication {
        unique_identifier: unique_identifier.to_string_lossy().into_owned(),
        name: field_to_string(application.name),
        author: field_to_string(application.author),
        version: field_to_string(application.version),
        interpreter: field_to_string(application.interpreter),
        metadata_file: field_to_string(application.metadata_file),
        binary_path: field_to_string(application.binary_path),
        source: application.source as u32,
    };

    let Some(metadata_badge_path) = metadata_badge_path(unique_identifier) else {
        return false;
    };
    let Some(host_path) = badge_path_to_host_file(metadata_badge_path.as_c_str()) else {
        return false;
    };
    let Ok(serialized) = serde_json::to_vec_pretty(&persisted) else {
        return false;
    };

    if !ensure_applications_root() {
        return false;
    }

    if let Some(parent) = host_path.parent()
        && fs::create_dir_all(parent).is_err()
    {
        return false;
    }

    fs::write(host_path, serialized).is_ok()
}

fn allocate_snapshot(loaded: LoadedApplication) -> *mut application_t {
    let installed_path = loaded
        .unique_identifier
        .as_ref()
        .and_then(|unique_identifier| CString::new(unique_identifier.as_bytes()).ok())
        .and_then(|unique_identifier| application_dir_badge_path(unique_identifier.as_c_str()))
        .map(OwnedMallocCString::into_raw)
        .unwrap_or(ptr::null_mut());

    Box::into_raw(Box::new(application_t {
        unique_identifier: loaded
            .unique_identifier
            .as_deref()
            .map(allocate_c_string)
            .unwrap_or(ptr::null_mut()),
        name: loaded
            .name
            .as_deref()
            .map(allocate_c_string)
            .unwrap_or(ptr::null_mut()),
        author: loaded
            .author
            .as_deref()
            .map(allocate_c_string)
            .unwrap_or(ptr::null_mut()),
        version: loaded
            .version
            .as_deref()
            .map(allocate_c_string)
            .unwrap_or(ptr::null_mut()),
        interpreter: loaded
            .interpreter
            .as_deref()
            .map(allocate_c_string)
            .unwrap_or(ptr::null_mut()),
        metadata_file: loaded
            .metadata_file
            .as_deref()
            .map(allocate_c_string)
            .unwrap_or(ptr::null_mut()),
        installed_path,
        binary_path: loaded
            .binary_path
            .as_deref()
            .map(allocate_c_string)
            .unwrap_or(ptr::null_mut()),
        source: source_from_raw(loaded.source.unwrap_or_default()),
    }))
}

unsafe fn replace_string_field(field: &mut *const c_char, value: *const c_char) {
    unsafe {
        free_guest_string(*field);
    }
    *field = duplicate_input_string(value);
}

pub(crate) fn application_launch(unique_identifier: *const c_char) -> pid_t {
    if unique_identifier.is_null() {
        return -1;
    }

    let application = application_get(unique_identifier);
    if application.is_null() {
        return -1;
    }

    let application_ref = unsafe { &*application };
    if application_ref.binary_path.is_null() || application_ref.installed_path.is_null() {
        application_free(application);
        return -1;
    }

    let Some(binary_path) = (unsafe {
        OwnedMallocCString::from_raw(path_concat(
            application_ref.installed_path,
            application_ref.binary_path,
        ))
    }) else {
        application_free(application);
        return -1;
    };

    let pid = process_create(binary_path.as_ptr(), 0, 0, ptr::null_mut());
    application_free(application);
    pid
}

pub(crate) fn application_create(
    unique_identifier: *const c_char,
    name: *const c_char,
    author: *const c_char,
    version: *const c_char,
    interpreter: *const c_char,
    source: application_source_t,
) -> *mut application_t {
    if unique_identifier.is_null() {
        return ptr::null_mut();
    }

    let unique_identifier = unsafe { CStr::from_ptr(unique_identifier) };
    let Some(metadata_badge_path) = metadata_badge_path(unique_identifier) else {
        return ptr::null_mut();
    };
    let Some(metadata_host_path) = badge_path_to_host_file(metadata_badge_path.as_c_str()) else {
        return ptr::null_mut();
    };

    if fs::File::open(&metadata_host_path).is_ok() {
        return ptr::null_mut();
    }

    let Some(application_dir) = application_dir_badge_path(unique_identifier) else {
        return ptr::null_mut();
    };
    if !mkdir_p(application_dir.as_ptr()) {
        return ptr::null_mut();
    }

    let application = Box::into_raw(Box::new(application_t {
        unique_identifier: duplicate_input_string(unique_identifier.as_ptr()),
        name: duplicate_input_string(name),
        author: duplicate_input_string(author),
        version: duplicate_input_string(version),
        interpreter: duplicate_input_string(interpreter),
        metadata_file: ptr::null_mut(),
        installed_path: application_dir.into_raw(),
        binary_path: ptr::null_mut(),
        source,
    }));

    if !save_metadata(application) {
        application_free(application);
        return ptr::null_mut();
    }

    application
}

pub(crate) fn application_set_metadata(
    application: *mut application_t,
    metadata_file: *const c_char,
) -> bool {
    if application.is_null() {
        return false;
    }

    let installed_path = unsafe { (*application).installed_path };
    if !metadata_file.is_null() && !validate_relative_path(installed_path, metadata_file) {
        return false;
    }

    unsafe {
        replace_string_field(&mut (*application).metadata_file, metadata_file);
    }
    save_metadata(application)
}

pub(crate) fn application_set_binary_path(
    application: *mut application_t,
    binary_path: *const c_char,
) -> bool {
    if application.is_null() {
        return false;
    }

    let installed_path = unsafe { (*application).installed_path };
    if !binary_path.is_null() && !validate_relative_path(installed_path, binary_path) {
        return false;
    }

    unsafe {
        replace_string_field(&mut (*application).binary_path, binary_path);
    }
    save_metadata(application)
}

pub(crate) fn application_set_version(
    application: *mut application_t,
    version: *const c_char,
) -> bool {
    if application.is_null() {
        return false;
    }

    unsafe {
        replace_string_field(&mut (*application).version, version);
    }
    save_metadata(application)
}

pub(crate) fn application_set_author(application: *mut application_t, author: *const c_char) -> bool {
    if application.is_null() {
        return false;
    }

    unsafe {
        replace_string_field(&mut (*application).author, author);
    }
    save_metadata(application)
}

pub(crate) fn application_set_name(application: *mut application_t, name: *const c_char) -> bool {
    if application.is_null() {
        return false;
    }

    unsafe {
        replace_string_field(&mut (*application).name, name);
    }
    save_metadata(application)
}

pub(crate) fn application_set_interpreter(
    application: *mut application_t,
    interpreter: *const c_char,
) -> bool {
    if application.is_null() {
        return false;
    }

    unsafe {
        replace_string_field(&mut (*application).interpreter, interpreter);
    }
    save_metadata(application)
}

pub(crate) fn application_destroy(application: *mut application_t) -> bool {
    if application.is_null() {
        return false;
    }

    let unique_identifier = unsafe { (*application).unique_identifier };
    if unique_identifier.is_null() {
        return false;
    }

    let unique_identifier = unsafe { CStr::from_ptr(unique_identifier) };
    let Some(application_dir) = application_dir_badge_path(unique_identifier) else {
        return false;
    };
    rm_rf(application_dir.as_ptr())
}

pub(crate) fn application_create_file(
    application: *mut application_t,
    file_path: *const c_char,
) -> *mut FILE {
    let Some(absolute_badge_path) = (unsafe {
        OwnedMallocCString::from_raw(application_create_file_string(application, file_path))
    }) else {
        return ptr::null_mut();
    };
    let Some(host_path) = badge_path_to_host_file(absolute_badge_path.as_c_str()) else {
        return ptr::null_mut();
    };
    let Ok(host_path) = CString::new(host_path.as_os_str().as_bytes()) else {
        return ptr::null_mut();
    };

    unsafe { libc::fopen(host_path.as_ptr(), c"w".as_ptr()).cast::<FILE>() }
}

pub(crate) fn application_create_file_string(
    application: *mut application_t,
    file_path: *const c_char,
) -> *mut c_char {
    if application.is_null() {
        return ptr::null_mut();
    }

    let application = unsafe { &*application };
    if file_path.is_null() || application.installed_path.is_null() {
        return ptr::null_mut();
    }

    let Some(absolute_badge_path) = (unsafe {
        OwnedMallocCString::from_raw(path_concat(application.installed_path, file_path))
    }) else {
        return ptr::null_mut();
    };
    let Some(dirname) = (unsafe { OwnedMallocCString::from_raw(path_dirname(absolute_badge_path.as_ptr())) }) else {
        return ptr::null_mut();
    };
    if !mkdir_p(dirname.as_ptr()) {
        return ptr::null_mut();
    }

    absolute_badge_path.into_raw()
}

pub(crate) fn application_list(out: *mut *mut application_t) -> application_list_handle {
    if !ensure_applications_root() {
        return ptr::null_mut();
    }

    let Some(root_directory) = applications_root_host_directory() else {
        return ptr::null_mut();
    };
    let Ok(entries) = fs::read_dir(root_directory) else {
        return ptr::null_mut();
    };

    let mut applications = Vec::new();
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        let Some(unique_identifier) = file_name.strip_suffix(".json") else {
            continue;
        };
        let Ok(unique_identifier) = CString::new(unique_identifier) else {
            continue;
        };

        let application = application_get(unique_identifier.as_ptr());
        if !application.is_null() {
            applications.push(application);
        }
    }

    if !out.is_null() {
        unsafe {
            *out = applications.first().copied().unwrap_or(ptr::null_mut());
        }
    }

    Box::into_raw(Box::new(ApplicationListState {
        applications,
        current_index: 0,
    }))
    .cast::<application_list>()
}

pub(crate) fn application_list_get_next(list: application_list_handle) -> *mut application_t {
    if list.is_null() {
        return ptr::null_mut();
    }

    let list = unsafe { &mut *list.cast::<ApplicationListState>() };
    list.current_index = list.current_index.saturating_add(1);
    list.applications
        .get(list.current_index)
        .copied()
        .unwrap_or(ptr::null_mut())
}

pub(crate) fn application_list_close(list: application_list_handle) {
    if list.is_null() {
        return;
    }

    let list = unsafe { Box::from_raw(list.cast::<ApplicationListState>()) };
    for application in list.applications {
        application_free(application);
    }
}

pub(crate) fn application_get(unique_identifier: *const c_char) -> *mut application_t {
    if unique_identifier.is_null() {
        return ptr::null_mut();
    }

    let unique_identifier = unsafe { CStr::from_ptr(unique_identifier) };
    let Some(loaded) = load_metadata(unique_identifier) else {
        return ptr::null_mut();
    };
    allocate_snapshot(loaded)
}

pub(crate) fn application_free(application: *mut application_t) {
    if application.is_null() {
        return;
    }

    unsafe {
        free_guest_string((*application).unique_identifier);
        free_guest_string((*application).name);
        free_guest_string((*application).author);
        free_guest_string((*application).version);
        free_guest_string((*application).interpreter);
        free_guest_string((*application).metadata_file);
        free_guest_string((*application).installed_path);
        free_guest_string((*application).binary_path);
        drop(Box::from_raw(application));
    }
}

fn validate_relative_path(installed_path: *const c_char, candidate: *const c_char) -> bool {
    if installed_path.is_null() || candidate.is_null() {
        return false;
    }

    unsafe { OwnedMallocCString::from_raw(path_concat(installed_path, candidate)).is_some() }
}