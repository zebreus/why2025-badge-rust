use crate::types::*;
use core::ffi::{CStr, VaList, c_char, c_long, c_void};
use reqwest::{
    blocking::ClientBuilder,
    header::{COOKIE, HeaderMap, HeaderName, HeaderValue, USER_AGENT},
    redirect::Policy,
    Certificate, Method,
};
use std::{
    ffi::CString,
    fs::File,
    io::{self, BufRead, BufReader, Read, Write},
    mem, ptr, slice,
    time::{SystemTime, UNIX_EPOCH},
};

type CurlWriteCallback = unsafe extern "C" fn(*mut c_void, usize, usize, *mut c_void) -> usize;
type CurlHeaderCallback = unsafe extern "C" fn(*mut c_void, usize, usize, *mut c_void) -> usize;

const CURLE_OK_CODE: CURLcode = 0;
const CURLE_UNSUPPORTED_PROTOCOL_CODE: CURLcode = 1;
const CURLE_FAILED_INIT_CODE: CURLcode = 2;
const CURLE_URL_MALFORMAT_CODE: CURLcode = 3;
const CURLE_COULDNT_RESOLVE_HOST_CODE: CURLcode = 5;
const CURLE_COULDNT_CONNECT_CODE: CURLcode = 6;
const CURLE_HTTP_RETURNED_ERROR_CODE: CURLcode = 8;
const CURLE_OPERATION_TIMEDOUT_CODE: CURLcode = 11;
const CURLE_SSL_CONNECT_ERROR_CODE: CURLcode = 12;

const CURLAUTH_BASIC_CODE: c_long = 1;
const CURLPROXY_HTTP_CODE: c_long = 0;

const CURLOPT_WRITEDATA_OPTION: CURLoption = 10001;
const CURLOPT_URL_OPTION: CURLoption = 10002;
const CURLOPT_PROXY_OPTION: CURLoption = 10004;
const CURLOPT_USERPWD_OPTION: CURLoption = 10005;
const CURLOPT_PROXYUSERPWD_OPTION: CURLoption = 10006;
const CURLOPT_RANGE_OPTION: CURLoption = 10007;
const CURLOPT_POSTFIELDS_OPTION: CURLoption = 10015;
const CURLOPT_REFERER_OPTION: CURLoption = 10016;
const CURLOPT_USERAGENT_OPTION: CURLoption = 10018;
const CURLOPT_COOKIE_OPTION: CURLoption = 10022;
const CURLOPT_HTTPHEADER_OPTION: CURLoption = 10023;
const CURLOPT_HEADERDATA_OPTION: CURLoption = 10029;
const CURLOPT_COOKIEFILE_OPTION: CURLoption = 10031;
const CURLOPT_CUSTOMREQUEST_OPTION: CURLoption = 10036;
const CURLOPT_VERBOSE_OPTION: CURLoption = 41;
const CURLOPT_NOBODY_OPTION: CURLoption = 44;
const CURLOPT_POST_OPTION: CURLoption = 47;
const CURLOPT_FOLLOWLOCATION_OPTION: CURLoption = 52;
const CURLOPT_PUT_OPTION: CURLoption = 54;
const CURLOPT_PROXYPORT_OPTION: CURLoption = 59;
const CURLOPT_POSTFIELDSIZE_OPTION: CURLoption = 60;
const CURLOPT_SSL_VERIFYPEER_OPTION: CURLoption = 64;
const CURLOPT_MAXREDIRS_OPTION: CURLoption = 68;
const CURLOPT_TIMEOUT_OPTION: CURLoption = 78;
const CURLOPT_HTTPGET_OPTION: CURLoption = 80;
const CURLOPT_SSL_VERIFYHOST_OPTION: CURLoption = 81;
const CURLOPT_COOKIEJAR_OPTION: CURLoption = 10082;
const CURLOPT_BUFFERSIZE_OPTION: CURLoption = 98;
const CURLOPT_PROXYTYPE_OPTION: CURLoption = 101;
const CURLOPT_HTTPAUTH_OPTION: CURLoption = 107;
const CURLOPT_PROXYAUTH_OPTION: CURLoption = 111;
const CURLOPT_WRITEFUNCTION_OPTION: CURLoption = 20011;
const CURLOPT_TIMEOUT_MS_OPTION: CURLoption = 155;
const CURLOPT_CONNECTTIMEOUT_MS_OPTION: CURLoption = 156;
const CURLOPT_CAINFO_OPTION: CURLoption = 10065;
const CURLOPT_CAPATH_OPTION: CURLoption = 10097;
const CURLOPT_HEADERFUNCTION_OPTION: CURLoption = 20079;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CurlMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Patch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CurlSameSite {
    None,
    Lax,
    Strict,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CookieEntry {
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) domain: String,
    pub(crate) path: String,
    pub(crate) expires: i64,
    pub(crate) secure: bool,
    pub(crate) http_only: bool,
    pub(crate) samesite: CurlSameSite,
}

pub(crate) struct CurlHandle {
    pub(crate) url: Option<CString>,
    pub(crate) user_agent: Option<CString>,
    pub(crate) timeout_ms: i64,
    pub(crate) cert_pem: Option<CString>,
    pub(crate) username: Option<CString>,
    pub(crate) password: Option<CString>,
    pub(crate) post_data: Option<CString>,
    pub(crate) post_data_size: usize,
    pub(crate) headers: *mut curl_slist,
    pub(crate) cookies: Vec<CookieEntry>,
    pub(crate) cookie_file: Option<CString>,
    pub(crate) cookie_jar: Option<CString>,
    pub(crate) manual_cookies: Option<CString>,
    pub(crate) proxy_url: Option<CString>,
    pub(crate) proxy_userpwd: Option<CString>,
    pub(crate) proxy_type: c_long,
    pub(crate) proxy_port: c_long,
    pub(crate) proxy_auth: c_long,
    pub(crate) response_code: i32,
    pub(crate) content_length: i64,
    pub(crate) content_type: Option<CString>,
    pub(crate) effective_url: Option<CString>,
    #[allow(dead_code)]
    pub(crate) configured: bool,
    pub(crate) verbose: bool,
    pub(crate) ssl_verify_peer: bool,
    pub(crate) verify_host: bool,
    pub(crate) http_auth: c_long,
    pub(crate) buffer_size: usize,
    pub(crate) method: CurlMethod,
    pub(crate) max_redirection_count: c_long,
    pub(crate) write_function: Option<CurlWriteCallback>,
    pub(crate) write_data: *mut c_void,
    pub(crate) header_function: Option<CurlHeaderCallback>,
    pub(crate) header_data: *mut c_void,
}

impl CurlHandle {
    fn new() -> Self {
        Self {
            url: None,
            user_agent: None,
            timeout_ms: 30_000,
            cert_pem: None,
            username: None,
            password: None,
            post_data: None,
            post_data_size: 0,
            headers: ptr::null_mut(),
            cookies: Vec::new(),
            cookie_file: None,
            cookie_jar: None,
            manual_cookies: None,
            proxy_url: None,
            proxy_userpwd: None,
            proxy_type: CURLPROXY_HTTP_CODE,
            proxy_port: 0,
            proxy_auth: CURLAUTH_BASIC_CODE,
            response_code: 0,
            content_length: 0,
            content_type: None,
            effective_url: None,
            configured: false,
            verbose: false,
            ssl_verify_peer: true,
            verify_host: true,
            http_auth: CURLAUTH_BASIC_CODE,
            buffer_size: 0,
            method: CurlMethod::Get,
            max_redirection_count: 0,
            write_function: Some(default_write_callback),
            write_data: ptr::null_mut(),
            header_function: Some(default_header_callback),
            header_data: ptr::null_mut(),
        }
    }
}

unsafe extern "C" fn default_write_callback(
    contents: *mut c_void,
    size: usize,
    nmemb: usize,
    _userp: *mut c_void,
) -> usize {
    write_default_stream(contents, size, nmemb, true)
}

unsafe extern "C" fn default_header_callback(
    contents: *mut c_void,
    size: usize,
    nmemb: usize,
    _userp: *mut c_void,
) -> usize {
    write_default_stream(contents, size, nmemb, false)
}

fn write_default_stream(contents: *mut c_void, size: usize, nmemb: usize, stdout: bool) -> usize {
    let real_size = size.saturating_mul(nmemb);
    if real_size == 0 || contents.is_null() {
        return real_size;
    }

    let bytes = unsafe { slice::from_raw_parts(contents.cast::<u8>(), real_size) };
    let printable_len = bytes.iter().position(|byte| *byte == 0).unwrap_or(real_size);

    let result = if stdout {
        let mut handle = io::stdout().lock();
        handle
            .write_all(&bytes[..printable_len])
            .and_then(|()| handle.flush())
    } else {
        let mut handle = io::stderr().lock();
        handle
            .write_all(&bytes[..printable_len])
            .and_then(|()| handle.flush())
    };

    let _ = result;
    real_size
}

unsafe fn handle_mut<'a>(curl: *mut CURL) -> Option<&'a mut CurlHandle> {
    if curl.is_null() {
        None
    } else {
        Some(unsafe { &mut *curl.cast::<CurlHandle>() })
    }
}

unsafe fn duplicate_c_string(value: *const c_char) -> Option<CString> {
    if value.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(value) }.to_owned())
    }
}

unsafe fn assign_owned_string(slot: &mut Option<CString>, value: *const c_char) {
    *slot = unsafe { duplicate_c_string(value) };
}

fn borrowed_string_ptr(value: Option<&CString>) -> *mut c_char {
    value
        .map(|string| string.as_ptr().cast_mut())
        .unwrap_or(ptr::null_mut())
}

fn current_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn add_cookie(curl: &mut CurlHandle, cookie: CookieEntry) {
    if let Some(index) = curl
        .cookies
        .iter()
        .position(|existing| existing.name == cookie.name)
    {
        curl.cookies.remove(index);
    }

    curl.cookies.insert(0, cookie);
}

fn load_cookies_from_file(curl: &mut CurlHandle, filename: *const c_char) -> i32 {
    if filename.is_null() {
        return -1;
    }

    let path = unsafe { CStr::from_ptr(filename) }.to_string_lossy().into_owned();
    let Ok(file) = File::open(&path) else {
        return 0;
    };

    let now = current_unix_seconds();
    let mut cookies_loaded = 0;

    for line in BufReader::new(file).lines() {
        let Ok(line) = line else {
            continue;
        };

        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut fields = line.split('\t');
        let Some(name) = fields.next() else {
            continue;
        };
        let Some(value) = fields.next() else {
            continue;
        };
        let Some(domain) = fields.next() else {
            continue;
        };
        let Some(path) = fields.next() else {
            continue;
        };
        let Some(expires) = fields.next() else {
            continue;
        };
        let Some(secure) = fields.next() else {
            continue;
        };
        let Some(http_only) = fields.next() else {
            continue;
        };
        let Some(samesite) = fields.next() else {
            continue;
        };

        let expires = expires.parse::<i64>().unwrap_or_default();
        if expires > 0 && expires < now {
            continue;
        }

        add_cookie(
            curl,
            CookieEntry {
                name: name.to_owned(),
                value: value.to_owned(),
                domain: domain.to_owned(),
                path: path.to_owned(),
                expires,
                secure: secure.parse::<i32>().unwrap_or_default() != 0,
                http_only: http_only.parse::<i32>().unwrap_or_default() != 0,
                samesite: if samesite.parse::<i32>().unwrap_or_default() != 0 {
                    CurlSameSite::Lax
                } else {
                    CurlSameSite::None
                },
            },
        );
        cookies_loaded += 1;
    }

    cookies_loaded
}

pub(crate) fn save_cookies_to_file(curl: &CurlHandle, filename: &CStr) -> i32 {
    let path = filename.to_string_lossy();
    let Ok(mut file) = File::create(path.as_ref()) else {
        return -1;
    };

    let _ = writeln!(file, "# BadgeVMS cookie jar file");
    let _ = writeln!(
        file,
        "# Format: name\\tvalue\\tdomain\\tpath\\texpires\\tsecure\\thttp_only\\tsamesite\\t"
    );

    let now = current_unix_seconds();
    let mut saved = 0;

    for cookie in &curl.cookies {
        if cookie.expires > 0 && cookie.expires < now {
            continue;
        }

        let samesite = match cookie.samesite {
            CurlSameSite::None => 0,
            CurlSameSite::Lax | CurlSameSite::Strict => 1,
        };

        if writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t",
            cookie.name,
            cookie.value,
            cookie.domain,
            cookie.path,
            cookie.expires,
            i32::from(cookie.secure),
            i32::from(cookie.http_only),
            samesite,
        )
        .is_ok()
        {
            saved += 1;
        }
    }

    saved
}

fn apply_userpwd(curl: &mut CurlHandle, userpwd: *const c_char) {
    if userpwd.is_null() {
        return;
    }

    let bytes = unsafe { CStr::from_ptr(userpwd) }.to_bytes();
    let Some(colon_index) = bytes.iter().position(|byte| *byte == b':') else {
        return;
    };

    curl.username = CString::new(&bytes[..colon_index]).ok();
    curl.password = CString::new(&bytes[colon_index + 1..]).ok();
}

fn parse_custom_method(method: *const c_char) -> CurlMethod {
    if method.is_null() {
        return CurlMethod::Get;
    }

    match unsafe { CStr::from_ptr(method) }.to_bytes() {
        b"GET" => CurlMethod::Get,
        b"POST" => CurlMethod::Post,
        b"PUT" => CurlMethod::Put,
        b"DELETE" => CurlMethod::Delete,
        b"HEAD" => CurlMethod::Head,
        b"PATCH" => CurlMethod::Patch,
        _ => CurlMethod::Get,
    }
}

// C varargs in Rust can read raw pointers directly but not function pointers, so the callback
// options round-trip through a raw pointer and are reinterpreted using the host C ABI.
fn decode_write_callback(raw: *mut c_void) -> Option<CurlWriteCallback> {
    if raw.is_null() {
        None
    } else {
        Some(unsafe { mem::transmute::<*mut c_void, CurlWriteCallback>(raw) })
    }
}

fn decode_header_callback(raw: *mut c_void) -> Option<CurlHeaderCallback> {
    if raw.is_null() {
        None
    } else {
        Some(unsafe { mem::transmute::<*mut c_void, CurlHeaderCallback>(raw) })
    }
}

unsafe fn next_long(args: &mut VaList<'_, '_>) -> c_long {
    unsafe { args.arg::<c_long>() }
}

unsafe fn next_c_string(args: &mut VaList<'_, '_>) -> *const c_char {
    unsafe { args.arg::<*const c_char>() }
}

unsafe fn next_void_ptr(args: &mut VaList<'_, '_>) -> *mut c_void {
    unsafe { args.arg::<*mut c_void>() }
}

unsafe fn next_slist_ptr(args: &mut VaList<'_, '_>) -> *mut curl_slist {
    unsafe { args.arg::<*mut curl_slist>() }
}

unsafe fn next_long_out_ptr(args: &mut VaList<'_, '_>) -> *mut c_long {
    unsafe { args.arg::<*mut c_long>() }
}

unsafe fn next_f64_out_ptr(args: &mut VaList<'_, '_>) -> *mut f64 {
    unsafe { args.arg::<*mut f64>() }
}

unsafe fn next_c_string_out_ptr(args: &mut VaList<'_, '_>) -> *mut *mut c_char {
    unsafe { args.arg::<*mut *mut c_char>() }
}

fn build_cookie_header(curl: &CurlHandle) -> Option<Vec<u8>> {
    let mut header = Vec::new();

    if let Some(manual) = curl.manual_cookies.as_ref() {
        header.extend_from_slice(manual.as_bytes());
    }

    for cookie in &curl.cookies {
        if !header.is_empty() {
            header.extend_from_slice(b"; ");
        }

        header.extend_from_slice(cookie.name.as_bytes());
        header.push(b'=');
        header.extend_from_slice(cookie.value.as_bytes());
    }

    if header.is_empty() {
        None
    } else {
        Some(header)
    }
}

fn build_request_headers(curl: &CurlHandle) -> HeaderMap {
    let mut headers = HeaderMap::new();

    if let Some(user_agent) = curl.user_agent.as_ref() {
        if let Ok(value) = HeaderValue::from_bytes(user_agent.as_bytes()) {
            headers.insert(USER_AGENT, value);
        }
    }

    let mut current = curl.headers;
    while !current.is_null() {
        unsafe {
            let data = (*current).data;
            if !data.is_null() {
                let bytes = CStr::from_ptr(data).to_bytes();
                if let Some(colon_index) = bytes.iter().position(|byte| *byte == b':') {
                    let name = &bytes[..colon_index];
                    let mut value = &bytes[colon_index + 1..];
                    while matches!(value.first(), Some(b' ' | b'\t')) {
                        value = &value[1..];
                    }

                    if let (Ok(name), Ok(value)) = (
                        HeaderName::from_bytes(name),
                        HeaderValue::from_bytes(value),
                    ) {
                        headers.insert(name, value);
                    }
                }
            }

            current = (*current).next;
        }
    }

    if let Some(cookie_header) = build_cookie_header(curl) {
        if let Ok(value) = HeaderValue::from_bytes(&cookie_header) {
            headers.insert(COOKIE, value);
        }
    }

    headers
}

fn trim_cookie_token(value: &str) -> &str {
    value.trim_matches([' ', '\t'])
}

fn parse_set_cookie(header: &[u8]) -> Option<CookieEntry> {
    let header = String::from_utf8_lossy(header);
    eprintln!("ESP_CURL: Parsing header '{}'", header);

    let mut parts = header.split(';');
    let name_value = parts.next()?;
    let (name, value) = name_value.split_once('=')?;

    let mut cookie = CookieEntry {
        name: trim_cookie_token(name).to_owned(),
        value: trim_cookie_token(value).to_owned(),
        domain: "example.com".to_owned(),
        path: "/".to_owned(),
        expires: 0,
        secure: false,
        http_only: false,
        samesite: CurlSameSite::None,
    };

    for attribute in parts {
        let attribute = attribute.trim_start_matches([' ', '\t']);
        if attribute.is_empty() {
            continue;
        }

        if let Some((name, value)) = attribute.split_once('=') {
            let name = trim_cookie_token(name);
            let value = trim_cookie_token(value);

            if name.eq_ignore_ascii_case("domain") {
                cookie.domain = value.to_owned();
            } else if name.eq_ignore_ascii_case("path") {
                cookie.path = value.to_owned();
            } else if name.eq_ignore_ascii_case("expires") {
                cookie.expires = 2_147_483_647;
            } else if name.eq_ignore_ascii_case("max-age") {
                let max_age = value.parse::<i64>().unwrap_or_default();
                if max_age > 0 {
                    cookie.expires = current_unix_seconds() + max_age;
                } else if max_age == 0 {
                    cookie.expires = 1;
                }
            } else if name.eq_ignore_ascii_case("samesite") {
                cookie.samesite = if value.eq_ignore_ascii_case("strict") {
                    CurlSameSite::Strict
                } else if value.eq_ignore_ascii_case("lax") {
                    CurlSameSite::Lax
                } else {
                    CurlSameSite::None
                };
            }
        } else if attribute.eq_ignore_ascii_case("secure") {
            cookie.secure = true;
        } else if attribute.eq_ignore_ascii_case("httponly") {
            cookie.http_only = true;
        }
    }

    Some(cookie)
}

fn apply_response_headers(curl: &mut CurlHandle, headers: &HeaderMap) {
    for (name, value) in headers {
        if name.as_str().eq_ignore_ascii_case("set-cookie") {
            if let Some(cookie) = parse_set_cookie(value.as_bytes()) {
                add_cookie(curl, cookie);
            }
        }

        if let Some(callback) = curl.header_function {
            let mut header = Vec::with_capacity(name.as_str().len() + value.as_bytes().len() + 2);
            header.extend_from_slice(name.as_str().as_bytes());
            header.extend_from_slice(b": ");
            header.extend_from_slice(value.as_bytes());

            unsafe {
                callback(
                    header.as_mut_ptr().cast::<c_void>(),
                    1,
                    header.len(),
                    curl.header_data,
                );
            }
        }
    }
}

fn stream_response_body(curl: &CurlHandle, response: &mut reqwest::blocking::Response) -> io::Result<()> {
    let chunk_size = curl.buffer_size.max(512);
    let mut buffer = vec![0_u8; chunk_size];

    loop {
        let bytes_read = response.read(&mut buffer)?;
        if bytes_read == 0 {
            return Ok(());
        }

        if let Some(callback) = curl.write_function {
            unsafe {
                callback(
                    buffer.as_mut_ptr().cast::<c_void>(),
                    1,
                    bytes_read,
                    curl.write_data,
                );
            }
        }
    }
}

fn method_to_reqwest(method: CurlMethod) -> Method {
    match method {
        CurlMethod::Get => Method::GET,
        CurlMethod::Post => Method::POST,
        CurlMethod::Put => Method::PUT,
        CurlMethod::Delete => Method::DELETE,
        CurlMethod::Head => Method::HEAD,
        CurlMethod::Patch => Method::PATCH,
    }
}

fn map_reqwest_error(error: &reqwest::Error) -> CURLcode {
    if error.is_timeout() {
        CURLE_OPERATION_TIMEDOUT_CODE
    } else if error.is_connect() {
        CURLE_COULDNT_CONNECT_CODE
    } else {
        CURLE_HTTP_RETURNED_ERROR_CODE
    }
}

fn build_client(curl: &CurlHandle) -> Result<ClientBuilder, CURLcode> {
    let mut builder = ClientBuilder::new()
        .danger_accept_invalid_certs(!curl.ssl_verify_peer)
        .danger_accept_invalid_hostnames(!curl.verify_host);

    if curl.timeout_ms > 0 {
        builder = builder.timeout(std::time::Duration::from_millis(curl.timeout_ms as u64));
    }

    if curl.max_redirection_count > 0 {
        builder = builder.redirect(Policy::limited(curl.max_redirection_count as usize));
    }

    if let Some(cert_pem) = curl.cert_pem.as_ref() {
        let certificates = Certificate::from_pem_bundle(cert_pem.as_bytes())
            .map_err(|_| CURLE_HTTP_RETURNED_ERROR_CODE)?;
        for certificate in certificates {
            builder = builder.add_root_certificate(certificate);
        }
    }

    Ok(builder)
}

fn perform_request(curl: &mut CurlHandle) -> (CURLcode, bool) {
    let Some(url) = curl.url.as_ref() else {
        return (CURLE_FAILED_INIT_CODE, false);
    };

    let builder = match build_client(curl) {
        Ok(builder) => builder,
        Err(error) => return (error, true),
    };

    let client = match builder.build() {
        Ok(client) => client,
        Err(error) => return (map_reqwest_error(&error), true),
    };

    let mut request = client.request(method_to_reqwest(curl.method), String::from_utf8_lossy(url.as_bytes()).into_owned());

    if let (Some(username), Some(password)) = (curl.username.as_ref(), curl.password.as_ref()) {
        request = request.basic_auth(
            String::from_utf8_lossy(username.as_bytes()).into_owned(),
            Some(String::from_utf8_lossy(password.as_bytes()).into_owned()),
        );
    }

    request = request.headers(build_request_headers(curl));

    if curl.method == CurlMethod::Post {
        if let Some(post_data) = curl.post_data.as_ref() {
            let body_len = curl.post_data_size.min(post_data.as_bytes().len());
            request = request.body(post_data.as_bytes()[..body_len].to_vec());
        }
    }

    let mut response = match request.send() {
        Ok(response) => response,
        Err(error) => return (map_reqwest_error(&error), true),
    };

    apply_response_headers(curl, response.headers());

    if stream_response_body(curl, &mut response).is_err() {
        return (CURLE_HTTP_RETURNED_ERROR_CODE, true);
    }

    curl.response_code = response.status().as_u16() as i32;
    curl.content_length = response.content_length().map(|length| length as i64).unwrap_or(-1);

    (CURLE_OK_CODE, true)
}

pub(crate) fn curl_easy_init_inner() -> *mut CURL {
    Box::into_raw(Box::new(CurlHandle::new())).cast::<CURL>()
}

pub(crate) unsafe fn curl_easy_setopt_inner(
    curl: *mut CURL,
    option: CURLoption,
    mut args: VaList<'_, '_>,
) -> CURLcode {
    let Some(curl) = (unsafe { handle_mut(curl) }) else {
        return CURLE_FAILED_INIT_CODE;
    };

    match option {
        CURLOPT_URL_OPTION => {
            unsafe { assign_owned_string(&mut curl.url, next_c_string(&mut args)) };
            CURLE_OK_CODE
        }
        CURLOPT_USERAGENT_OPTION => {
            unsafe { assign_owned_string(&mut curl.user_agent, next_c_string(&mut args)) };
            CURLE_OK_CODE
        }
        CURLOPT_TIMEOUT_OPTION => {
            curl.timeout_ms = unsafe { next_long(&mut args) }.saturating_mul(1000);
            CURLE_OK_CODE
        }
        CURLOPT_TIMEOUT_MS_OPTION => {
            curl.timeout_ms = unsafe { next_long(&mut args) };
            CURLE_OK_CODE
        }
        CURLOPT_SSL_VERIFYPEER_OPTION => {
            let verify = unsafe { next_long(&mut args) };
            curl.ssl_verify_peer = verify != 0;
            curl.verify_host = verify != 0;
            CURLE_OK_CODE
        }
        CURLOPT_SSL_VERIFYHOST_OPTION => {
            curl.verify_host = unsafe { next_long(&mut args) } != 0;
            CURLE_OK_CODE
        }
        CURLOPT_CAINFO_OPTION => {
            unsafe { assign_owned_string(&mut curl.cert_pem, next_c_string(&mut args)) };
            CURLE_OK_CODE
        }
        CURLOPT_USERPWD_OPTION => {
            apply_userpwd(curl, unsafe { next_c_string(&mut args) });
            CURLE_OK_CODE
        }
        CURLOPT_POSTFIELDS_OPTION => {
            let value = unsafe { next_c_string(&mut args) };
            curl.post_data = unsafe { duplicate_c_string(value) };
            curl.post_data_size = curl
                .post_data
                .as_ref()
                .map(|data| data.as_bytes().len())
                .unwrap_or_default();
            CURLE_OK_CODE
        }
        CURLOPT_POSTFIELDSIZE_OPTION => {
            let size = unsafe { next_long(&mut args) };
            curl.post_data_size = if size <= 0 { 0 } else { size as usize };
            CURLE_OK_CODE
        }
        CURLOPT_HTTPHEADER_OPTION => {
            curl.headers = unsafe { next_slist_ptr(&mut args) };
            CURLE_OK_CODE
        }
        CURLOPT_WRITEFUNCTION_OPTION => {
            curl.write_function = decode_write_callback(unsafe { next_void_ptr(&mut args) });
            CURLE_OK_CODE
        }
        CURLOPT_WRITEDATA_OPTION => {
            curl.write_data = unsafe { next_void_ptr(&mut args) };
            CURLE_OK_CODE
        }
        CURLOPT_HEADERFUNCTION_OPTION => {
            curl.header_function = decode_header_callback(unsafe { next_void_ptr(&mut args) });
            CURLE_OK_CODE
        }
        CURLOPT_HEADERDATA_OPTION => {
            curl.header_data = unsafe { next_void_ptr(&mut args) };
            CURLE_OK_CODE
        }
        CURLOPT_FOLLOWLOCATION_OPTION => {
            curl.max_redirection_count = if unsafe { next_long(&mut args) } != 0 { 10 } else { 0 };
            CURLE_OK_CODE
        }
        CURLOPT_MAXREDIRS_OPTION => {
            curl.max_redirection_count = unsafe { next_long(&mut args) };
            CURLE_OK_CODE
        }
        CURLOPT_VERBOSE_OPTION => {
            curl.verbose = unsafe { next_long(&mut args) } != 0;
            CURLE_OK_CODE
        }
        CURLOPT_CUSTOMREQUEST_OPTION => {
            curl.method = parse_custom_method(unsafe { next_c_string(&mut args) });
            CURLE_OK_CODE
        }
        CURLOPT_POST_OPTION => {
            if unsafe { next_long(&mut args) } != 0 {
                curl.method = CurlMethod::Post;
            }
            CURLE_OK_CODE
        }
        CURLOPT_PUT_OPTION => {
            if unsafe { next_long(&mut args) } != 0 {
                curl.method = CurlMethod::Put;
            }
            CURLE_OK_CODE
        }
        CURLOPT_HTTPGET_OPTION => {
            if unsafe { next_long(&mut args) } != 0 {
                curl.method = CurlMethod::Get;
            }
            CURLE_OK_CODE
        }
        CURLOPT_NOBODY_OPTION => {
            if unsafe { next_long(&mut args) } != 0 {
                curl.method = CurlMethod::Head;
            }
            CURLE_OK_CODE
        }
        CURLOPT_COOKIE_OPTION => {
            unsafe { assign_owned_string(&mut curl.manual_cookies, next_c_string(&mut args)) };
            CURLE_OK_CODE
        }
        CURLOPT_COOKIEFILE_OPTION => {
            let filename = unsafe { next_c_string(&mut args) };
            unsafe { assign_owned_string(&mut curl.cookie_file, filename) };
            let _ = load_cookies_from_file(curl, filename);
            CURLE_OK_CODE
        }
        CURLOPT_COOKIEJAR_OPTION => {
            unsafe { assign_owned_string(&mut curl.cookie_jar, next_c_string(&mut args)) };
            CURLE_OK_CODE
        }
        CURLOPT_BUFFERSIZE_OPTION => {
            let size = unsafe { next_long(&mut args) };
            curl.buffer_size = if size <= 0 { 0 } else { size as usize };
            CURLE_OK_CODE
        }
        CURLOPT_HTTPAUTH_OPTION => {
            curl.http_auth = unsafe { next_long(&mut args) };
            CURLE_OK_CODE
        }
        CURLOPT_PROXY_OPTION => {
            unsafe { assign_owned_string(&mut curl.proxy_url, next_c_string(&mut args)) };
            CURLE_UNSUPPORTED_PROTOCOL_CODE
        }
        CURLOPT_PROXYUSERPWD_OPTION => {
            unsafe { assign_owned_string(&mut curl.proxy_userpwd, next_c_string(&mut args)) };
            CURLE_UNSUPPORTED_PROTOCOL_CODE
        }
        CURLOPT_PROXYTYPE_OPTION => {
            curl.proxy_type = unsafe { next_long(&mut args) };
            CURLE_UNSUPPORTED_PROTOCOL_CODE
        }
        CURLOPT_PROXYPORT_OPTION => {
            curl.proxy_port = unsafe { next_long(&mut args) };
            CURLE_UNSUPPORTED_PROTOCOL_CODE
        }
        CURLOPT_PROXYAUTH_OPTION => {
            curl.proxy_auth = unsafe { next_long(&mut args) };
            CURLE_UNSUPPORTED_PROTOCOL_CODE
        }
        CURLOPT_RANGE_OPTION | CURLOPT_REFERER_OPTION | CURLOPT_CAPATH_OPTION => {
            let _ = unsafe { next_c_string(&mut args) };
            CURLE_UNSUPPORTED_PROTOCOL_CODE
        }
        CURLOPT_CONNECTTIMEOUT_MS_OPTION => {
            let _ = unsafe { next_long(&mut args) };
            CURLE_UNSUPPORTED_PROTOCOL_CODE
        }
        _ => CURLE_UNSUPPORTED_PROTOCOL_CODE,
    }
}

pub(crate) fn curl_easy_perform_inner(curl: *mut CURL) -> CURLcode {
    let Some(curl) = (unsafe { handle_mut(curl) }) else {
        return CURLE_FAILED_INIT_CODE;
    };

    let (result, save_cookie_jar) = perform_request(curl);
    if save_cookie_jar {
        if let Some(cookie_jar) = curl.cookie_jar.as_ref() {
            let _ = save_cookies_to_file(curl, cookie_jar.as_c_str());
        }
    }

    result
}

pub(crate) fn curl_easy_cleanup_inner(curl: *mut CURL) {
    if curl.is_null() {
        return;
    }

    unsafe {
        drop(Box::from_raw(curl.cast::<CurlHandle>()));
    }
}

pub(crate) unsafe fn curl_easy_getinfo_inner(
    curl: *mut CURL,
    info: curl_easy_info_t,
    mut args: VaList<'_, '_>,
) -> CURLcode {
    let Some(curl) = (unsafe { handle_mut(curl) }) else {
        return CURLE_FAILED_INIT_CODE;
    };

    match info {
        curl_easy_info_t::CURLINFO_RESPONSE_CODE => {
            let output = unsafe { next_long_out_ptr(&mut args) };
            if !output.is_null() {
                unsafe { *output = curl.response_code as c_long };
            }
            CURLE_OK_CODE
        }
        curl_easy_info_t::CURLINFO_CONTENT_LENGTH_DOWNLOAD => {
            let output = unsafe { next_f64_out_ptr(&mut args) };
            if !output.is_null() {
                unsafe { *output = curl.content_length as f64 };
            }
            CURLE_OK_CODE
        }
        curl_easy_info_t::CURLINFO_CONTENT_TYPE => {
            let output = unsafe { next_c_string_out_ptr(&mut args) };
            if !output.is_null() {
                unsafe {
                    *output = borrowed_string_ptr(curl.content_type.as_ref());
                }
            }
            CURLE_OK_CODE
        }
        curl_easy_info_t::CURLINFO_EFFECTIVE_URL => {
            let output = unsafe { next_c_string_out_ptr(&mut args) };
            if !output.is_null() {
                unsafe {
                    *output = borrowed_string_ptr(curl.effective_url.as_ref().or(curl.url.as_ref()));
                }
            }
            CURLE_OK_CODE
        }
    }
}

pub(crate) fn curl_easy_strerror_inner(error: CURLcode) -> *const c_char {
    match error {
        CURLE_OK_CODE => b"No error\0".as_ptr().cast::<c_char>(),
        CURLE_UNSUPPORTED_PROTOCOL_CODE => b"Unsupported protocol\0".as_ptr().cast::<c_char>(),
        CURLE_FAILED_INIT_CODE => b"Failed initialization\0".as_ptr().cast::<c_char>(),
        CURLE_URL_MALFORMAT_CODE => b"URL malformat\0".as_ptr().cast::<c_char>(),
        CURLE_COULDNT_RESOLVE_HOST_CODE => b"Couldn't resolve host\0".as_ptr().cast::<c_char>(),
        CURLE_COULDNT_CONNECT_CODE => b"Couldn't connect\0".as_ptr().cast::<c_char>(),
        CURLE_HTTP_RETURNED_ERROR_CODE => b"HTTP returned error\0".as_ptr().cast::<c_char>(),
        CURLE_OPERATION_TIMEDOUT_CODE => b"Operation timed out\0".as_ptr().cast::<c_char>(),
        CURLE_SSL_CONNECT_ERROR_CODE => b"SSL connect error\0".as_ptr().cast::<c_char>(),
        _ => b"Unknown error\0".as_ptr().cast::<c_char>(),
    }
}

pub(crate) fn curl_slist_append_inner(
    list: *mut curl_slist,
    string: *const c_char,
) -> *mut curl_slist {
    let duplicated = if string.is_null() {
        ptr::null_mut()
    } else {
        unsafe { CStr::from_ptr(string) }.to_owned().into_raw()
    };

    let new_node = Box::into_raw(Box::new(curl_slist {
        data: duplicated,
        next: ptr::null_mut(),
    }));

    if list.is_null() {
        return new_node;
    }

    let mut current = list;
    unsafe {
        while !(*current).next.is_null() {
            current = (*current).next;
        }
        (*current).next = new_node;
    }

    list
}

pub(crate) fn curl_slist_free_all_inner(mut list: *mut curl_slist) {
    while !list.is_null() {
        unsafe {
            let next = (*list).next;
            if !(*list).data.is_null() {
                drop(CString::from_raw((*list).data));
            }
            drop(Box::from_raw(list));
            list = next;
        }
    }
}

pub(crate) fn curl_global_init_inner(_flags: c_long) -> CURLcode {
    CURLE_OK_CODE
}

pub(crate) fn curl_global_cleanup_inner() {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emulated::badgevms::curl::{
        curl_easy_cleanup, curl_easy_getinfo, curl_easy_init, curl_easy_perform,
        curl_easy_setopt,
    };
    use std::{
        fs,
        net::{TcpListener, TcpStream},
        sync::{Arc, Mutex},
        thread,
    };

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct RecordedRequest {
        method: String,
        path: String,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
    }

    fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack.windows(needle.len()).position(|window| window == needle)
    }

    fn read_request(stream: &mut TcpStream) -> RecordedRequest {
        let mut buffer = Vec::new();
        let mut chunk = [0_u8; 1024];
        let header_end = loop {
            let bytes_read = stream.read(&mut chunk).unwrap();
            assert!(bytes_read > 0, "connection closed before request headers arrived");
            buffer.extend_from_slice(&chunk[..bytes_read]);

            if let Some(position) = find_subslice(&buffer, b"\r\n\r\n") {
                break position + 4;
            }
        };

        let header_text = String::from_utf8_lossy(&buffer[..header_end]).into_owned();
        let mut lines = header_text.split("\r\n");
        let request_line = lines.next().unwrap_or_default();
        let mut request_line_parts = request_line.split_whitespace();
        let method = request_line_parts.next().unwrap_or_default().to_owned();
        let path = request_line_parts.next().unwrap_or_default().to_owned();

        let mut headers = Vec::new();
        let mut content_length = 0_usize;
        for line in lines {
            if line.is_empty() {
                break;
            }

            if let Some((name, value)) = line.split_once(':') {
                let value = value.trim_start().to_owned();
                if name.eq_ignore_ascii_case("Content-Length") {
                    content_length = value.parse::<usize>().unwrap_or_default();
                }
                headers.push((name.to_owned(), value));
            }
        }

        let mut body = buffer[header_end..].to_vec();
        while body.len() < content_length {
            let bytes_read = stream.read(&mut chunk).unwrap();
            if bytes_read == 0 {
                break;
            }
            body.extend_from_slice(&chunk[..bytes_read]);
        }
        body.truncate(content_length);

        RecordedRequest {
            method,
            path,
            headers,
            body,
        }
    }

    fn write_response(stream: &mut TcpStream, headers: &[(String, String)], body: &[u8]) {
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n",
            body.len()
        )
        .unwrap();

        for (name, value) in headers {
            write!(stream, "{}: {}\r\n", name, value).unwrap();
        }

        write!(stream, "\r\n").unwrap();
        stream.write_all(body).unwrap();
        stream.flush().unwrap();
    }

    fn spawn_single_request_server(
        response_headers: Vec<(String, String)>,
        response_body: Vec<u8>,
    ) -> (
        String,
        Arc<Mutex<Option<RecordedRequest>>>,
        thread::JoinHandle<()>,
    ) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let captured = Arc::new(Mutex::new(None));
        let captured_request = Arc::clone(&captured);

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let request = read_request(&mut stream);
            *captured_request.lock().unwrap() = Some(request);
            write_response(&mut stream, &response_headers, &response_body);
        });

        (format!("http://{address}"), captured, handle)
    }

    fn unique_temp_file(name: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!("why2025-curl-{name}-{unique}.txt"))
    }

    #[test]
    fn put_with_postfields_keeps_the_upstream_no_body_bug() {
        let (url, captured, server) = spawn_single_request_server(Vec::new(), Vec::new());
        let url = CString::new(url).unwrap();
        let put = CString::new("PUT").unwrap();
        let postfields = CString::new("payload").unwrap();

        let curl = curl_easy_init();
        assert!(!curl.is_null());

        unsafe {
            assert_eq!(curl_easy_setopt(curl, CURLOPT_URL_OPTION, url.as_ptr()), CURLE_OK_CODE);
            assert_eq!(
                curl_easy_setopt(curl, CURLOPT_CUSTOMREQUEST_OPTION, put.as_ptr()),
                CURLE_OK_CODE
            );
            assert_eq!(
                curl_easy_setopt(curl, CURLOPT_POSTFIELDS_OPTION, postfields.as_ptr()),
                CURLE_OK_CODE
            );
        }

        assert_eq!(curl_easy_perform(curl), CURLE_OK_CODE);

        let mut response_code: c_long = 0;
        unsafe {
            assert_eq!(
                curl_easy_getinfo(
                    curl,
                    curl_easy_info_t::CURLINFO_RESPONSE_CODE,
                    &mut response_code as *mut c_long,
                ),
                CURLE_OK_CODE,
            );
        }
        assert_eq!(response_code, 200);

        curl_easy_cleanup(curl);
        server.join().unwrap();

        let request = captured.lock().unwrap().take().unwrap();
        assert_eq!(request.method, "PUT");
        assert_eq!(request.path, "/");
        assert!(request.body.is_empty());
    }

    #[test]
    fn cookie_jar_save_collapses_strict_to_lax() {
        let response_headers = vec![(
            "Set-Cookie".to_owned(),
            "session=value; SameSite=Strict".to_owned(),
        )];
        let (url, _captured, server) = spawn_single_request_server(response_headers, Vec::new());
        let url = CString::new(url).unwrap();
        let cookie_jar = unique_temp_file("cookie-jar");
        let cookie_jar_c = CString::new(cookie_jar.to_string_lossy().into_owned()).unwrap();

        let curl = curl_easy_init();
        assert!(!curl.is_null());

        unsafe {
            assert_eq!(curl_easy_setopt(curl, CURLOPT_URL_OPTION, url.as_ptr()), CURLE_OK_CODE);
            assert_eq!(
                curl_easy_setopt(curl, CURLOPT_COOKIEJAR_OPTION, cookie_jar_c.as_ptr()),
                CURLE_OK_CODE
            );
        }

        assert_eq!(curl_easy_perform(curl), CURLE_OK_CODE);
        curl_easy_cleanup(curl);
        server.join().unwrap();

        let cookie_file = fs::read_to_string(&cookie_jar).unwrap();
        let cookie_line = cookie_file
            .lines()
            .find(|line| !line.is_empty() && !line.starts_with('#'))
            .unwrap();
        assert!(cookie_line.ends_with("\t1\t"));

        let _ = fs::remove_file(cookie_jar);
    }
}