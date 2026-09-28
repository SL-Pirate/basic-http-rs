pub mod request_parser;
pub mod response_parser;

use std::collections::HashMap;

pub struct HttpRequest {
    method: HttpMethod,
    standard: String,
    path: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

pub struct HttpResponse {
    status: String,
    status_code: u16,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
}

#[derive(Clone)]
pub enum HttpMethod {
    GET,
    POST,
    DELETE,
    PUT,
    PATCH,
    OPTIONS,
    QUERY,
    HEAD,
    TRACE,
    CONNECT,
}

/// a few known header literals
pub const CONTENT_TYPE: &str = "Content-Type";
pub const CONTENT_LENGTH_HEADER: &str = "Content-Length";
pub const CONNECTION_HEADER: &str = "Connection";
pub const LOCATION: &str = "Location";

pub type HttpResponseCode = (u16, &'static str);

/// a few used Http status literals
pub const BAD_REQUEST: HttpResponseCode = (400, "Bad Request");
pub const NOT_FOUND: HttpResponseCode = (404, "Not Found");
pub const FOUND: HttpResponseCode = (302, "Found");
pub const OK: HttpResponseCode = (200, "OK");

pub const INTERNAL_SERVER_ERROR: HttpResponseCode = (500, "Internal Server Error");
