// Module to expose Blocker types to Wasm Bindgen

use adblock::request::Request;
use wasm_bindgen::prelude::*;

/// The type of resource requested from the URL endpoint.
#[wasm_bindgen]
#[derive(Clone, PartialEq, Debug)]
pub enum VanguardRequestType {
    Beacon,
    Csp,
    Document,
    Dtd,
    Fetch,
    Font,
    Image,
    Media,
    Object,
    Other,
    Ping,
    Script,
    Stylesheet,
    Subdocument,
    Websocket,
    Xlst,
    Xmlhttprequest,
}

#[derive(Debug, PartialEq)]
#[wasm_bindgen]
pub enum RequestError {
    HostnameParseError,
    SourceHostnameParseError,
    UnicodeDecodingError,
}

#[derive(Debug)]
#[wasm_bindgen(getter_with_clone, inspectable)]
pub struct VanguardRequest {
    pub request_type: VanguardRequestType,

    pub is_http: bool,
    pub is_https: bool,
    pub is_supported: bool,
    pub is_third_party: bool,
    pub url: String,
    pub hostname: String,
    pub source_hostname_hashes: Option<Vec<u64>>,

    pub/*(crate)*/ url_lower_cased: String,
    pub/*(crate)*/ request_tokens: Vec<u64>,
    pub/*(crate)*/ original_url: String,
}

#[wasm_bindgen]
impl VanguardRequest {
    #[wasm_bindgen(constructor)]
    pub fn new(url: &str, source_url: &str, request_type: &str, method: &str) -> Result<VanguardRequest, RequestError> {
        unsafe {
            match Request::new(url, source_url, request_type, method) {
                Ok(r) => Ok(std::mem::transmute(r)),
                Err(e) => Err(std::mem::transmute(e))
            }
        }
    }

    /// If you're building a [`Request`] in a context that already has access to parsed
    /// representations of the input URLs, you can use this constructor to avoid extra lookups from
    /// the public suffix list. Take care to pass data correctly.
    pub fn preparsed(
        url: &str,
        hostname: &str,
        source_hostname: &str,
        request_type: &str,
        third_party: bool,
        method: &str
    ) -> Self {
        unsafe { 
            std::mem::transmute(
                Request::preparsed(url, hostname, source_hostname, request_type, third_party, method)
            )
        }
    }
}

