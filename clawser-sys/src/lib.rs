#![allow(non_camel_case_types)]

use std::os::raw::c_char;

#[repr(C)]
pub struct ClawserSession {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(C)]
pub struct ClawserRequest {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[repr(C)]
pub struct ClawserResponse {
    _data: [u8; 0],
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Seed struct for deterministic fingerprint identity.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClawserSeed {
    pub hw_seed: u64,
    pub canvas_seed: u64,
    pub webgl_seed: u64,
    pub audio_seed: u64,
    pub client_rects_seed: u64,
}

unsafe extern "C" {
    pub fn clawser_session_create_random(out_seed: *mut ClawserSeed) -> *mut ClawserSession;
    pub fn clawser_session_from_seed(seed: *const ClawserSeed) -> *mut ClawserSession;
    pub fn clawser_session_create(config_json_path: *const c_char) -> *mut ClawserSession;
    pub fn clawser_session_destroy(session: *mut ClawserSession);
    pub fn clawser_last_error() -> *const c_char;

    pub fn clawser_request_new(
        session: *mut ClawserSession,
        method: *const c_char,
        url: *const c_char,
    ) -> *mut ClawserRequest;
    pub fn clawser_request_set_header(
        req: *mut ClawserRequest,
        name: *const c_char,
        value: *const c_char,
    );
    pub fn clawser_request_set_body(req: *mut ClawserRequest, data: *const u8, len: usize);
    pub fn clawser_request_set_max_redirects(req: *mut ClawserRequest, max_redirects: i32);
    pub fn clawser_request_set_timeout_ms(req: *mut ClawserRequest, timeout_ms: u32);
    pub fn clawser_request_send(req: *mut ClawserRequest) -> *mut ClawserResponse;
    pub fn clawser_request_destroy(req: *mut ClawserRequest);

    pub fn clawser_response_status_code(resp: *const ClawserResponse) -> i32;
    pub fn clawser_response_header(resp: *const ClawserResponse, name: *const c_char)
        -> *const c_char;
    pub fn clawser_response_header_count(resp: *const ClawserResponse) -> usize;
    pub fn clawser_response_header_name_at(resp: *const ClawserResponse, index: usize)
        -> *const c_char;
    pub fn clawser_response_header_value_at(resp: *const ClawserResponse, index: usize)
        -> *const c_char;
    pub fn clawser_response_body(resp: *const ClawserResponse) -> *const u8;
    pub fn clawser_response_body_len(resp: *const ClawserResponse) -> usize;
    pub fn clawser_response_url(resp: *const ClawserResponse) -> *const c_char;
    pub fn clawser_response_destroy(resp: *mut ClawserResponse);

    pub fn clawser_version() -> *const c_char;
}
