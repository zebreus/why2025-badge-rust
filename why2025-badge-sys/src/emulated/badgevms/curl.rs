//! Host-side weak stubs for BadgeVMS's curl compatibility layer.
//!
//! The upstream firmware implementation lives in `firmware/badgevms/curl.c` at commit
//! `a548d825a3295432d374939607feb552eb505210` (`Update espressif/eppp_link`).
//!
//! It is not libcurl. Upstream implements a small libcurl-shaped adapter around
//! `esp_http_client` plus a custom in-memory and tab-separated cookie jar.
//!
//! High-level upstream design:
//!
//! - `curl_easy_init()` allocates one persistent handle that owns duplicated option strings and an
//!   in-memory cookie list.
//! - `curl_easy_perform()` creates a fresh `esp_http_client_handle_t` from that saved config for
//!   each request and destroys it again before returning. Reusing a `CURL *` therefore preserves
//!   options and cookies, but not an already-open transport connection.
//! - response bodies go to stdout and response headers to stderr by default, even when
//!   `CURLOPT_VERBOSE` is left at `0`.
//! - many declared libcurl options and info selectors are missing, stubbed, or only partially
//!   implemented.
//!
//! Current upstream caller-visible quirks:
//!
//! - `sdk_apps/curl_test/capture_response_example()` returns `0` from its write callback on
//!   `realloc` failure, but the wrapper ignores callback return values, so that path does not abort
//!   the transfer and may still end as `CURLE_OK` if the transport succeeds.
//! - `sdk_apps/curl_test/upload_example()` sets `CURLOPT_CUSTOMREQUEST = "PUT"` together with
//!   `CURLOPT_POSTFIELDS`, but upstream only attaches the buffered request body when the method is
//!   exactly `POST`, so that example does not send the advertised payload.
//! - `sdk_apps/curl_test/cookie_visibility_example()` tries to observe outbound `Cookie:` headers
//!   through `CURLOPT_HEADERFUNCTION`, but the wrapper only invokes that callback for response
//!   headers delivered by `HTTP_EVENT_ON_HEADER`.
//! - `sdk_apps/curl_test/multiple_requests_example()` describes connection reuse, but the wrapper
//!   recreates and destroys `esp_http_client` on every `curl_easy_perform()` call; any reuse below
//!   that layer is outside the wrapper itself.
//! - `sdk_apps/hardware_test/run_tests.c` stores the boolean expression
//!   `(curl_easy_perform(curl) != CURLE_OK)` into a `CURLcode`, so any failure becomes `1` and is
//!   later stringified as `Unsupported protocol`. That bug is in the caller, not in the curl
//!   wrapper itself.
use crate::types::*;
use core::ffi::c_char;

const CURL_STUB_MESSAGE: &str =
    "Host-side BadgeVMS curl emulation is not implemented yet";

/// Allocate a new BadgeVMS curl handle.
///
/// # Exact upstream behavior
///
/// Upstream allocates `sizeof(curl_handle_t)` bytes with `dlcalloc(1, ...)` and returns the raw
/// pointer cast to `CURL *`. Allocation failure returns `NULL`.
///
/// The freshly allocated handle starts with these defaults:
///
/// - `write_function = default_write_callback`, which prints each response body chunk to stdout via
///   `printf("%.*s", len, data)`
/// - `header_function = default_header_callback`, which prints each response header chunk to stderr
///   via `why_fprintf(stderr, "%.*s", len, data)`
/// - `config.event_handler = http_event_handler`
/// - `config.user_data = curl`
/// - `config.timeout_ms = 30000`
/// - `config.crt_bundle_attach = esp_crt_bundle_attach`
/// - `proxy_type = CURLPROXY_HTTP`
/// - `proxy_port = 0`
/// - `proxy_auth = CURLAUTH_BASIC`
/// - `http_auth = CURLAUTH_BASIC`
/// - everything else zeroed by `dlcalloc`
///
/// Because `esp_http_client_config_t` is zero-initialized, later `esp_http_client_init()` sees a
/// default HTTP method of `GET` (`HTTP_METHOD_GET = 0` in ESP-IDF), automatic redirect handling is
/// left enabled (`disable_auto_redirect = false`), and both RX/TX buffer sizes are left at `0`
/// until `esp_http_client` replaces them with its own default of 512 bytes.
///
/// # Hidden state and dead fields
///
/// Upstream stores more state than the public header suggests. Notably:
///
/// - `response_code` and `content_length` start at `0` and are only updated on
///   `HTTP_EVENT_ON_FINISH`
/// - `content_type` is allocated and freed as part of the handle lifetime, but is never populated
/// - `effective_url` is likewise never populated
/// - `configured` is declared in the struct but never read or written after zero-initialization
/// - `ssl_verify_peer` is updated by `CURLOPT_SSL_VERIFYPEER`, but later request behavior is driven
///   by the embedded `esp_http_client_config_t` fields instead
///
/// The handle owns duplicated option strings and the in-memory cookie list across multiple request
/// executions. The underlying `esp_http_client_handle_t` is not persistent; `curl_easy_perform()`
/// creates and destroys that object on every call.
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn curl_easy_init() -> *mut CURL {
    unimplemented!("{CURL_STUB_MESSAGE}")
}

/// Apply one option to a BadgeVMS curl handle.
///
/// # Exact upstream behavior
///
/// `NULL` handles return `CURLE_FAILED_INIT`. Otherwise upstream opens a C varargs list and mutates
/// the in-memory handle state immediately.
///
/// Supported options and their exact semantics:
///
/// - `CURLOPT_URL`: frees the previous `config.url` and stores `why_strdup(url)`; no validation is
///   performed here
/// - `CURLOPT_USERAGENT`: frees the previous `config.user_agent` and stores `why_strdup(agent)`
/// - `CURLOPT_TIMEOUT`: reads a `long` in seconds and stores `config.timeout_ms = timeout * 1000`
/// - `CURLOPT_TIMEOUT_MS`: reads a `long` and stores it directly in `config.timeout_ms`
/// - `CURLOPT_SSL_VERIFYPEER`: toggles `ssl_verify_peer`, reattaches the ESP certificate bundle
///   when nonzero, and when zero both detaches the bundle and sets
///   `skip_cert_common_name_check = true`
/// - `CURLOPT_SSL_VERIFYHOST`: treats any nonzero value as "verify hostname" and `0` as "skip
///   hostname verification" by directly flipping `skip_cert_common_name_check`
/// - `CURLOPT_CAINFO`: frees the previous `config.cert_pem`, stores `why_strdup(ca_file)`, and
///   disables the bundled root store by setting `crt_bundle_attach = NULL`; despite the libcurl
///   name, ESP-IDF expects inline certificate PEM here, not a filesystem path
/// - `CURLOPT_USERPWD`: duplicates the whole `user:pass` string, splits on the first `:`, and if a
///   colon exists replaces `config.username` and `config.password` with new duplicates of both
///   halves; if no colon exists, the temporary copy is just freed and the previously configured
///   credentials remain unchanged
/// - `CURLOPT_POSTFIELDS`: frees `post_data`, duplicates the provided C string with `why_strdup`,
///   and sets `post_data_size = strlen(data)`
/// - `CURLOPT_POSTFIELDSIZE`: overwrites `post_data_size` with the caller-provided `long`
/// - `CURLOPT_HTTPHEADER`: stores the caller-owned `struct curl_slist *` pointer without cloning it
/// - `CURLOPT_WRITEFUNCTION`, `CURLOPT_WRITEDATA`, `CURLOPT_HEADERFUNCTION`, and
///   `CURLOPT_HEADERDATA`: simply store the function or data pointer
/// - `CURLOPT_FOLLOWLOCATION`: sets `config.max_redirection_count` to `10` when nonzero and `0`
///   when zero
/// - `CURLOPT_MAXREDIRS`: stores the caller-provided maximum in `config.max_redirection_count`
/// - `CURLOPT_VERBOSE`: only toggles the wrapper's own `verbose` flag for `ESP_LOGI` event prints;
///   it does not control the default stdout/stderr data callbacks
/// - `CURLOPT_CUSTOMREQUEST`: maps only the exact uppercase strings `GET`, `POST`, `PUT`,
///   `DELETE`, `HEAD`, and `PATCH` to ESP-IDF method enums; any other string still returns
///   `CURLE_OK` but leaves the method at `GET`
/// - `CURLOPT_POST`, `CURLOPT_PUT`, `CURLOPT_HTTPGET`, and `CURLOPT_NOBODY`: when passed a
///   nonzero `long`, set the method to `POST`, `PUT`, `GET`, or `HEAD` respectively; passing `0`
///   does nothing and does not revert earlier method changes
/// - `CURLOPT_COOKIE`: frees the previous manual cookie string and stores `why_strdup(cookies)`
/// - `CURLOPT_COOKIEFILE`: frees the previous cookie filename, stores `why_strdup(filename)`, and
///   immediately calls `load_cookies_from_file(curl, filename)`; missing or unreadable files only
///   log a warning and count as `0` loaded cookies, and the existing in-memory cookie list is not
///   cleared before loading
/// - `CURLOPT_COOKIEJAR`: frees the previous jar filename and stores `why_strdup(filename)`; the
///   actual save happens later in `curl_easy_perform()`
/// - `CURLOPT_BUFFERSIZE`: stores the caller's `long` in `config.buffer_size`
/// - `CURLOPT_HTTPAUTH`: stores the caller's `long` in the handle's `http_auth` field and returns
///   success, but that field is never propagated into `esp_http_client_config_t`
///
/// Unsupported or misleading options:
///
/// - `CURLOPT_PROXY`, `CURLOPT_PROXYUSERPWD`, `CURLOPT_PROXYTYPE`, `CURLOPT_PROXYPORT`, and
///   `CURLOPT_PROXYAUTH` all store their input in the handle, log a warning, and then return
///   `CURLE_UNSUPPORTED_PROTOCOL`; the stored proxy state is never consulted later
/// - `CURLOPT_RANGE` and `CURLOPT_REFERER` log a warning and return
///   `CURLE_UNSUPPORTED_PROTOCOL`
/// - `CURLOPT_CAPATH` is declared in the header but has no `switch` case, so it falls through to
///   the generic unsupported-option warning
/// - `CURLOPT_CONNECTTIMEOUT` is declared with the same numeric value as `CURLOPT_TIMEOUT`, so it
///   hits the `CURLOPT_TIMEOUT` case and changes the total request timeout rather than a
///   connect-only timeout
/// - `CURLOPT_CONNECTTIMEOUT_MS` is declared in the header but has no implementation case, so it
///   returns `CURLE_UNSUPPORTED_PROTOCOL`
/// - `CURLOPT_FOLLOWLOCATION = 0` does not actually disable redirects: upstream never sets
///   `disable_auto_redirect`, and ESP-IDF treats `max_redirection_count == 0` as "use the default
///   redirect limit"
/// - `CURLOPT_MAXREDIRS = 0` has the same ESP-IDF interaction and therefore also means "default
///   redirect limit", not "no redirects"
/// - `CURLOPT_HTTPAUTH` is effectively dead state; real HTTP auth in this wrapper depends on
///   `CURLOPT_USERPWD` plus `esp_http_client`'s own 401 challenge handling
///
/// Bugs and edge cases in current upstream:
///
/// - pointer arguments are almost never null-checked before being passed to `why_strdup`, `strlen`,
///   `strcmp`, or `strchr`
/// - most `why_strdup` allocation failures are ignored, so the function can return `CURLE_OK` even
///   though the option became `NULL` or only partially updated
/// - `CURLOPT_USERPWD` without a `:` silently keeps the previous username/password instead of
///   clearing them or returning an error
/// - `CURLOPT_POSTFIELDS` is binary-unsafe because it always copies via `why_strdup` and sizes via
///   `strlen`; embedded NUL bytes are truncated immediately
/// - if the caller later sets `CURLOPT_POSTFIELDSIZE` to a value larger than the copied string
///   length, `curl_easy_perform()` will hand `esp_http_client` a pointer to the short allocation
///   plus the oversized length, which can make ESP-IDF read past the allocation
/// - setting `CURLOPT_POSTFIELDSIZE` before `CURLOPT_POSTFIELDS` is order-sensitive because the
///   later `CURLOPT_POSTFIELDS` call overwrites the size with `strlen(data)`
/// - request bodies are only attached later when the method is exactly `POST`; the wrapper does not
///   attach `post_data` for `PUT`, `PATCH`, or arbitrary `CURLOPT_CUSTOMREQUEST` verbs
/// - `CURLOPT_CUSTOMREQUEST` resets `config.method` to `GET` before comparing strings, so an
///   unrecognized or mixed-case token silently overwrites any earlier `POST`/`PUT`/`HEAD`
///   selection with `GET`
/// - repeated `CURLOPT_COOKIEFILE` calls merge new entries into the existing cookie list rather than
///   replacing it, with duplicate replacement still keyed only by cookie name
/// - the `CURLOPT_BUFFERSIZE` case calls `va_end(args)` inside the `switch` and then the function
///   calls `va_end(args)` again after the `switch`, which is undefined behavior
///
/// # Interaction with ESP-IDF authentication
///
/// `CURLOPT_USERPWD` can still make Basic or Digest auth work in practice, but not because
/// `CURLOPT_HTTPAUTH` is honored. The underlying `esp_http_client` inspects `WWW-Authenticate`
/// headers on `401 Unauthorized`, switches its own internal auth type to Basic or Digest, and then
/// retries using the stored username/password. The BadgeVMS wrapper's `http_auth` field is not part
/// of that path.
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub unsafe extern "C" fn curl_easy_setopt(
    _curl: *mut CURL,
    _option: CURLoption,
    mut _args: ...,
) -> CURLcode {
    unimplemented!("{CURL_STUB_MESSAGE}")
}

/// Execute one HTTP request using the options currently stored on the handle.
///
/// # Exact upstream behavior
///
/// `NULL` handles return `CURLE_FAILED_INIT`.
///
/// Otherwise upstream performs the request in this order:
///
/// - leave any prior `response_code` and `content_length` values untouched at request start
/// - call `esp_http_client_init(&curl->config)` and store the returned handle in `curl->esp_client`
/// - if that returns `NULL`, return `CURLE_FAILED_INIT`
/// - if `curl->headers` is non-`NULL`, iterate the caller-owned linked list and for each header
///   string containing a `:`, split a temporary duplicate at the first colon, trim leading spaces or
///   tabs from the value, and call `esp_http_client_set_header()`
/// - build a `Cookie` header by concatenating `manual_cookies` first and then every in-memory
///   cookie as `name=value`, separated by `; `, and install that header when nonempty
/// - if `post_data` is non-`NULL` and the method is exactly `HTTP_METHOD_POST`, call
///   `esp_http_client_set_post_field(curl->esp_client, curl->post_data, curl->post_data_size)`
/// - call `esp_http_client_perform(curl->esp_client)`
/// - if `cookie_jar` is non-`NULL`, call `save_cookies_to_file(curl, curl->cookie_jar)` even when
///   the request itself failed; the save result is ignored
/// - call `esp_http_client_cleanup(curl->esp_client)` and set `curl->esp_client = NULL`
/// - map the final `esp_err_t` to a `CURLcode`
///
/// The error mapping is deliberately small:
///
/// - `ESP_OK -> CURLE_OK`
/// - `ESP_ERR_TIMEOUT -> CURLE_OPERATION_TIMEDOUT`
/// - `ESP_ERR_HTTP_CONNECT -> CURLE_COULDNT_CONNECT`
/// - every other nonzero `esp_err_t -> CURLE_HTTP_RETURNED_ERROR`
///
/// That means HTTP status codes like `404` or `500` do not change the return code when the
/// transport itself succeeded. In-tree callers that care about HTTP-level failure, such as the OTA
/// helpers, call `curl_easy_getinfo(..., CURLINFO_RESPONSE_CODE, ...)` afterward and interpret the
/// status code themselves.
///
/// # Callback behavior
///
/// Upstream wires `http_event_handler()` into `esp_http_client`. That handler behaves as follows:
///
/// - `HTTP_EVENT_ON_DATA` forwards the raw chunk to `write_function(evt->data, 1, evt->data_len,
///   write_data)` if a callback is present
/// - `HTTP_EVENT_ON_HEADER` first parses `Set-Cookie` headers into the in-memory cookie list, then
///   forwards response headers to `header_function`
/// - `HTTP_EVENT_ON_FINISH` snapshots `response_code` and `content_length` from the live
///   `esp_http_client`
/// - `HTTP_EVENT_ERROR`, `HTTP_EVENT_ON_CONNECTED`, `HTTP_EVENT_HEADER_SENT`,
///   `HTTP_EVENT_DISCONNECTED`, and `HTTP_EVENT_REDIRECT` only emit `ESP_LOGI` output when
///   `verbose` is true
///
/// The header callback receives a temporary heap buffer containing `"Key: Value"` with:
///
/// - no terminating NUL byte
/// - no trailing `\r\n`
/// - `size = 1` and `nmemb = key_len + value_len + 2`
///
/// Upstream ignores both callback return values completely. Returning `0`, a short count, or any
/// other error indicator does not abort the request and can never produce `CURLE_WRITE_ERROR` or
/// `CURLE_ABORTED_BY_CALLBACK`.
///
/// The default callbacks installed by `curl_easy_init()` are active even with `CURLOPT_VERBOSE = 0`:
///
/// - the default write callback prints each body chunk to stdout using `%.*s`, which is text- and
///   NUL-sensitive rather than binary-safe
/// - the default header callback prints each response header chunk to stderr using `%.*s`
/// - both defaults ignore their userdata pointer, so the `NULL` `write_data`/`header_data` values
///   set by `curl_easy_init()` are harmless until the caller overrides them
///
/// # Cookie subsystem details and bugs
///
/// BadgeVMS implements its own cookie parser and jar rather than reusing libcurl's cookie engine.
/// Exact behavior:
///
/// - every `Set-Cookie` header is logged with `ESP_LOGW(TAG, "Parsing header '%s'", header)` even
///   when verbose logging is off
/// - cookies default to `domain = "example.com"`, `path = "/"`, `expires = 0`, `secure = false`,
///   `http_only = false`, and `samesite = CURL_SAMESITE_NONE`
/// - cookie names, values, and attribute key/value pairs are trimmed for leading/trailing spaces or
///   tabs before storage
/// - `Domain`, `Path`, `Expires`, `Max-Age`, `Secure`, `HttpOnly`, and `SameSite` attributes are
///   parsed; unknown attributes are silently ignored
/// - `Expires=` is not really parsed as RFC 1123 time; any present `Expires=` attribute simply sets
///   `expires = 2147483647`
/// - invalid `SameSite=` values log a warning and fall back to `CURL_SAMESITE_NONE`
/// - `Max-Age=0` marks the cookie as expired by setting `expires = 1`, but the cookie is still kept
///   in memory and is still sent on subsequent requests from the same handle
/// - duplicate replacement is keyed only by cookie name, ignoring domain and path, so distinct
///   same-name cookies cannot coexist
/// - when sending cookies, upstream ignores domain, path, expiry, `Secure`, `HttpOnly`, and
///   `SameSite`; every stored cookie is always appended to the outgoing `Cookie` header, and expiry
///   is only consulted when loading from or saving to a cookie file
/// - `CURLOPT_COOKIEFILE` and `CURLOPT_COOKIEJAR` use a custom tab-separated text format, not the
///   Netscape/libcurl cookie jar format; the writer truncates the target file, emits two comment
///   lines, then writes `name\tvalue\tdomain\tpath\texpires\tsecure\thttp_only\tsamesite\t\n`
///   records
/// - loading skips blank lines, `#` comments, malformed lines, and cookies already expired at load
///   time; a missing file only logs a warning and is treated as success with zero cookies loaded
/// - file round-tripping collapses `SameSite=LAX` and `SameSite=STRICT`: the writer serializes
///   `cookie->samesite ? 1 : 0`, and the loader interprets any nonzero field as enum value `1`
///   (`LAX`)
///
/// # State persistence across requests
///
/// Reusing the same `CURL *` across multiple `curl_easy_perform()` calls preserves:
///
/// - duplicated option strings
/// - the in-memory cookie list
/// - the borrowed header-list pointer
/// - the last `response_code` and `content_length` values until a later finished request overwrites
///   them
///
/// It does not preserve an already-open ESP-IDF client or a live network connection, because the
/// wrapper tears `esp_http_client` down at the end of every request.
///
/// # Additional upstream bugs
///
/// - header-list entries with no colon are silently skipped
/// - `curl_slist_append()` can create a node with `data = NULL`; `curl_easy_perform()` later passes
///   that pointer to `strchr()`, which will crash
/// - allocation failures for duplicated request headers or temporary response-header buffers are not
///   checked before `strchr()` or `memcpy()`, so low-memory paths can crash
/// - `load_cookies_from_file()` does not validate any of the per-field `why_strdup()` results before
///   inserting the cookie, so allocation failure can leave partially initialized cookie nodes that
///   later crash comparisons or `build_cookie_header()`
/// - `response_code` and `content_length` are not reset at the start of a request, so after a
///   failed request they can still report stale values from a previous successful request
/// - `content_type` and `effective_url` are never updated anywhere, so redirects do not populate a
///   final effective URL and content type is always absent
/// - `CURLOPT_FOLLOWLOCATION = 0` still leaves ESP-IDF auto-redirect handling active because
///   `disable_auto_redirect` is never set
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn curl_easy_perform(_curl: *mut CURL) -> CURLcode {
    unimplemented!("{CURL_STUB_MESSAGE}")
}

/// Destroy a BadgeVMS curl handle and free all state owned by it.
///
/// # Exact upstream behavior
///
/// `NULL` is accepted as a no-op.
///
/// Otherwise upstream frees, in order:
///
/// - duplicated strings in `config.url`, `config.user_agent`, `config.username`, `config.password`,
///   and `config.cert_pem`
/// - `post_data`
/// - `content_type`
/// - `effective_url`
/// - the entire in-memory cookie list via `free_all_cookies()`
/// - `cookie_file`, `cookie_jar`, and `manual_cookies`
/// - `proxy_url` and `proxy_userpwd`
/// - the live `esp_client`, if one still exists
/// - the handle allocation itself
///
/// # Ownership boundaries and omissions
///
/// Upstream does not free:
///
/// - the caller-owned `struct curl_slist *headers`
/// - any callback userdata pointers supplied through `CURLOPT_WRITEDATA` or `CURLOPT_HEADERDATA`
/// - any buffers the caller allocated for response capture
///
/// `curl_easy_cleanup()` also does not flush cookies to `cookie_jar`; cookie-jar writes only happen
/// inside `curl_easy_perform()` after each request.
///
/// Because `content_type` and `effective_url` are never populated in current upstream, their frees
/// are usually just `dlfree(NULL)`.
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn curl_easy_cleanup(_curl: *mut CURL) {
    unimplemented!("{CURL_STUB_MESSAGE}")
}

/// Query one piece of cached metadata from the handle.
///
/// # Exact upstream behavior
///
/// `NULL` handles return `CURLE_FAILED_INIT`. Otherwise upstream switches on the requested info code
/// and writes directly through the caller's varargs out-pointer without validating that pointer.
///
/// Supported selectors:
///
/// - `CURLINFO_RESPONSE_CODE`: expects `long *` and writes `curl->response_code`
/// - `CURLINFO_CONTENT_LENGTH_DOWNLOAD`: expects `double *` and writes
///   `(double)curl->content_length`
/// - `CURLINFO_CONTENT_TYPE`: expects `char **` and writes `curl->content_type`
/// - `CURLINFO_EFFECTIVE_URL`: expects `char **` and writes `curl->effective_url` when non-`NULL`,
///   otherwise the current configured URL pointer `curl->config.url`
///
/// Unsupported selectors return `CURLE_UNSUPPORTED_PROTOCOL`.
///
/// # Important limitations and bugs
///
/// - `response_code` and `content_length` are only updated on `HTTP_EVENT_ON_FINISH`; if the most
///   recent request failed before that event, these values can still be stale from an earlier
///   success
/// - `content_length` comes from `esp_http_client_get_content_length()` and is then cast to
///   `double`; unknown lengths therefore surface as ESP-IDF's sentinel value, commonly `-1`
/// - `content_type` is never populated anywhere in `firmware/badgevms/curl.c`, so this selector is
///   effectively always `NULL`
/// - `effective_url` is also never populated, so `CURLINFO_EFFECTIVE_URL` reports the currently
///   configured URL, not the final URL after redirects
/// - the returned string pointers are borrowed from the handle; callers must not free them
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub unsafe extern "C" fn curl_easy_getinfo(
    _curl: *mut CURL,
    _info: curl_easy_info_t,
    mut _args: ...,
) -> CURLcode {
    unimplemented!("{CURL_STUB_MESSAGE}")
}

/// Convert a `CURLcode` to one of BadgeVMS's static error strings.
///
/// # Exact upstream behavior
///
/// Upstream returns pointers to string literals for only this subset of codes:
///
/// - `CURLE_OK -> "No error"`
/// - `CURLE_UNSUPPORTED_PROTOCOL -> "Unsupported protocol"`
/// - `CURLE_FAILED_INIT -> "Failed initialization"`
/// - `CURLE_URL_MALFORMAT -> "URL malformat"`
/// - `CURLE_COULDNT_RESOLVE_HOST -> "Couldn't resolve host"`
/// - `CURLE_COULDNT_CONNECT -> "Couldn't connect"`
/// - `CURLE_HTTP_RETURNED_ERROR -> "HTTP returned error"`
/// - `CURLE_OPERATION_TIMEDOUT -> "Operation timed out"`
/// - `CURLE_SSL_CONNECT_ERROR -> "SSL connect error"`
/// - anything else -> `"Unknown error"`
///
/// # Implications
///
/// - the table is much smaller than libcurl's real `curl_easy_strerror()` table
/// - several enum values exported in the public header, such as `CURLE_COULDNT_RESOLVE_PROXY`,
///   stringify as `"Unknown error"`
/// - the returned pointer is static storage and must not be freed
/// - current upstream call sites treat this as total coverage, but it is not
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn curl_easy_strerror(_error: CURLcode) -> *const c_char {
    unimplemented!("{CURL_STUB_MESSAGE}")
}

/// Append one string node to a `curl_slist` chain.
///
/// # Exact upstream behavior
///
/// Upstream allocates a new node with `dlmalloc(sizeof(struct curl_slist))`, duplicates the input
/// string with `why_strdup(string)`, sets `next = NULL`, and appends the node to the tail of the
/// existing list.
///
/// Return behavior:
///
/// - if node allocation fails, upstream returns the original `list` unchanged
/// - if `list == NULL`, the new node becomes the new head and is returned
/// - otherwise the original head pointer is returned after an O(n) tail walk
///
/// # Bugs and omissions
///
/// - the result of `why_strdup(string)` is not checked, so upstream can return a node whose
///   `data == NULL`
/// - later `curl_easy_perform()` assumes every node's `data` is non-`NULL` and passes it to
///   `strchr()`, so a duplication failure here can become a crash later
/// - the function does not validate `string`; passing `NULL` relies entirely on `why_strdup(NULL)`
///   behavior
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn curl_slist_append(
    _list: *mut curl_slist,
    _string: *const c_char,
) -> *mut curl_slist {
    unimplemented!("{CURL_STUB_MESSAGE}")
}

/// Free an entire `curl_slist` chain.
///
/// # Exact upstream behavior
///
/// Upstream walks the list iteratively and, for each node:
///
/// - saves `next`
/// - frees `list->data` with `dlfree`
/// - frees the node itself with `dlfree`
/// - continues with the saved `next`
///
/// Passing `NULL` is a no-op.
///
/// This only frees the linked list allocated by `curl_slist_append()`. It does not update any curl
/// handle that still points at that list through `CURLOPT_HTTPHEADER`; handles merely borrow the
/// list pointer.
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn curl_slist_free_all(_list: *mut curl_slist) {
    unimplemented!("{CURL_STUB_MESSAGE}")
}

/// Perform libcurl-compatible global initialization.
///
/// # Exact upstream behavior
///
/// This function ignores `flags` completely and immediately returns `CURLE_OK`.
///
/// No global state is initialized, no reference counting is maintained, and request behavior does
/// not depend on calling this first. In-tree firmware apps still call it for libcurl API
/// compatibility.
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn curl_global_init(_flags: ::core::ffi::c_long) -> CURLcode {
    unimplemented!("{CURL_STUB_MESSAGE}")
}

/// Perform libcurl-compatible global teardown.
///
/// # Exact upstream behavior
///
/// This is a complete no-op in current upstream firmware.
///
/// It does not free shared resources, flush cookie jars, or invalidate handles created by
/// `curl_easy_init()`.
#[unsafe(no_mangle)]
#[linkage = "weak"]
pub extern "C" fn curl_global_cleanup() {
    unimplemented!("{CURL_STUB_MESSAGE}")
}