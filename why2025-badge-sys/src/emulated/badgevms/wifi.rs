//! Detailed reference for the upstream BadgeVMS Wi-Fi ABI.
//!
//! These comments describe the current upstream C implementation in
//! `firmware/badgevms/drivers/wifi.c` and `firmware/badgevms/wrapped_funcs.c`
//! as it actually behaves today, including observable quirks, blocking
//! behavior, and implementation bugs. The intent is to preserve the firmware's
//! real contract for a later Rust implementation, not to restate the intended
//! API at a higher level.
//!
//! # Boot-Time Initialization
//!
//! The real firmware registers `WIFI0` during boot from
//! `firmware/badgevms/why2025_firmware.c` by calling `wifi_create()`. That
//! constructor:
//! - calls `flash_slave_c6_if_needed()` before any ESP-IDF Wi-Fi setup;
//! - allocates a `device_t`-backed driver object and exposes it as a block
//!   device;
//! - sets `status.status` to `WIFI_ENABLED` and `status.connection_status` to
//!   `WIFI_DISCONNECTED`;
//! - creates a shared FreeRTOS event group;
//! - calls `start_wifi()`, which runs `esp_netif_init()`, registers one event
//!   handler for all `WIFI_EVENT`s plus `IP_EVENT_STA_GOT_IP`, creates the
//!   default STA netif, calls `esp_wifi_init()`, forces `WIFI_MODE_STA`, and
//!   starts the ESP-IDF Wi-Fi driver;
//! - creates `status.mutex`, a command queue with capacity 5, and a kernel task
//!   named `Hermes` with stack size 4096 and priority 5.
//!
//! All public functions below assume that boot path has already happened.
//!
//! # Shared State
//!
//! The upstream driver keeps one static `status` struct containing:
//! - the public on/off state (`wifi_status_t`);
//! - the public connection state (`wifi_connection_status_t`);
//! - the most recently requested connection state (`connection_status_want`);
//! - a mutex;
//! - one cached snapshot of the currently connected access point;
//! - a fixed scan cache of 20 access points plus the current valid count; and
//! - the monotonic timestamp of the last scan request.
//!
//! The mutex is only used when copying the current-station snapshot and when
//! reading or writing the scan cache. The event handler updates
//! `connection_status` and `current` without taking that mutex, so the firmware
//! does not provide a fully synchronized cross-context snapshot.
//!
//! # Command Path
//!
//! `wifi_connect()`, `wifi_disconnect()`, and `wifi_scan_get_num_results()` all
//! serialize work through the `Hermes` task.
//!
//! `send_command()` allocates a queue message with `calloc()`, records the
//! calling task handle, pushes the message to the Hermes queue, and then blocks
//! forever in `ulTaskNotifyTakeIndexed(0, pdTRUE, portMAX_DELAY)`. Hermes
//! performs the requested operation and replies with
//! `xTaskNotifyIndexed(caller, 0, status.connection_status,
//! eSetValueWithOverwrite)`.
//!
//! Consequences of that design:
//! - callers must be normal FreeRTOS tasks; this path is not ISR-safe;
//! - reply traffic uses task-notification index 0 of the caller, so any other
//!   feature using that same notification slot can interfere with Wi-Fi calls;
//! - the return value of scan commands is ignored by
//!   `wifi_scan_get_num_results()`, but the task-notification side effect still
//!   happens.
//!
//! # Event Handling
//!
//! The registered event handler drives most connection state transitions:
//! - `WIFI_EVENT_STA_START` immediately calls `esp_wifi_connect()`.
//! - `WIFI_EVENT_STA_DISCONNECTED` logs the disconnect reason, sets
//!   `status.connection_status = WIFI_DISCONNECTED`, and then either:
//!   - if the user wanted to stay connected, converts
//!     `WIFI_REASON_ASSOC_NOT_AUTHED`, `WIFI_REASON_AUTH_FAIL`, and
//!     `WIFI_REASON_802_1X_AUTH_FAILED` into
//!     `WIFI_ERROR_WRONG_CREDENTIALS`, sets `WIFI_AUTH_ERR_BIT`, and returns; or
//!   - retries `esp_wifi_connect()` up to 10 times for all other reasons, then
//!     sets `WIFI_FAIL_BIT` and changes the public status to `WIFI_ERROR`; or
//!   - if the user explicitly requested a disconnect, sets
//!     `WIFI_DISCONNECTED_BIT`.
//! - `IP_EVENT_STA_GOT_IP` logs the IP address, resets the retry counter,
//!   changes the public status to `WIFI_CONNECTED`, refreshes the cached
//!   `status.current` snapshot with `esp_wifi_sta_get_ap_info()`, and sets
//!   `WIFI_CONNECTED_BIT`.
//!
//! # Important Upstream Quirks
//!
//! Several behaviors are worth preserving because in-tree applications already
//! observe them:
//! - `wifi_get_status()` effectively always reports `WIFI_ENABLED` after boot;
//!   the current upstream code never transitions it to `WIFI_DISABLED` or
//!   `WIFI_ASK`.
//! - `wifi_connect()` waits on the event group with `clearOnExit = pdFALSE` and
//!   `waitForAllBits = pdTRUE` for
//!   `WIFI_CONNECTED_BIT | WIFI_FAIL_BIT | WIFI_AUTH_ERR_BIT`. In normal
//!   success and wrong-credential cases only one of those bits becomes set, so
//!   the call usually sits until the full 5 second timeout before returning.
//!   Those bits also remain latched because connect never clears them.
//! - Because connect leaves `WIFI_CONNECTED_BIT` latched, a later
//!   `wifi_disconnect()` can consume that stale bit on its first wait and issue
//!   at least one extra `esp_wifi_disconnect()` retry before it sees a fresh
//!   `WIFI_DISCONNECTED_BIT`.
//! - `wifi_set_connection_parameters()` never calls `nvs_commit()`. It writes
//!   keys, closes the NVS handle, and returns success or failure based only on
//!   the immediate `nvs_set_str()` results. It also forgets to flip the return
//!   value to `false` if `nvs_open()` itself fails.
//! - `wifi_scan_get_result()` rejects indices `>= num_scan_results` but does not
//!   reject negative indices, so a negative `num` becomes an out-of-bounds read
//!   in upstream C.

use crate::types::*;

mod runtime;

use runtime::{
    get_mac_address_inner, wifi_connect_inner, wifi_disconnect_inner,
    wifi_get_connection_station_inner, wifi_get_connection_status_inner,
    wifi_scan_free_station_inner, wifi_scan_get_num_results_inner, wifi_scan_get_result_inner,
    wifi_set_connection_parameters_inner, wifi_station_get_bssid_inner,
    wifi_station_get_mode_inner, wifi_station_get_primary_channel_inner,
    wifi_station_get_rssi_inner, wifi_station_get_secondary_channel_inner,
    wifi_station_get_ssid_inner, wifi_station_wps_inner,
};

/// Returns the raw global Wi-Fi enable state.
///
/// In upstream this is a plain `return status.status;` with no locking,
/// computation, or side effects. `wifi_create()` initializes it to
/// `wifi_status_t::WIFI_ENABLED`, and no other code in `wifi.c` changes it.
/// That means the real firmware effectively reports `WIFI_ENABLED` for the
/// entire lifetime of the driver after boot.
#[unsafe(no_mangle)]
pub extern "C" fn wifi_get_status() -> wifi_status_t {
    wifi_status_t::WIFI_ENABLED
}
/// Returns the current global connection state.
///
/// This is also a raw field read with no locking. The value is driven by the
/// Hermes task and the ESP-IDF event handler:
/// - starts as `wifi_connection_status_t::WIFI_DISCONNECTED`;
/// - becomes `wifi_connection_status_t::WIFI_CONNECTED` on
///   `IP_EVENT_STA_GOT_IP`;
/// - becomes `wifi_connection_status_t::WIFI_DISCONNECTED` on any station
///   disconnect before any retry logic runs;
/// - becomes `wifi_connection_status_t::WIFI_ERROR_WRONG_CREDENTIALS` for the
///   three auth-related disconnect reasons explicitly checked upstream;
/// - becomes `wifi_connection_status_t::WIFI_ERROR` after the event handler
///   exhausts its 10 automatic reconnect attempts.
#[unsafe(no_mangle)]
pub extern "C" fn wifi_get_connection_status() -> wifi_connection_status_t {
    wifi_get_connection_status_inner()
}
/// Returns a heap-allocated snapshot of the currently connected access point.
///
/// Upstream behavior:
/// - locks `status.mutex`;
/// - only succeeds when `status.connection_status` is exactly
///   `wifi_connection_status_t::WIFI_CONNECTED`;
/// - allocates one `wifi_station_t` with `dlmalloc()`;
/// - copies the cached `status.current` snapshot into that allocation;
/// - unlocks the mutex and returns the new pointer.
///
/// The snapshot is not live. It is refreshed only when the Wi-Fi event handler
/// receives `IP_EVENT_STA_GOT_IP`, so callers observe the last AP info captured
/// at the moment the interface obtained an IP address.
///
/// Returns null if the driver is not currently connected or if `dlmalloc()`
/// fails. The caller is expected to release the returned handle with
/// `wifi_scan_free_station()`, even though the name of that free function only
/// mentions scan results.
#[unsafe(no_mangle)]
pub extern "C" fn wifi_get_connection_station() -> wifi_station_handle {
    wifi_get_connection_station_inner()
}
/// Performs a blocking connection attempt through the Hermes worker task.
///
/// Fast path:
/// - the public wrapper reads `status.connection_status` under `status.mutex`;
/// - if it is already `wifi_connection_status_t::WIFI_CONNECTED`, the function
///   logs `Already connected` and returns immediately without touching the queue
///   or ESP-IDF.
///
/// Slow path via Hermes:
/// - `connection_status_want` is set to `WIFI_CONNECTED`;
/// - if Hermes itself sees that the status is already connected, it logs and
///   returns without doing more work;
/// - otherwise it calls `esp_wifi_disconnect()` unconditionally to reset the
///   station state, then forces `WIFI_MODE_STA` again with
///   `esp_wifi_set_mode()`;
/// - it prepares credentials from NVS namespace `badgevms_wifi`, keys `ssid`
///   and `password`, using default values `"WHY2025-open"` and `""` if the
///   namespace cannot be opened;
/// - it ignores the return values of `nvs_get_str()`, so missing or oversized
///   keys silently leave the stack buffers at their current contents;
/// - it copies the resulting strings into `wifi_config_t.sta`, logging the SSID
///   and, for secured networks, the password in plaintext;
/// - it retries `esp_wifi_set_config()` up to 11 total attempts, delaying
///   `vTaskDelay(100)` between retries. That delay is 100 RTOS ticks, not an
///   explicitly millisecond-based duration.
///
/// After configuration Hermes calls `esp_wifi_connect()` and waits on the event
/// group for `WIFI_CONNECTED_BIT | WIFI_FAIL_BIT | WIFI_AUTH_ERR_BIT` with
/// `clearOnExit = pdFALSE`, `waitForAllBits = pdTRUE`, and a 5 second timeout.
/// That exact flag combination means:
/// - successful connects usually still wait out the full timeout before Hermes
///   continues, because only `WIFI_CONNECTED_BIT` is normally set;
/// - wrong-credential failures also usually wait for the timeout, then return
///   `wifi_connection_status_t::WIFI_ERROR_WRONG_CREDENTIALS` because the auth
///   error bit is set;
/// - the connect-side bits stay latched for later calls because Hermes does not
///   clear them on exit.
///
/// If Hermes does not see `WIFI_CONNECTED_BIT` in the returned bitmask, it
/// retries the whole connect wait loop up to 10 times. When that retry counter
/// expires it still falls through to the success log message (`Mount olympus
/// paged`) and simply returns the current `status.connection_status`, which can
/// therefore be `WIFI_DISCONNECTED`, `WIFI_ERROR`, or
/// `WIFI_ERROR_WRONG_CREDENTIALS`.
///
/// The public API blocks forever waiting for Hermes to reply, and that reply is
/// delivered through task-notification index 0 of the calling FreeRTOS task.
#[unsafe(no_mangle)]
pub extern "C" fn wifi_connect() -> wifi_connection_status_t {
    wifi_connect_inner()
}
/// Performs a blocking disconnect command through the Hermes worker task.
///
/// Public wrapper behavior:
/// - reads `status.connection_status` under `status.mutex`;
/// - if the state is already exactly
///   `wifi_connection_status_t::WIFI_DISCONNECTED`, returns immediately;
/// - otherwise queues a Hermes disconnect command and waits forever for a reply
///   on task-notification index 0.
///
/// Hermes behavior:
/// - sets `connection_status_want = WIFI_DISCONNECTED`;
/// - if the current public status is not exactly
///   `wifi_connection_status_t::WIFI_CONNECTED`, logs `Already disconnected`
///   and returns without changing `status.connection_status`. This means a
///   caller can receive `WIFI_ERROR` or `WIFI_ERROR_WRONG_CREDENTIALS` back
///   from `wifi_disconnect()`;
/// - if the status was connected, it eagerly changes the public state to
///   `wifi_connection_status_t::WIFI_DISCONNECTED` before calling
///   `esp_wifi_disconnect()`;
/// - it then waits for any of `WIFI_CONNECTED_BIT | WIFI_DISCONNECTED_BIT |
///   WIFI_FAIL_BIT` with `clearOnExit = pdTRUE`, `waitForAllBits = pdFALSE`,
///   and a 5 second timeout;
/// - if the wait result does not include `WIFI_DISCONNECTED_BIT`, Hermes retries
///   `esp_wifi_disconnect()` up to 5 more times.
///
/// Because the connect path leaves `WIFI_CONNECTED_BIT` latched, the first wait
/// in disconnect can complete immediately on that stale bit, clear it, and then
/// force an extra retry before a real disconnect event is seen.
#[unsafe(no_mangle)]
pub extern "C" fn wifi_disconnect() -> wifi_connection_status_t {
    wifi_disconnect_inner()
}
/// Frees a station handle previously returned by `wifi_get_connection_station()`
/// or `wifi_scan_get_result()`.
///
/// Upstream this is a single `dlfree(station);` call with no validation or
/// bookkeeping. The function name mentions scan results, but the settings UI
/// also uses it to free the handle returned by `wifi_get_connection_station()`.
#[unsafe(no_mangle)]
pub extern "C" fn wifi_scan_free_station(station: wifi_station_handle) {
    wifi_scan_free_station_inner(station)
}
/// Scans for access points if scanning is enabled, then returns the cached AP
/// count.
///
/// Public wrapper behavior:
/// - if `status.status != wifi_status_t::WIFI_DISABLED`, queues a Hermes scan
///   command;
/// - ignores the scan command's reply value;
/// - returns `status.num_scan_results` under `status.mutex`.
///
/// In the current upstream firmware `status.status` is set to `WIFI_ENABLED`
/// during `wifi_create()` and is never changed again, so this path always tries
/// to scan after boot.
///
/// Hermes scan behavior:
/// - rate-limits rescans using `clock_gettime(CLOCK_MONOTONIC, ...)` and the
///   cached `last_scan_time`;
/// - if the previous cached result count was non-zero, refuses to rescan for 10
///   seconds (`MIN_SCAN_INTERVAL`);
/// - if the previous cached result count was zero, refuses to rescan for 1
///   second (`MIN_SCAN_INTERVAL_EMPTY`);
/// - on a throttled call, simply returns without changing the cache, so callers
///   receive the old count;
/// - otherwise stores the new scan timestamp, runs
///   `esp_wifi_scan_start(NULL, true)` synchronously, fetches the AP count and
///   up to 20 `wifi_ap_record_t`s, truncates the visible result count to the
///   number of records actually copied, and rewrites the cached entries under
///   `status.mutex`.
///
/// The cache stores at most 20 results. Unused tail entries are not cleared;
/// only `status.num_scan_results` controls which entries are visible.
#[unsafe(no_mangle)]
pub extern "C" fn wifi_scan_get_num_results() -> ::core::ffi::c_int {
    wifi_scan_get_num_results_inner()
}
/// Returns a heap-allocated copy of one cached scan result.
///
/// This does not trigger a scan. It only reads `status.scan_results` under the
/// mutex and copies one cached entry with `dlmalloc()` plus `memcpy()`.
///
/// Exact upstream rules:
/// - if `num >= status.num_scan_results`, returns null;
/// - otherwise allocates and copies `status.scan_results[num]`;
/// - returns null on allocation failure.
///
/// Upstream does not reject negative `num` values before indexing the array, so
/// `num < 0` is an out-of-bounds read in the original C implementation.
///
/// The returned handle must be freed with `wifi_scan_free_station()`.
#[unsafe(no_mangle)]
pub extern "C" fn wifi_scan_get_result(num: ::core::ffi::c_int) -> wifi_station_handle {
    wifi_scan_get_result_inner(num)
}
/// Returns a borrowed pointer to the SSID buffer stored inside a station
/// snapshot.
///
/// Upstream simply returns `station->ssid` with no validation. The buffer is a
/// 33-byte inline array populated from `wifi_ap_record_t.ssid` (for scan
/// results) or from `esp_wifi_sta_get_ap_info()` via the cached current-station
/// snapshot. The returned pointer becomes invalid as soon as the owning station
/// handle is freed with `wifi_scan_free_station()`.
#[unsafe(no_mangle)]
pub extern "C" fn wifi_station_get_ssid(
    station: wifi_station_handle,
) -> *const ::core::ffi::c_char {
    wifi_station_get_ssid_inner(station)
}
/// Returns a borrowed pointer to the BSSID bytes stored inside a station
/// snapshot.
///
/// Upstream simply returns `station->bssid` with no validation. The backing C
/// struct oddly declares this field as `mac_address_t bssid[6]` but only copies
/// one MAC address worth of bytes into it and returns the first element as the
/// public BSSID pointer. Callers observe a normal 6-byte MAC address, but the
/// underlying storage layout is more awkward than the API suggests.
///
/// The returned pointer is into the station object itself and becomes invalid
/// once the handle is freed.
#[unsafe(no_mangle)]
pub extern "C" fn wifi_station_get_bssid(station: wifi_station_handle) -> *mut mac_address_t {
    wifi_station_get_bssid_inner(station)
}
/// Returns the primary channel recorded in the station snapshot.
///
/// This is a direct field read with no validation. For scan results it comes
/// from `wifi_ap_record_t.primary`; for the current connection snapshot it comes
/// from `esp_wifi_sta_get_ap_info()` at the moment `IP_EVENT_STA_GOT_IP` was
/// processed.
#[unsafe(no_mangle)]
pub extern "C" fn wifi_station_get_primary_channel(
    station: wifi_station_handle,
) -> ::core::ffi::c_int {
    wifi_station_get_primary_channel_inner(station)
}
/// Returns the secondary channel recorded in the station snapshot.
///
/// This is a direct field read with no validation. The upstream driver copies
/// the ESP-IDF `wifi_ap_record_t.second` field into `station->secondary`.
#[unsafe(no_mangle)]
pub extern "C" fn wifi_station_get_secondary_channel(
    station: wifi_station_handle,
) -> ::core::ffi::c_int {
    wifi_station_get_secondary_channel_inner(station)
}
/// Returns the raw RSSI stored in the station snapshot.
///
/// Upstream stores the `wifi_ap_record_t.rssi` byte in an `int8_t` field and
/// returns it as a C `int`, so the public value is the sign-extended RSSI in
/// dBm. The settings application currently turns this into a UI signal-strength
/// percentage by adding 150.
#[unsafe(no_mangle)]
pub extern "C" fn wifi_station_get_rssi(station: wifi_station_handle) -> ::core::ffi::c_int {
    wifi_station_get_rssi_inner(station)
}
/// Returns the authentication mode stored in the station snapshot.
///
/// Despite the name, this accessor does not return the PHY mode bitmask stored
/// internally in `station->mode`; it returns `station->authmode`.
///
/// That auth mode is populated by the upstream helper
/// `esp_authmode_to_badgevms()`. Notable lossy mappings include:
/// - `WIFI_AUTH_WPA3_EXT_PSK`, `WIFI_AUTH_WPA3_EXT_PSK_MIXED_MODE`, and
///   `WIFI_AUTH_WPA3_PSK` all collapse to
///   `wifi_auth_mode_t::WIFI_AUTH_WPA3_PSK`;
/// - `WIFI_AUTH_ENTERPRISE` becomes
///   `wifi_auth_mode_t::WIFI_AUTH_WPA2_ENTERPRISE`;
/// - `WIFI_AUTH_WPA_ENTERPRISE` becomes
///   `wifi_auth_mode_t::WIFI_AUTH_WPA2_WPA3_ENTERPRISE`;
/// - unknown values become `wifi_auth_mode_t::WIFI_AUTH_NONE`.
///
/// The settings application currently treats every returned mode other than
/// `WIFI_AUTH_OPEN` as a secured network.
#[unsafe(no_mangle)]
pub extern "C" fn wifi_station_get_mode(station: wifi_station_handle) -> wifi_auth_mode_t {
    wifi_station_get_mode_inner(station)
}
/// Returns the raw WPS-capable flag stored in the station snapshot.
///
/// Upstream copies this directly from `wifi_ap_record_t.wps` and exposes it as
/// a plain boolean with no validation.
#[unsafe(no_mangle)]
pub extern "C" fn wifi_station_wps(station: wifi_station_handle) -> bool {
    wifi_station_wps_inner(station)
}
/// Stores the credentials that a later `wifi_connect()` call will read back
/// from NVS.
///
/// Exact upstream behavior:
/// - truncates `ssid` to 31 bytes plus a trailing `\0` in a local 32-byte
///   buffer;
/// - truncates `password` to 63 bytes plus a trailing `\0` in a local 64-byte
///   buffer;
/// - opens NVS namespace `badgevms_wifi` in read-write mode;
/// - writes the truncated strings to keys `ssid` and `password` using
///   `nvs_set_str()`;
/// - logs an error and flips the return value to `false` if either individual
///   `nvs_set_str()` fails;
/// - closes the NVS handle.
///
/// Important quirks:
/// - upstream never calls `nvs_commit()`, so the code does not explicitly commit
///   the new values to flash before closing the handle;
/// - if `nvs_open()` itself fails, upstream logs `Unable to open NVS store` but
///   still returns `true` because it forgot to change the return flag in that
///   branch;
/// - there is no null-pointer validation before `strncpy()`.
///
/// This function does not reconnect the interface and does not modify the
/// in-memory `status.current` snapshot. Its effect is only consumed the next
/// time `wifi_connect()` re-reads NVS.
#[unsafe(no_mangle)]
pub extern "C" fn wifi_set_connection_parameters(
    ssid: *const ::core::ffi::c_char,
    password: *const ::core::ffi::c_char,
) -> bool {
    wifi_set_connection_parameters_inner(ssid, password)
}
/// Returns the base MAC address as an uppercase, colon-separated ASCII string.
///
/// This function is not implemented in `wifi.c`; upstream defines it in
/// `firmware/badgevms/wrapped_funcs.c`. Its behavior is nonetheless closely
/// tied to networking features because OTA and hardware-test code use it as the
/// device identifier.
///
/// Exact upstream behavior:
/// - uses one process-global static buffer of 18 bytes;
/// - if that buffer has already been populated once, returns it immediately
///   without touching hardware again;
/// - otherwise calls `esp_read_mac(..., ESP_MAC_BASE)` to read the base MAC,
///   not a per-interface MAC derived from it;
/// - on success, formats the bytes as `"%02X:%02X:%02X:%02X:%02X:%02X"`;
/// - on failure, logs an error, stores `"00:00:00:00:00:00"`, and returns that
///   fallback string.
///
/// Because the result is cached in the static buffer, a first-call failure is
/// sticky for the rest of the process: later calls keep returning the all-zero
/// string and do not retry `esp_read_mac()`.
#[unsafe(no_mangle)]
pub extern "C" fn get_mac_address() -> *const ::core::ffi::c_char {
    get_mac_address_inner()
}
