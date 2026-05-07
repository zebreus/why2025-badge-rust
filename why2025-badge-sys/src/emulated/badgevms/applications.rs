//! Emulated stubs for BadgeVMS application-management functions.
//!
//! The rustdoc on the items in this file is derived from the upstream firmware
//! implementation in `WHY2025/team-badge/firmware` at commit
//! `a548d825a3295432d374939607feb552eb505210`
//! (`Update espressif/eppp_link`). The goal is to document the exact
//! implementation behavior of the current firmware, including details that are
//! only visible in the `.c` files and not in the public headers.
//!
//! # Upstream Storage Model
//!
//! The public application API is implemented in `firmware/badgevms/application.c`.
//! It is not backed by a database. Instead, it maintains a hidden process-wide
//! base directory string, `applications_base_dir`, which is set by the private
//! `application_init()` helper from `why2025_firmware.c`.
//!
//! In the normal firmware boot path `why2025_firmware.c` first defines the
//! logical name `APPS:` and then calls `application_init("APPS:", ...)`.
//! When `SD0` is available, `APPS:` resolves to the search path
//! `SD0:[BADGEVMS.APPS], FLASH0:[BADGEVMS.APPS]`; otherwise it resolves to
//! `FLASH0:[BADGEVMS.APPS]` only. `application_init()` copies the literal
//! string `APPS:` into the hidden buffer, eagerly calls `mkdir_p()` on the
//! physical flash/SD directories it was passed, and finally calls
//! `mkdir_p(applications_base_dir)` as well. The storage roots therefore exist
//! before any public application API is used. Two kinds of paths are then
//! derived from an application's unique identifier:
//!
//! - metadata lives in a flat JSON file at `APPS:<unique_identifier>.json`
//! - payload files live under a directory at `APPS:[<unique_identifier>]`
//!
//! Each public `application_t *` is just a heap-allocated snapshot of that JSON
//! metadata plus a reconstructed `installed_path`. There is no central in-memory
//! registry of live `application_t` objects, and there is no locking anywhere in
//! `application.c`. Concurrent callers can therefore race on both the JSON files
//! and the hidden global base-directory string.
//!
//! `application_to_json()` serializes missing strings as empty JSON strings via
//! `field ?: ""`. `json_to_application()` then reconstructs a fresh heap object
//! with `why_strdup()` for each string field it finds. `installed_path` is not
//! loaded from JSON at all; it is recomputed from `unique_identifier` using the
//! current `applications_base_dir`.
//!
//! Because `application_to_json()` always emits all string keys, a field that
//! was null in memory and then saved is normally reloaded later as a non-null
//! empty string. That null-versus-empty distinction is therefore not stable
//! across save/load boundaries.
//!
//! `save_application_metadata()` also treats opening the metadata file as the
//! last meaningful I/O failure point. Once `why_fopen(..., "w")` succeeds, the
//! implementation ignores the return values from `why_fputs()` and
//! `why_fclose()`, so late write or flush failures are not surfaced.
//!
//! A consequence of that model is that individual string-allocation failures are
//! often silent: several setters free the old string, assign the result of
//! `why_strdup()` without checking it, and then save metadata. If `why_strdup()`
//! returns null and the JSON rewrite still succeeds, the field is effectively
//! persisted as an empty string.

use crate::types::*;

pub(crate) mod runtime;

use runtime::{
    application_create as application_create_inner,
    application_create_file as application_create_file_inner,
    application_create_file_string as application_create_file_string_inner,
    application_destroy as application_destroy_inner,
    application_free as application_free_inner,
    application_get as application_get_inner,
    application_launch as application_launch_inner,
    application_list as application_list_inner,
    application_list_close as application_list_close_inner,
    application_list_get_next as application_list_get_next_inner,
    application_set_author as application_set_author_inner,
    application_set_binary_path as application_set_binary_path_inner,
    application_set_interpreter as application_set_interpreter_inner,
    application_set_metadata as application_set_metadata_inner,
    application_set_name as application_set_name_inner,
    application_set_version as application_set_version_inner,
};

/// Launch an installed application by unique identifier.
///
/// # Exact upstream behavior
///
/// Upstream first rejects a null `unique_identifier` with `-1`.
///
/// It then calls `application_get(unique_identifier)`, which loads the JSON
/// snapshot from `APPS:<unique_identifier>.json` and reconstructs an
/// `application_t` on the heap. The loaded snapshot is not freed anywhere in the
/// success or failure paths of `application_launch`, so each non-null launch
/// attempt leaks one `application_t` plus its duplicated strings.
///
/// Launch then requires all of the following to be present:
///
/// - the loaded `application_t *`
/// - `app->binary_path`
/// - `app->installed_path`
///
/// If any of those are missing it logs
/// `"Cannot launch %s, no binary path or installed_path?"` and returns `-1`
/// without freeing the loaded snapshot.
///
/// Because saved metadata usually reloads cleared string fields as non-null
/// empty strings, an application with a persisted empty `binary_path` does not
/// trip the `!app->binary_path` guard. It reaches `path_concat()` instead,
/// which rejects an empty append path and causes the later
/// `"Could not create a sane path..."` failure path.
///
/// Otherwise it computes an absolute path with
/// `path_concat(app->installed_path, app->binary_path)`. That means the stored
/// `binary_path` is treated as a path relative to the install directory even
/// though the public header describes it as physical. If concatenation fails it
/// logs a warning and returns `-1`, again leaking the loaded `application_t`.
///
/// On success it logs `"Attempting to launch %s"`, calls
/// `process_create(binary_path, 0, 0, NULL)`, frees only the temporary absolute
/// path string, and returns the PID or `-1` from `process_create` unchanged.
/// With the current `process_create` implementation that also means the child
/// sees `argv[0]` synthesized from the resolved binary path.
///
/// # Important omissions and interactions
///
/// The launch path does not consult `app->interpreter` at all. Script-like
/// applications whose metadata records an interpreter are still launched exactly
/// the same way as native binaries: by passing the resolved `binary_path` to
/// `process_create` with `stack_size = 0`, `argc = 0`, and `argv = NULL`.
///
/// Unlike the startup loader in `init.c`, `application_launch` also does not set
/// the spawned task's `application_uid` with `task_set_application_uid()`. A
/// task started through this API therefore does not automatically participate in
/// `task_application_is_running()` tracking, even though the startup loader does.
///
/// The launcher UI and the compositor both call this function directly. The
/// launcher uses it for user-selected apps, and the compositor uses it to
/// relaunch `badgevms_launcher` when no windows exist. Neither caller adds extra
/// `application_uid` bookkeeping around the launch.
///
/// Because launch works from the JSON metadata file, stale metadata can outlive a
/// destroyed install directory and still make an application appear launchable
/// until `process_create` fails on the missing binary path.
#[unsafe(no_mangle)]
pub extern "C" fn application_launch(unique_identifier: *const ::core::ffi::c_char) -> pid_t {
    application_launch_inner(unique_identifier)
}
/// Create a new application metadata record and install directory.
///
/// # Exact upstream behavior
///
/// Upstream rejects the request immediately if either:
///
/// - `unique_identifier == NULL`, or
/// - the hidden global `applications_base_dir` is still empty because the
///   private `application_init()` boot-time setup did not run successfully.
///
/// It then validates the identifier indirectly by trying to build the metadata
/// path `APPS:<unique_identifier>.json` with `get_metadata_path()`. That helper
/// appends `".json"` and then runs the result through `parse_path()`. Invalid
/// device/file characters therefore cause creation to fail with a warning
/// `"Illegal application name %s"`.
///
/// Duplicate detection is just a readability test on the metadata file: if
/// `why_fopen(metadata_path, "r")` succeeds, upstream closes the file and
/// returns null. If opening fails for some other reason than non-existence,
/// upstream does not distinguish that case and continues as if the application
/// were absent.
///
/// Duplicate detection is therefore metadata-only. If `APPS:[<unique_identifier>]`
/// already exists on disk but `APPS:<unique_identifier>.json` does not,
/// `application_create()` treats the application as absent and reuses the
/// existing directory tree.
///
/// After that it derives the install directory as `APPS:[<unique_identifier>]`
/// with `get_application_dir()` and creates it with `mkdir_p()`. The install
/// directory is created before the `application_t` is allocated and before the
/// JSON metadata is written.
///
/// A successful allocation path then:
///
/// - allocates a zeroed `application_t` with `why_calloc()`
/// - duplicates `unique_identifier`, `name`, `author`, `version`, and
///   `interpreter` with `why_strdup()`
/// - stores the already-allocated `app_dir` as `installed_path`
/// - writes the `source` enum by casting away the `const` qualifier on that
///   field
/// - serializes the object to JSON and writes it to `APPS:<uid>.json`
///
/// The `source` value is stored and serialized verbatim. There is no range
/// check against `APPLICATION_SOURCE_MAX`.
///
/// The JSON payload contains exactly these keys:
/// `unique_identifier`, `name`, `author`, `version`, `interpreter`,
/// `metadata_file`, `binary_path`, and `source`.
///
/// # Failure paths and subtleties
///
/// The function returns null if path construction, duplicate detection,
/// directory creation, object allocation, JSON construction, JSON rendering, or
/// opening the metadata file for writing fails.
///
/// Individual `why_strdup()` results are not checked. If one of those
/// allocations fails but `save_application_metadata()` still succeeds, the
/// application can be created with some fields silently null in memory and then
/// persisted as empty strings in JSON.
///
/// `unique_identifier` is the exception to that "silent" pattern: if
/// `why_strdup(unique_identifier)` fails, `save_application_metadata()` rejects
/// the object because `app->unique_identifier == NULL`, so creation returns
/// null after freeing the heap object.
///
/// Once the metadata file is open, upstream ignores `why_fputs()` and
/// `why_fclose()` results. A short write or flush failure can therefore still
/// report success.
///
/// If metadata saving fails after the install directory was already created,
/// upstream calls `application_free(app)` but does not remove the newly created
/// directory. Failed creates can therefore leave behind an empty
/// `APPS:[<unique_identifier>]` directory with no matching `.json` file.
///
/// # Interactions with other features
///
/// The OTA updater uses this function to materialize placeholder application
/// records for default apps before any payload files are downloaded. The launcher
/// and OTA scanning paths discover applications by listing `*.json` files in the
/// base directory, so creating the metadata file is what makes an application
/// visible to those features.
#[unsafe(no_mangle)]
pub extern "C" fn application_create(
    unique_identifier: *const ::core::ffi::c_char,
    name: *const ::core::ffi::c_char,
    author: *const ::core::ffi::c_char,
    version: *const ::core::ffi::c_char,
    interpreter: *const ::core::ffi::c_char,
    source: application_source_t,
) -> *mut application_t {
    application_create_inner(unique_identifier, name, author, version, interpreter, source)
}
/// Set or clear the metadata-file path stored in an application snapshot.
///
/// # Exact upstream behavior
///
/// Upstream rejects a null `application` pointer with `false`.
///
/// If `metadata_file != NULL`, it validates the path with `validate_path()`. The
/// validation logic is not just a syntax check on the provided string. It first
/// concatenates `app->installed_path` and `metadata_file` with `path_concat()`,
/// then requires the resulting full path to pass `parse_path()`.
///
/// More precisely, `path_concat()` accepts a device-less append path with an
/// optional leading `[dir.subdir]` segment and an optional trailing filename.
/// Practical examples that pass are `file.ext`, `[dir.subdir]file.ext`, and the
/// directory-only form `[dir.subdir]`. Empty brackets such as `[]file.ext` are
/// also accepted and behave like `file.ext`.
///
/// Device-qualified paths, slash-separated Unix-style paths, malformed bracket
/// forms, and strings containing characters outside BadgeVMS path syntax are all
/// rejected. If `app->installed_path` itself is null, validation of any non-null
/// `metadata_file` also fails because `path_concat()` returns null and
/// `parse_path(NULL, ...)` reports an empty path.
///
/// On a valid request upstream:
///
/// - frees the previous `app->metadata_file`
/// - assigns `why_strdup(metadata_file)` (or null if the caller passed null)
/// - rewrites the whole JSON metadata file with `save_application_metadata()`
///
/// The setter returns the boolean result of `save_application_metadata()`.
///
/// # Subtleties
///
/// The in-memory object is mutated before the JSON save is attempted, so a save
/// failure leaves the caller's `application_t` changed even though the function
/// returns `false`.
///
/// Passing `metadata_file = NULL` is the supported clear operation.
/// `why_strdup(NULL)` returns null, and a successful save rewrites the JSON
/// field as `""`.
///
/// If `why_strdup()` fails, upstream stores null and still attempts to save. A
/// successful save in that case persists the field as an empty JSON string.
///
/// Once the metadata file is open, late write or close failures are ignored for
/// the same reason described in the module-level storage model above.
///
/// BadgeVMS itself does not interpret the contents or existence of this file.
/// The setter only stores the relative path string.
#[unsafe(no_mangle)]
pub extern "C" fn application_set_metadata(
    application: *mut application_t,
    metadata_file: *const ::core::ffi::c_char,
) -> bool {
    application_set_metadata_inner(application, metadata_file)
}
/// Set or clear the relative main-binary path stored in an application snapshot.
///
/// # Exact upstream behavior
///
/// This follows the same structure as `application_set_metadata()`:
///
/// - null `application` returns `false`
/// - non-null `binary_path` must pass `validate_path()` relative to
///   `app->installed_path`
/// - the previous `app->binary_path` is freed
/// - `app->binary_path` is replaced with `why_strdup(binary_path)` or null
/// - the full JSON file is rewritten with `save_application_metadata()`
///
/// The stored value is relative, not absolute. Launch later reconstructs the
/// executable path with `path_concat(app->installed_path, app->binary_path)`.
/// The exact accepted relative-path grammar is the same as for
/// `application_set_metadata()`: device-less, with an optional bracketed
/// directory prefix and an optional filename.
///
/// # Subtleties
///
/// The same mutation-before-save behavior applies here: if JSON rewriting fails,
/// the caller's in-memory snapshot has already changed.
///
/// Passing `binary_path = NULL` clears the field in memory and, on successful
/// save, rewrites the JSON field as `""`. Passing an actual empty string is not
/// accepted by the setter: `validate_path()` rejects it because `path_concat()`
/// refuses empty append paths.
///
/// Individual allocation failure is again silent. A failed `why_strdup()` can be
/// persisted as an empty-string binary path if the later JSON write succeeds.
/// Once loaded back from disk, that empty JSON string becomes a non-null empty
/// C string, which is why `application_launch()` can reach its later
/// `path_concat()` failure path instead of failing the initial null check.
///
/// Once the metadata file is open, late write or close failures are ignored.
///
/// The launcher filters its application list partly on whether `binary_path` is
/// non-null and non-empty, so changing this field directly affects what appears
/// launchable in the UI.
#[unsafe(no_mangle)]
pub extern "C" fn application_set_binary_path(
    application: *mut application_t,
    binary_path: *const ::core::ffi::c_char,
) -> bool {
    application_set_binary_path_inner(application, binary_path)
}
/// Replace the version string in an application snapshot and persist it.
///
/// # Exact upstream behavior
///
/// Upstream only checks that `application != NULL`. It performs no validation on
/// `version`.
///
/// It then:
///
/// - frees the old `app->version`
/// - assigns `why_strdup(version)`
/// - rewrites the whole metadata JSON via `save_application_metadata()`
///
/// The return value is exactly the result of that JSON rewrite.
///
/// # Subtleties
///
/// Passing `version = NULL` clears the in-memory pointer, and a successful save
/// rewrites the JSON field as `""`.
///
/// As with the path setters, the in-memory object is modified before saving, and
/// a failed `why_strdup()` can still be serialized as an empty string if writing
/// the JSON file succeeds.
/// Late write or close failures after opening the metadata file are ignored.
///
/// The OTA updater uses this setter after successful downloads, so this field is
/// the firmware's authoritative installed-version record.
#[unsafe(no_mangle)]
pub extern "C" fn application_set_version(
    application: *mut application_t,
    version: *const ::core::ffi::c_char,
) -> bool {
    application_set_version_inner(application, version)
}
/// Replace the author string in an application snapshot and persist it.
///
/// Upstream behavior is structurally identical to `application_set_version()`:
/// null `application` returns `false`, otherwise the old string is freed, the
/// new pointer is set to `why_strdup(author)`, and the full metadata JSON is
/// rewritten. There is no validation of the author string, `author = NULL`
/// clears the field and persists as `""` on successful save, late write errors
/// are ignored after opening the file, and the object is mutated before save
/// success is known.
#[unsafe(no_mangle)]
pub extern "C" fn application_set_author(
    application: *mut application_t,
    author: *const ::core::ffi::c_char,
) -> bool {
    application_set_author_inner(application, author)
}
/// Replace the human-readable application name and persist it.
///
/// Upstream behavior is again the same as the other simple string setters:
/// null `application` returns `false`; otherwise the previous `name` is freed,
/// the new one is assigned with `why_strdup(name)`, and the whole JSON metadata
/// file is rewritten. There is no validation of the content, `name = NULL`
/// clears the field and persists as `""` on successful save, late write errors
/// are ignored after opening the file, and the object is mutated before the
/// save result is known.
///
/// The launcher displays this field directly when rendering the installed-app
/// list.
#[unsafe(no_mangle)]
pub extern "C" fn application_set_name(
    application: *mut application_t,
    name: *const ::core::ffi::c_char,
) -> bool {
    application_set_name_inner(application, name)
}
/// Replace the interpreter string and persist it.
///
/// Upstream only checks `application != NULL`, frees the old interpreter,
/// assigns `why_strdup(interpreter)`, and rewrites the full JSON metadata.
/// There is no validation, `interpreter = NULL` clears the field and persists
/// as `""` on successful save, late write errors are ignored after opening the
/// file, and the same mutation-before-save behavior applies.
///
/// The currently shipped `application_launch()` implementation ignores this
/// field completely, so setting it affects metadata consumers but not launch
/// semantics.
#[unsafe(no_mangle)]
pub extern "C" fn application_set_interpreter(
    application: *mut application_t,
    interpreter: *const ::core::ffi::c_char,
) -> bool {
    application_set_interpreter_inner(application, interpreter)
}
/// Remove an application's install directory from storage.
///
/// # Exact upstream behavior
///
/// Upstream rejects null `application` and null `app->unique_identifier` with
/// `false`.
///
/// It then rebuilds the install directory path with
/// `get_application_dir(unique_id)`, logs `"Attempting to recursively delete %s"`,
/// and calls `rm_rf(app_dir)`. If `get_application_dir()` itself fails it logs
/// `"No valid app_dir for %s"` and returns `false`.
///
/// The function returns only the boolean result of `rm_rf(app_dir)`.
/// `rm_rf()` itself returns `true` when the target path does not exist, so a
/// missing install directory is treated as a successful destroy.
///
/// # Important upstream bug
///
/// The metadata JSON file is not inside the install directory. It lives beside
/// it at `APPS:<unique_identifier>.json`, while the install directory is
/// `APPS:[<unique_identifier>]`. `application_destroy()` only deletes the
/// directory tree. It does not remove the metadata JSON file.
///
/// As a result, the current upstream implementation can report success while
/// leaving behind a perfectly loadable metadata record. `application_get()` and
/// `application_list()` will continue to find the application until something
/// else deletes the stale `.json` file.
///
/// # Other omissions and interactions
///
/// This function does not free the passed `application_t`; callers still need to
/// call `application_free()` on their local snapshot.
///
/// It also does not check whether the application is running, does not try to
/// stop running tasks, and does not coordinate with the launcher or startup
/// manager. It is purely a filesystem deletion attempt.
#[unsafe(no_mangle)]
pub extern "C" fn application_destroy(application: *mut application_t) -> bool {
    application_destroy_inner(application)
}
/// Create or truncate a file relative to an application's install directory.
///
/// # Exact upstream behavior
///
/// Upstream first calls `application_create_file_string(app, file_path)`.
///
/// If that returns null, this function returns null.
/// Otherwise it opens the resolved absolute path with `why_fopen(..., "w")`,
/// frees the temporary absolute path string, and returns the resulting `FILE *`.
///
/// Using mode `"w"` means an existing file is truncated and a missing file is
/// created if the underlying filesystem permits it.
///
/// # Side effects inherited from `application_create_file_string`
///
/// All parent directories are created before the file is opened. Those directory
/// creations happen even if `why_fopen()` then fails, so callers can observe
/// partial side effects where directories exist but the file was never created.
///
/// If `file_path` uses the directory-only relative form `[dir.subdir]`, the
/// helper still resolves that path and `application_create_file()` then tries to
/// open the resulting directory path with mode `"w"`. The directory creation
/// side effects still occur first.
///
/// The OTA updater relies on this helper family when staging downloaded
/// application content into the install tree.
#[unsafe(no_mangle)]
pub extern "C" fn application_create_file(
    application: *mut application_t,
    file_path: *const ::core::ffi::c_char,
) -> *mut FILE {
    application_create_file_inner(application, file_path)
}
/// Build an absolute BadgeVMS path for a file inside an application's install
/// directory and ensure all parent directories exist.
///
/// # Exact upstream behavior
///
/// Upstream rejects null `application`, null `file_path`, or null
/// `app->installed_path`.
///
/// It then computes the candidate absolute path with
/// `path_concat(app->installed_path, file_path)`. `path_concat()` accepts a
/// device-less append path with an optional leading `[dir.subdir]` segment and
/// an optional trailing filename. That means `file.ext`,
/// `[dir.subdir]file.ext`, and `[dir.subdir]` are all accepted. Empty brackets
/// such as `[]file.ext` are also accepted and behave like `file.ext`.
///
/// Device-qualified paths, slash-separated paths, malformed bracket forms, or
/// invalid path characters are rejected.
///
/// On a syntactically valid path it then:
///
/// - logs `"Attempting to create %s"`
/// - computes the parent directory with `path_dirname(absolute_file_path)`
/// - logs `"Creating directory %s"`
/// - calls `mkdir_p(dirname)`
/// - returns the heap-allocated `absolute_file_path` string on success
///
/// `mkdir_p()` itself walks each dotted directory component and creates missing
/// directories with `why_mkdir(..., 0755)`. If an intermediate path already
/// exists but is not a directory, the call fails.
///
/// # Important side effects and quirks
///
/// Directories are created even if the caller never creates the file afterward.
/// The OTA updater intentionally depends on that behavior when preparing nested
/// install paths.
/// OTA also uses placeholder application records created from metadata alone, so
/// this helper can be invoked before any real payload file exists.
///
/// Upstream leaks the `dirname` string on the success path. It is only freed on
/// failure. Every successful `application_create_file_string()` call therefore
/// leaks one heap allocation in the current firmware.
///
/// The returned string is a fresh heap allocation. Upstream callers free it with
/// `free()`/`why_free()` after use.
#[unsafe(no_mangle)]
pub extern "C" fn application_create_file_string(
    application: *mut application_t,
    file_path: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    application_create_file_string_inner(application, file_path)
}
/// Enumerate installed applications by scanning metadata JSON files.
///
/// # Exact upstream behavior
///
/// Upstream returns null immediately if the hidden `applications_base_dir` has
/// not been initialized.
///
/// Otherwise it opens that directory with `why_opendir()` and scans for entries
/// whose names end in `.json`. Only those files count as installed
/// applications. Bare install directories without metadata files are invisible,
/// while stale metadata files remain visible even if the payload directory has
/// been deleted.
///
/// The function allocates an `application_list_t` with `why_calloc()` and uses a
/// two-pass directory walk:
///
/// - first pass counts `.json` files
/// - second pass allocates an array of `application_t *`, strips the `.json`
///   suffix from each filename, and calls `load_application_metadata()`
///
/// Each successfully loaded metadata file becomes one heap-allocated
/// `application_t` stored in the list. Failed loads are silently skipped.
/// The OTA updater intentionally relies on that metadata-file-only discovery:
/// its placeholder `.json` records become visible immediately, even before the
/// application has a usable payload.
///
/// If `out != NULL` and at least one application loaded successfully,
/// `*out` is set to the first loaded entry. If no entries were loaded,
/// `*out = NULL`.
///
/// # Ordering and ownership
///
/// Enumeration order is the raw `why_readdir()` order. There is no sorting.
///
/// The returned list owns all loaded `application_t` snapshots. They are meant
/// to stay alive until `application_list_close()` frees them.
///
/// # Important quirks
///
/// If no `.json` files exist, upstream still returns a valid, empty list object
/// rather than null.
///
/// If `out == NULL`, the first application remains stored in the list but there
/// is no public API to retrieve it later: `application_list_get_next()` advances
/// before returning anything. In that case the first entry is effectively lost
/// to the caller.
#[unsafe(no_mangle)]
pub extern "C" fn application_list(out: *mut *mut application_t) -> application_list_handle {
    application_list_inner(out)
}
/// Advance an application list iterator and return the next loaded application.
///
/// # Exact upstream behavior
///
/// Upstream rejects null `list` with null.
///
/// Otherwise it increments `list->current_index` first, then returns
/// `list->applications[list->current_index]` if the incremented index is still
/// `< list->count`. If the incremented index reaches or exceeds `count`, it
/// returns null.
///
/// Because the increment happens before the bounds check, the first application
/// in a list is never returned by this function. Upstream expects callers to get
/// the first entry from the `out` parameter of `application_list()` and only use
/// this function for subsequent entries.
///
/// The returned pointer is owned by the list and must not be freed separately if
/// the caller also intends to call `application_list_close()`.
#[unsafe(no_mangle)]
pub extern "C" fn application_list_get_next(list: application_list_handle) -> *mut application_t {
    application_list_get_next_inner(list)
}
/// Free an application list and every application snapshot currently stored in
/// it.
///
/// # Exact upstream behavior
///
/// Upstream rejects null `list` by returning immediately.
///
/// Otherwise it walks `list->applications[0..count)`, calls `application_free()`
/// on each non-null entry, then frees the application-pointer array and finally
/// the list object itself.
///
/// This means the list owns the `application_t *` values it produced. If the
/// caller frees one of those snapshots manually and then later calls
/// `application_list_close()`, the current firmware will double-free that
/// snapshot.
#[unsafe(no_mangle)]
pub extern "C" fn application_list_close(list: application_list_handle) {
    application_list_close_inner(list)
}
/// Load one application snapshot from its metadata JSON file.
///
/// # Exact upstream behavior
///
/// Upstream rejects null `unique_identifier` with null.
/// An uninitialized `applications_base_dir` or an identifier that cannot be
/// turned into a legal `APPS:<uid>.json` path also returns null before any file
/// I/O happens.
///
/// It then builds the metadata path `APPS:<unique_identifier>.json`, opens it
/// with `why_fopen(..., "r")`, reads the entire file into memory, parses it with
/// `cJSON_Parse()`, and reconstructs a fresh `application_t` with
/// `json_to_application()`.
///
/// `json_to_application()` duplicates each recognized JSON string field with
/// `why_strdup()` and reconstructs `installed_path` from the unique identifier
/// instead of trusting JSON to provide it. The numeric `source` field is written
/// into the `const` enum member by casting away constness.
/// Unknown keys are ignored, missing or wrongly typed keys are left at their
/// zero-initialized defaults, and persisted empty JSON strings reload as
/// non-null empty C strings.
///
/// The returned object is independent of any earlier snapshot. Callers own it
/// and must free it with `application_free()`.
///
/// # Subtleties
///
/// Upstream does not verify that the corresponding install directory actually
/// exists. A stale metadata file is enough for this function to succeed.
///
/// File-size and read results are not checked rigorously. The code trusts
/// `why_ftell()` and `why_fread()` and only treats JSON parse failure as the
/// final validity check.
#[unsafe(no_mangle)]
pub extern "C" fn application_get(
    unique_identifier: *const ::core::ffi::c_char,
) -> *mut application_t {
    application_get_inner(unique_identifier)
}
/// Free a heap-allocated application snapshot.
///
/// # Exact upstream behavior
///
/// Upstream rejects null `application` by returning immediately.
///
/// Otherwise it frees, in order:
///
/// - `unique_identifier`
/// - `name`
/// - `author`
/// - `version`
/// - `interpreter`
/// - `metadata_file`
/// - `installed_path`
/// - `binary_path`
/// - the `application_t` itself
///
/// This is purely a heap-object destructor. It does not remove files from disk,
/// update the application list, or affect running tasks.
#[unsafe(no_mangle)]
pub extern "C" fn application_free(application: *mut application_t) {
    application_free_inner(application)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        emulated::badgevms::{
            fs::paths::{TestBaseDirectoryGuard, set_base_directory_for_tests},
            misc::{get_num_tasks, wait},
            misc::runtime::reset_runtime_for_tests,
        },
        free,
    };
    use std::{
        ffi::{CStr, CString},
        fs,
        os::unix::fs::PermissionsExt,
        path::PathBuf,
        ptr,
        thread,
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    };

    struct TestApplicationDirectory {
        root: PathBuf,
        _guard: TestBaseDirectoryGuard,
    }

    impl TestApplicationDirectory {
        fn new(test_name: &str) -> Self {
            let suffix = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after the unix epoch")
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "why2025-applications-test-{test_name}-{}-{suffix}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&root);
            let guard = set_base_directory_for_tests(root.clone());
            reset_runtime_for_tests();

            Self {
                root,
                _guard: guard,
            }
        }

        fn metadata_file(&self, unique_identifier: &str) -> PathBuf {
            self.root.join("APPS").join(format!("{unique_identifier}.json"))
        }

        fn install_directory(&self, unique_identifier: &str) -> PathBuf {
            self.root.join("APPS").join(unique_identifier)
        }
    }

    impl Drop for TestApplicationDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn wait_for_task_count(expected: u32) {
        let start = Instant::now();
        while get_num_tasks() != expected {
            assert!(
                start.elapsed() < Duration::from_secs(5),
                "timed out waiting for task count {expected}, got {}",
                get_num_tasks()
            );
            thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn create_and_reload_persist_metadata() {
        let directory = TestApplicationDirectory::new("create-and-reload");
        let unique_identifier = CString::new("com_example_app").unwrap();
        let name = CString::new("Example App").unwrap();
        let version = CString::new("1.0.0").unwrap();
        let metadata_file = CString::new("[config]metadata.json").unwrap();
        let binary_path = CString::new("run.sh").unwrap();

        let application = application_create(
            unique_identifier.as_ptr(),
            name.as_ptr(),
            ptr::null(),
            version.as_ptr(),
            ptr::null(),
            application_source_t::APPLICATION_SOURCE_BADGEHUB,
        );

        assert!(!application.is_null());
        assert!(directory.metadata_file("com_example_app").is_file());
        assert!(directory.install_directory("com_example_app").is_dir());
        assert!(application_set_metadata(application, metadata_file.as_ptr()));
        assert!(application_set_binary_path(application, binary_path.as_ptr()));

        application_free(application);

        let reloaded = application_get(unique_identifier.as_ptr());
        assert!(!reloaded.is_null());
        let reloaded_ref = unsafe { &*reloaded };
        assert_eq!(
            unsafe { CStr::from_ptr(reloaded_ref.unique_identifier) }
                .to_str()
                .unwrap(),
            "com_example_app"
        );
        assert_eq!(
            unsafe { CStr::from_ptr(reloaded_ref.name) }.to_str().unwrap(),
            "Example App"
        );
        assert_eq!(
            unsafe { CStr::from_ptr(reloaded_ref.version) }
                .to_str()
                .unwrap(),
            "1.0.0"
        );
        assert_eq!(
            unsafe { CStr::from_ptr(reloaded_ref.metadata_file) }
                .to_str()
                .unwrap(),
            "[config]metadata.json"
        );
        assert_eq!(
            unsafe { CStr::from_ptr(reloaded_ref.binary_path) }
                .to_str()
                .unwrap(),
            "run.sh"
        );
        assert_eq!(
            reloaded_ref.source,
            application_source_t::APPLICATION_SOURCE_BADGEHUB
        );

        application_free(reloaded);
    }

    #[test]
    fn list_uses_out_parameter_for_first_entry() {
        let _directory = TestApplicationDirectory::new("list-first-entry");
        let first_uid = CString::new("com_example_first").unwrap();
        let second_uid = CString::new("com_example_second").unwrap();

        let first = application_create(
            first_uid.as_ptr(),
            first_uid.as_ptr(),
            ptr::null(),
            ptr::null(),
            ptr::null(),
            application_source_t::APPLICATION_SOURCE_UNKNOWN,
        );
        let second = application_create(
            second_uid.as_ptr(),
            second_uid.as_ptr(),
            ptr::null(),
            ptr::null(),
            ptr::null(),
            application_source_t::APPLICATION_SOURCE_UNKNOWN,
        );

        assert!(!first.is_null());
        assert!(!second.is_null());

        let mut current = ptr::null_mut();
        let list = application_list(&mut current);
        assert!(!list.is_null());
        assert!(!current.is_null());

        let next = application_list_get_next(list);
        assert!(!next.is_null());
        assert_ne!(current, next);
        assert!(application_list_get_next(list).is_null());

        application_list_close(list);
        application_free(first);
        application_free(second);
    }

    #[test]
    fn destroy_removes_install_directory_but_leaves_metadata() {
        let directory = TestApplicationDirectory::new("destroy-leaves-metadata");
        let unique_identifier = CString::new("com_example_destroy").unwrap();

        let application = application_create(
            unique_identifier.as_ptr(),
            unique_identifier.as_ptr(),
            ptr::null(),
            ptr::null(),
            ptr::null(),
            application_source_t::APPLICATION_SOURCE_UNKNOWN,
        );

        assert!(!application.is_null());
        assert!(application_destroy(application));
        assert!(!directory.install_directory("com_example_destroy").exists());
        assert!(directory.metadata_file("com_example_destroy").is_file());

        let stale = application_get(unique_identifier.as_ptr());
        assert!(!stale.is_null());

        application_free(stale);
        application_free(application);
    }

    #[test]
    fn create_file_string_creates_parent_directories() {
        let directory = TestApplicationDirectory::new("create-file-string");
        let unique_identifier = CString::new("com_example_files").unwrap();
        let file_path = CString::new("[config.subdir]settings.json").unwrap();

        let application = application_create(
            unique_identifier.as_ptr(),
            unique_identifier.as_ptr(),
            ptr::null(),
            ptr::null(),
            ptr::null(),
            application_source_t::APPLICATION_SOURCE_UNKNOWN,
        );

        assert!(!application.is_null());
        let absolute_path = application_create_file_string(application, file_path.as_ptr());
        assert!(!absolute_path.is_null());
        assert_eq!(
            unsafe { CStr::from_ptr(absolute_path) }.to_str().unwrap(),
            "APPS:[com_example_files.config.subdir]settings.json"
        );
        assert!(
            directory
                .install_directory("com_example_files")
                .join("config")
                .join("subdir")
                .is_dir()
        );

        unsafe {
            free(absolute_path.cast());
        }
        application_free(application);
    }

    #[test]
    fn launch_runs_relative_binary_path() {
        let directory = TestApplicationDirectory::new("launch");
        let unique_identifier = CString::new("com_example_launch").unwrap();
        let binary_path = CString::new("run.sh").unwrap();

        let application = application_create(
            unique_identifier.as_ptr(),
            unique_identifier.as_ptr(),
            ptr::null(),
            ptr::null(),
            ptr::null(),
            application_source_t::APPLICATION_SOURCE_UNKNOWN,
        );

        assert!(!application.is_null());
        assert!(application_set_binary_path(application, binary_path.as_ptr()));

        let script_path = directory.install_directory("com_example_launch").join("run.sh");
        fs::write(&script_path, "#!/bin/sh\nexit 0\n").unwrap();
        let mut permissions = fs::metadata(&script_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script_path, permissions).unwrap();

        let base_count = get_num_tasks();
        let pid = application_launch(unique_identifier.as_ptr());

        assert!(pid > 0);
        wait_for_task_count(base_count);
        assert_eq!(wait(true, 0), pid);
        assert_eq!(get_num_tasks(), base_count);

        application_free(application);
    }
}
