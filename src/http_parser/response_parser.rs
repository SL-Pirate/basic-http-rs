use crate::http_parser::{
    BAD_REQUEST, FOUND, HttpResponse, HttpResponseCode, INTERNAL_SERVER_ERROR, LOCATION, NOT_FOUND,
    OK,
};
use std::collections::HashMap;

impl<T: Clone> HttpResponse<T> {
    pub fn builder() -> HttpResponseBuilder<T> {
        HttpResponseBuilder::new()
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn body(&self) -> &Option<T> {
        &self.body
    }
}

impl HttpResponse<Vec<u8>> {
    /// status defaults to OK
    pub fn bad_request(body: Option<String>) -> HttpResponse<Vec<u8>> {
        let mut res = Self::builder().status_and_code(BAD_REQUEST).build();
        res.body = body.map(|raw| raw.into_bytes());
        res
    }

    pub fn not_found(body: Option<String>) -> HttpResponse<Vec<u8>> {
        let mut res = Self::builder().status_and_code(NOT_FOUND).build();
        res.body = body.map(|some| some.into_bytes());
        res
    }

    pub fn internal_server_error(body: Option<String>) -> HttpResponse<Vec<u8>> {
        let mut res = Self::builder()
            .status_and_code(INTERNAL_SERVER_ERROR)
            .build();
        res.body = body.map(|b| b.into_bytes());
        res
    }

    pub fn redirect(location: &str) -> HttpResponse<Vec<u8>> {
        Self::builder()
            .status_and_code(FOUND)
            .add_header(LOCATION, location)
            .build()
    }
}

pub struct HttpResponseBuilder<T> {
    status: String,
    status_code: u16,
    headers: HashMap<String, String>,
    body: Option<T>,
}

impl<T: Clone> HttpResponseBuilder<T> {
    fn new() -> HttpResponseBuilder<T> {
        HttpResponseBuilder {
            body: None,
            headers: HashMap::new(),
            status_code: OK.0,
            status: String::from(OK.1),
        }
    }

    pub fn status(&mut self, status: &str) -> &mut HttpResponseBuilder<T> {
        self.status = String::from(status);
        self
    }

    pub fn status_and_code(
        &mut self,
        (code, status): HttpResponseCode,
    ) -> &mut HttpResponseBuilder<T> {
        self.status = String::from(status);
        self.status_code = code;
        self
    }

    pub fn status_code(&mut self, status_code: u16) -> &mut HttpResponseBuilder<T> {
        self.status_code = status_code;
        self
    }

    pub fn add_header(&mut self, key: &str, value: &str) -> &mut HttpResponseBuilder<T> {
        self.headers.insert(String::from(key), String::from(value));

        self
    }

    pub fn headers(&mut self, headers: HashMap<String, String>) -> &mut HttpResponseBuilder<T> {
        for (k, v) in headers {
            self.headers.insert(k, v);
        }

        self
    }

    pub fn body(&mut self, body: T) -> &mut HttpResponseBuilder<T> {
        self.body = Some(body);
        self
    }

    pub fn build(&self) -> HttpResponse<T> {
        HttpResponse {
            body: self.body.clone(),
            headers: self.headers.clone(),
            status: self.status.clone(),
            status_code: self.status_code,
        }
    }
}
