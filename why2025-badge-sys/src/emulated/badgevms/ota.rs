use crate::types::*;

mod runtime;

pub(crate) use runtime::abort_task_owned_ota_session;
use runtime::{
    ota_get_invalid_version_inner, ota_get_running_version_inner, ota_session_abort_inner,
    ota_session_commit_inner, ota_session_open_inner, ota_write_inner,
};

// Emulated stubs for BadgeVMS OTA functions.
//
// The rustdoc on the items in this file is derived from the upstream firmware implementation in
// `WHY2025/team-badge/firmware` at commit `a548d825a3295432d374939607feb552eb505210`
// (`Update espressif/eppp_link`). The goal is to document the exact implementation behavior of the
// current firmware, including details that are only visible in the `.c` files and not in the
// public headers.

/// Open the singleton OTA write session used by the firmware.
///
/// # Exact upstream behavior
///
/// Upstream does not allocate a fresh session object per call. It owns one process-global static
/// `ota_session_t session` and returns `&session` on success.
///
/// Opening works like this:
///
/// - `atomic_flag_test_and_set(&session.open)` guards the singleton; if the flag was already set,
///   the call returns `NULL` immediately with no log output
/// - the current task records `RES_OTA` ownership for `&session`, but only if it is a managed task
///   with a non-zero BadgeVMS PID; kernel-context callers are not resource-tracked
/// - the function snapshots the configured boot partition and the currently running partition into
///   `session.configured` and `session.running`
/// - it logs both partition offsets with `ESP_LOGW`
/// - it chooses the target partition via `esp_ota_get_next_update_partition(NULL)`
/// - if no update partition exists, it clears `session.open` and returns `NULL`
/// - otherwise it starts ESP-IDF OTA with
///   `esp_ota_begin(session.update_partition, OTA_WITH_SEQUENTIAL_WRITES, &session.update_handle)`
///
/// On successful open the function stores `false` into `session.error` and returns the address of
/// the static singleton.
///
/// # Failure paths and subtleties
///
/// If `esp_ota_begin` fails, upstream logs the error and then calls `ota_session_abort(&session)`.
/// That abort path:
///
/// - calls `esp_ota_abort(session.update_handle)` even though `update_handle` may still be zero or
///   stale from an earlier attempt
/// - removes the session from the current task's resource table if it had been recorded there
/// - clears `session.open`
/// - stores `true` into `session.error`
///
/// The failure return is therefore `NULL`, but it also leaves the singleton in an explicitly
/// aborted state until the next successful `ota_session_open()` resets `session.error` back to
/// `false`.
///
/// Because the handle is always the same static address, stale handles from older sessions alias the
/// same storage reused by later sessions.
///
/// # Interactions with other features
///
/// Managed-task teardown in `task.c` treats `RES_OTA` specially and calls `ota_session_abort(ptr)`
/// during cleanup. That means a task that dies with an open OTA session will implicitly abort it,
/// but only if the session was opened from a managed task in the first place.
#[unsafe(no_mangle)]
pub extern "C" fn ota_session_open() -> ota_handle_t {
    ota_session_open_inner()
}
/// Append one block of image data to the currently open OTA session.
///
/// # Exact upstream behavior
///
/// Upstream assumes `session` is a valid pointer to an `ota_session_t`; it does not check for
/// `NULL`, confirm that the pointer is the singleton returned by `ota_session_open()`, or validate
/// `buffer`.
///
/// The implementation is:
///
/// - if `session->error` is already `true`, return `false` immediately
/// - otherwise call `esp_ota_write(session->update_handle, buffer, block_size)`
/// - if `esp_ota_write` returns an error, call `ota_session_abort(session)` and return `false`
/// - on success return `true`
///
/// `block_size` is accepted as a signed `int` because that is the BadgeVMS ABI, but upstream does
/// not range-check it before handing it to ESP-IDF. Negative values therefore rely entirely on the
/// behavior of the C integer conversion at the call boundary.
///
/// # Buffer ownership and side effects
///
/// The OTA layer does not copy the caller's buffer on its own; it simply forwards the pointer to
/// `esp_ota_write` for that synchronous call. The in-tree OTA apps therefore allocate their own
/// temporary heap buffers inside the libcurl write callback before calling `ota_write()`, but that
/// buffering is a caller-side choice rather than a property of the API itself.
///
/// A failed write is terminal for the session in current upstream: `ota_write()` auto-aborts the
/// session, clears the open flag, removes the `RES_OTA` resource entry, and sets `session.error`.
#[unsafe(no_mangle)]
pub extern "C" fn ota_write(
    session: ota_handle_t,
    buffer: *mut ::core::ffi::c_void,
    block_size: ::core::ffi::c_int,
) -> bool {
    ota_write_inner(session, buffer, block_size)
}
/// Finalize the pending OTA image and mark its partition for the next boot.
///
/// # Exact upstream behavior
///
/// Upstream again assumes `session` is valid and does no pointer validation.
///
/// The commit sequence is:
///
/// - if `session->error` is already `true`, return `false` immediately
/// - call `esp_ota_end(session->update_handle)`
/// - if that fails with `ESP_ERR_OTA_VALIDATE_FAILED`, log `Image validation failed, image is corrupted`
/// - if it fails with any other error, log `esp_ota_end failed (...)!`
/// - on any `esp_ota_end` failure, return `false`
/// - otherwise call `esp_ota_set_boot_partition(session->update_partition)`
/// - if that fails, log `esp_ota_set_boot_partition failed (...)!` and return `false`
/// - on full success, remove `RES_OTA` from the current task resource table, clear `session.open`,
///   and return `true`
///
/// # Important asymmetry on failure
///
/// Commit failures do not go through `ota_session_abort()`. If `esp_ota_end()` or
/// `esp_ota_set_boot_partition()` fails, upstream leaves:
///
/// - `session.open` set
/// - the `RES_OTA` resource record still present for the owning managed task
/// - `session.error` unchanged
///
/// So unlike `ota_write()`, a failed commit does not automatically close the session. The caller is
/// expected to decide whether to retry or explicitly abort.
///
/// # Interactions with boot and rollback
///
/// Successful commit only selects the update partition for the next boot. It does not reboot the
/// badge, does not validate the image immediately, and does not change the currently running
/// firmware version in-place.
///
/// The validation step happens later during boot: `init.c` calls `validate_ota_partition()` once
/// startup configuration has loaded successfully, and that function marks a `PENDING_VERIFY` image
/// as valid via `esp_ota_mark_app_valid_cancel_rollback()`. Conversely, various early startup
/// failures in `why2025_firmware.c` call `invalidate_ota_partition()`, which asks ESP-IDF to mark
/// the running image invalid and reboot into rollback.
///
/// The in-tree OTA UIs reflect that contract: they call `ota_session_commit()` and then tell the
/// user to restart the badge rather than expecting an automatic reboot.
#[unsafe(no_mangle)]
pub extern "C" fn ota_session_commit(session: ota_handle_t) -> bool {
    ota_session_commit_inner(session)
}
/// Abort the singleton OTA session and release its resource bookkeeping.
///
/// # Exact upstream behavior
///
/// Upstream unconditionally performs these steps:
///
/// - call `esp_ota_abort(session->update_handle)`
/// - remove `RES_OTA` from the current task's resource table
/// - clear `session.open`
/// - store `true` into `session.error`
/// - return `true`
///
/// The return value is therefore not an indication of whether ESP-IDF accepted the abort. Upstream
/// ignores the result of `esp_ota_abort()` entirely.
///
/// # Subtleties
///
/// As with the other OTA entry points, there is no pointer validation. Passing `NULL` or a bogus
/// handle would dereference invalid memory in upstream C.
///
/// Because the underlying storage is the singleton static session object, aborting through any live
/// alias poisons the shared session state for all aliases until a later successful
/// `ota_session_open()` resets `session.error`.
///
/// # Interactions with task cleanup
///
/// `thread_delete()` in `task.c` invokes this function automatically for leaked `RES_OTA` resources.
/// That gives managed tasks a best-effort cleanup path for abandoned OTA sessions.
#[unsafe(no_mangle)]
pub extern "C" fn ota_session_abort(session: ota_handle_t) -> bool {
    ota_session_abort_inner(session)
}
/// Return the version string of the currently running firmware image.
///
/// # Exact upstream behavior
///
/// Upstream fetches the running partition with `esp_ota_get_running_partition()`, then asks ESP-IDF
/// for its `esp_app_desc_t` via `esp_ota_get_partition_description()`.
///
/// If that description lookup fails, the function logs `Could not get running partition` and returns
/// `false`.
///
/// On success it allocates a new heap string with `strdup(running_app_info.version)`, stores that
/// pointer through `*version`, and returns `true`.
///
/// # Ownership and ABI quirks
///
/// The public header now declares `char **version`, and the implementation really does expect an
/// out-pointer for a newly allocated string. The caller owns the returned `strdup()` result and must
/// free it.
///
/// Upstream does not:
///
/// - check whether `version` itself is `NULL`
/// - free any previous string already stored in `*version`
/// - copy into a caller-provided fixed-size buffer
///
/// The stale comment in `ota.c` still says the argument is a `char[32]`, but that is no longer what
/// the implementation does.
///
/// # Interaction with other features
///
/// In-tree callers currently expose an ABI transition in progress:
///
/// - `sdk_apps/why2025_ota` uses a compatibility workaround that allocates a 32-byte buffer, calls
///   `ota_get_running_version(&running)`, and then frees either the old buffer or the new string
///   depending on which ABI it observed
/// - `sdk_apps/ota_wifi_update` still calls `ota_get_running_version(running)` using the old
///   one-pointer style, which no longer matches the implementation in `ota.c`
#[unsafe(no_mangle)]
pub extern "C" fn ota_get_running_version(version: *mut *mut ::core::ffi::c_char) -> bool {
    ota_get_running_version_inner(version)
}
/// Return the version string of the last partition ESP-IDF marked invalid.
///
/// # Exact upstream behavior
///
/// Upstream first calls `esp_ota_get_last_invalid_partition()`.
///
/// - if that returns `NULL`, the function returns `false` with no log output
/// - otherwise it calls `esp_ota_get_partition_description(last_invalid_app, &invalid_app_info)`
/// - if description lookup fails, it logs `Could not get invalid partition info` and returns `false`
/// - on success it allocates `strdup(invalid_app_info.version)`, stores it through `*version`, and
///   returns `true`
///
/// # Ownership and API caveats
///
/// Ownership matches `ota_get_running_version()`: the caller receives a freshly allocated string and
/// is responsible for freeing it.
///
/// Upstream does not validate that `version` is non-null, and it does not free any prior value in
/// `*version` before overwriting it.
///
/// # Interaction with rollback logic
///
/// The meaning of "invalid" here is entirely driven by ESP-IDF OTA state, not by BadgeVMS-local
/// bookkeeping.
///
/// In the current firmware that state is affected by both sides of the boot flow:
///
/// - successful boots eventually call `validate_ota_partition()` from `init.c`, which marks a
///   pending image valid and cancels rollback
/// - several fatal startup failures in `why2025_firmware.c` call `invalidate_ota_partition()`,
///   which tells ESP-IDF to mark the running image invalid and reboot
///
/// So this accessor can surface versions that failed image verification, versions that booted but
/// were later invalidated during startup, or any other image ESP-IDF currently considers the last
/// invalid rollback candidate.
#[unsafe(no_mangle)]
pub extern "C" fn ota_get_invalid_version(version: *mut *mut ::core::ffi::c_char) -> bool {
    ota_get_invalid_version_inner(version)
}
