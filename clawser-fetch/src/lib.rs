mod error;

pub use error::{ClawserError, Result};

use clawser_sys as ffi;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::path::Path;
use std::ptr;

/// Deterministic seed for fingerprint identity.
/// Same seed = same fingerprint every time.
///
/// Fields:
/// - `hw_seed`: Selects hardware profile (GPU, CPU cores, screen) + Chrome version + HTTP/2 profile
/// - `canvas_seed`: Deterministic canvas fingerprint noise
/// - `webgl_seed`: Deterministic WebGL readPixels noise
/// - `audio_seed`: Deterministic AudioContext noise
/// - `client_rects_seed`: Deterministic getBoundingClientRect noise
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seed {
    pub hw_seed: u64,
    pub canvas_seed: u64,
    pub webgl_seed: u64,
    pub audio_seed: u64,
    pub client_rects_seed: u64,
}

impl Seed {
    /// Create a seed from 5 raw u64 values.
    pub fn new(hw: u64, canvas: u64, webgl: u64, audio: u64, client_rects: u64) -> Self {
        Seed {
            hw_seed: hw,
            canvas_seed: canvas,
            webgl_seed: webgl,
            audio_seed: audio,
            client_rects_seed: client_rects,
        }
    }

    /// Serialize to a compact hex string for storage.
    pub fn to_hex(&self) -> String {
        format!(
            "{:016x}{:016x}{:016x}{:016x}{:016x}",
            self.hw_seed, self.canvas_seed, self.webgl_seed,
            self.audio_seed, self.client_rects_seed
        )
    }

    /// Parse from a hex string produced by `to_hex()`.
    pub fn from_hex(s: &str) -> Option<Self> {
        if s.len() != 80 {
            return None;
        }
        Some(Seed {
            hw_seed: u64::from_str_radix(&s[0..16], 16).ok()?,
            canvas_seed: u64::from_str_radix(&s[16..32], 16).ok()?,
            webgl_seed: u64::from_str_radix(&s[32..48], 16).ok()?,
            audio_seed: u64::from_str_radix(&s[48..64], 16).ok()?,
            client_rects_seed: u64::from_str_radix(&s[64..80], 16).ok()?,
        })
    }
}

/// An HTTP session backed by Chromium's network stack with antidetect
/// fingerprinting. Maintains cookies and HTTP/2 connections across requests.
///
/// Thread-safe: can be shared across threads via `Arc<Session>`.
pub struct Session {
    ptr: *mut ffi::ClawserSession,
}

unsafe impl Send for Session {}
unsafe impl Sync for Session {}

impl Session {
    /// Create a session with a random fingerprint identity.
    /// Returns the session and the seed used. Save the seed to replay later.
    pub fn random() -> Result<(Self, Seed)> {
        let mut raw_seed = ffi::ClawserSeed {
            hw_seed: 0,
            canvas_seed: 0,
            webgl_seed: 0,
            audio_seed: 0,
            client_rects_seed: 0,
        };
        let ptr = unsafe { ffi::clawser_session_create_random(&mut raw_seed) };
        if ptr.is_null() {
            return Err(ClawserError::InitFailed(last_error()));
        }
        let seed = Seed {
            hw_seed: raw_seed.hw_seed,
            canvas_seed: raw_seed.canvas_seed,
            webgl_seed: raw_seed.webgl_seed,
            audio_seed: raw_seed.audio_seed,
            client_rects_seed: raw_seed.client_rects_seed,
        };
        Ok((Session { ptr }, seed))
    }

    /// Create a session from a seed. Same seed = same fingerprint.
    pub fn from_seed(seed: &Seed) -> Result<Self> {
        let raw_seed = ffi::ClawserSeed {
            hw_seed: seed.hw_seed,
            canvas_seed: seed.canvas_seed,
            webgl_seed: seed.webgl_seed,
            audio_seed: seed.audio_seed,
            client_rects_seed: seed.client_rects_seed,
        };
        let ptr = unsafe { ffi::clawser_session_from_seed(&raw_seed) };
        if ptr.is_null() {
            return Err(ClawserError::InitFailed(last_error()));
        }
        Ok(Session { ptr })
    }

    /// Create a session from a config JSON file (full control).
    pub fn from_config(config_path: impl AsRef<Path>) -> Result<Self> {
        let path_str = config_path
            .as_ref()
            .to_str()
            .ok_or_else(|| ClawserError::InvalidArgument("path contains invalid UTF-8".into()))?;
        let c_path = CString::new(path_str)
            .map_err(|_| ClawserError::InvalidArgument("path contains null byte".into()))?;
        let ptr = unsafe { ffi::clawser_session_create(c_path.as_ptr()) };
        if ptr.is_null() {
            return Err(ClawserError::InitFailed(last_error()));
        }
        Ok(Session { ptr })
    }

    pub fn get(&self, url: &str) -> Result<RequestBuilder> {
        self.request("GET", url)
    }

    pub fn post(&self, url: &str) -> Result<RequestBuilder> {
        self.request("POST", url)
    }

    pub fn put(&self, url: &str) -> Result<RequestBuilder> {
        self.request("PUT", url)
    }

    pub fn delete(&self, url: &str) -> Result<RequestBuilder> {
        self.request("DELETE", url)
    }

    pub fn request(&self, method: &str, url: &str) -> Result<RequestBuilder> {
        let c_method = CString::new(method)
            .map_err(|_| ClawserError::InvalidArgument("method contains null byte".into()))?;
        let c_url = CString::new(url)
            .map_err(|_| ClawserError::InvalidArgument("url contains null byte".into()))?;
        let ptr =
            unsafe { ffi::clawser_request_new(self.ptr, c_method.as_ptr(), c_url.as_ptr()) };
        if ptr.is_null() {
            return Err(ClawserError::NullPointer);
        }
        Ok(RequestBuilder { ptr })
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::clawser_session_destroy(self.ptr) };
        }
    }
}

/// Builder for constructing HTTP requests.
pub struct RequestBuilder {
    ptr: *mut ffi::ClawserRequest,
}

impl RequestBuilder {
    pub fn header(self, name: &str, value: &str) -> Self {
        if let (Ok(c_name), Ok(c_value)) = (CString::new(name), CString::new(value)) {
            unsafe { ffi::clawser_request_set_header(self.ptr, c_name.as_ptr(), c_value.as_ptr()) };
        }
        self
    }

    pub fn body(self, data: &[u8]) -> Self {
        unsafe { ffi::clawser_request_set_body(self.ptr, data.as_ptr(), data.len()) };
        self
    }

    pub fn json(self, body: &str) -> Self {
        self.header("Content-Type", "application/json")
            .body(body.as_bytes())
    }

    pub fn max_redirects(self, n: i32) -> Self {
        unsafe { ffi::clawser_request_set_max_redirects(self.ptr, n) };
        self
    }

    pub fn timeout_ms(self, ms: u32) -> Self {
        unsafe { ffi::clawser_request_set_timeout_ms(self.ptr, ms) };
        self
    }

    /// Send the request (blocking). Consumes the builder.
    pub fn send(mut self) -> Result<Response> {
        let req_ptr = self.ptr;
        self.ptr = ptr::null_mut();
        let resp_ptr = unsafe { ffi::clawser_request_send(req_ptr) };
        if resp_ptr.is_null() {
            return Err(ClawserError::Network(last_error()));
        }
        Ok(Response { ptr: resp_ptr })
    }
}

impl Drop for RequestBuilder {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::clawser_request_destroy(self.ptr) };
        }
    }
}

/// An HTTP response.
pub struct Response {
    ptr: *mut ffi::ClawserResponse,
}

impl Response {
    pub fn status(&self) -> u16 {
        unsafe { ffi::clawser_response_status_code(self.ptr) as u16 }
    }

    pub fn header(&self, name: &str) -> Option<String> {
        let c_name = CString::new(name).ok()?;
        let ptr = unsafe { ffi::clawser_response_header(self.ptr, c_name.as_ptr()) };
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned())
    }

    pub fn headers(&self) -> HashMap<String, String> {
        let count = unsafe { ffi::clawser_response_header_count(self.ptr) };
        let mut map = HashMap::with_capacity(count);
        for i in 0..count {
            let name_ptr = unsafe { ffi::clawser_response_header_name_at(self.ptr, i) };
            let value_ptr = unsafe { ffi::clawser_response_header_value_at(self.ptr, i) };
            if !name_ptr.is_null() && !value_ptr.is_null() {
                let name = unsafe { CStr::from_ptr(name_ptr) }.to_string_lossy().into_owned();
                let value = unsafe { CStr::from_ptr(value_ptr) }.to_string_lossy().into_owned();
                map.insert(name, value);
            }
        }
        map
    }

    pub fn body(&self) -> &[u8] {
        let ptr = unsafe { ffi::clawser_response_body(self.ptr) };
        let len = unsafe { ffi::clawser_response_body_len(self.ptr) };
        if ptr.is_null() || len == 0 {
            return &[];
        }
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }

    pub fn text(&self) -> Result<String> {
        String::from_utf8(self.body().to_vec()).map_err(|_| ClawserError::InvalidUtf8)
    }

    pub fn url(&self) -> String {
        let ptr = unsafe { ffi::clawser_response_url(self.ptr) };
        if ptr.is_null() {
            return String::new();
        }
        unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned()
    }
}

impl Drop for Response {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::clawser_response_destroy(self.ptr) };
        }
    }
}

pub fn version() -> String {
    let ptr = unsafe { ffi::clawser_version() };
    if ptr.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned()
}

fn last_error() -> String {
    let ptr = unsafe { ffi::clawser_last_error() };
    if ptr.is_null() {
        return "unknown error".to_string();
    }
    unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned()
}
