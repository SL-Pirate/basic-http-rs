use crate::http_parser::{
    BAD_REQUEST, CONNECTION_HEADER, CONTENT_LENGTH_HEADER, FOUND, HttpResponse, HttpResponseCode,
    INTERNAL_SERVER_ERROR, LOCATION, NOT_FOUND, OK,
};
use std::collections::HashMap;

impl HttpResponse {
    /// status defaults to OK
    pub fn builder() -> HttpResponseBuilder {
        HttpResponseBuilder::new()
    }

    pub fn bad_request(body: Option<String>) -> HttpResponse {
        let mut res = Self::builder().status_and_code(BAD_REQUEST).build();
        res.body = body.map(|raw| raw.into_bytes());
        res
    }

    pub fn not_found(body: Option<String>) -> HttpResponse {
        let mut res = Self::builder().status_and_code(NOT_FOUND).build();
        res.body = body.map(|some| some.into_bytes());
        res
    }

    pub fn internal_server_error(body: Option<String>) -> HttpResponse {
        let mut res = Self::builder()
            .status_and_code(INTERNAL_SERVER_ERROR)
            .build();
        res.body = body.map(|b| b.into_bytes());
        res
    }

    pub fn redirect(location: &str) -> HttpResponse {
        Self::builder()
            .status_and_code(FOUND)
            .add_header(LOCATION, location)
            .build()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let res = self.encode();
        let mut res_raw = res.into_bytes();
        if let Some(mut body) = self.body.clone() {
            res_raw.append(body.as_mut())
        }

        res_raw
    }

    fn encode(&self) -> String {
        let status_code = self.status_code;
        let content_len = self.body.clone().map_or(0, |body| body.len());
        let status = self.status.clone();
        let mut res = format!("HTTP/1.1 {status_code} {status}\r\n");
        for (header, value) in self.headers.clone() {
            res.push_str(format!("{header}: {value}\r\n").as_str())
        }
        if !self.headers.contains_key(CONTENT_LENGTH_HEADER) {
            res.push_str(format!("{CONTENT_LENGTH_HEADER}: {content_len}\r\n").as_str())
        }
        if !self.headers.contains_key(CONNECTION_HEADER) {
            res.push_str(format!("{CONNECTION_HEADER}: close\r\n").as_str())
        }
        res.push_str("\r\n");
        res
    }
}

pub struct HttpResponseBuilder {
    status: String,
    status_code: u16,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
}

impl HttpResponseBuilder {
    fn new() -> HttpResponseBuilder {
        HttpResponseBuilder {
            body: None,
            headers: HashMap::new(),
            status_code: OK.0,
            status: String::from(OK.1),
        }
    }

    pub fn status(&mut self, status: &str) -> &mut HttpResponseBuilder {
        self.status = String::from(status);
        self
    }

    pub fn status_and_code(
        &mut self,
        (code, status): HttpResponseCode,
    ) -> &mut HttpResponseBuilder {
        self.status = String::from(status);
        self.status_code = code;
        self
    }

    pub fn status_code(&mut self, status_code: u16) -> &mut HttpResponseBuilder {
        self.status_code = status_code;
        self
    }

    pub fn add_header(&mut self, key: &str, value: &str) -> &mut HttpResponseBuilder {
        self.headers.insert(String::from(key), String::from(value));

        self
    }

    pub fn headers(&mut self, headers: HashMap<String, String>) -> &mut HttpResponseBuilder {
        for (k, v) in headers {
            self.headers.insert(k, v);
        }

        self
    }

    pub fn body(&mut self, body: Vec<u8>) -> &mut HttpResponseBuilder {
        self.body = Some(body);
        self
    }

    pub fn body_from_string(&mut self, body: String) -> &mut HttpResponseBuilder {
        self.body = Some(body.into_bytes());
        self
    }

    pub fn build(&self) -> HttpResponse {
        HttpResponse {
            body: self.body.clone(),
            headers: self.headers.clone(),
            status: self.status.clone(),
            status_code: self.status_code,
        }
    }
}
