use crate::http_parser::{ACCEPT_ENCODING, HttpMethod, HttpRequest};
use std::collections::HashMap;

impl HttpRequest {
    pub fn from_string(parsed: String) -> Result<HttpRequest, HttpRequestParseError> {
        let mut lines = parsed.split("\r\n");
        let first = lines.next();
        if first.is_none() {
            return Err(HttpRequestParseError::EmptyRequest);
        }
        let first_split: Vec<&str> = first.unwrap().split_whitespace().collect();

        if first_split.len() != 3 {
            return Err(HttpRequestParseError::InvalidRequest);
        }

        let mut headers: HashMap<String, String> = HashMap::new();

        // extracting headers
        loop {
            match lines.next() {
                None => {
                    break;
                }
                Some(header) => {
                    if header == "\r\n\r\n" {
                        break;
                    }
                    let header_split: Vec<&str> = header.split(":").collect();
                    if header_split.len() != 2 {
                        continue;
                    }
                    headers.insert(String::from(header_split[0]), String::from(header_split[1]));
                }
            };
        }

        let body_lines: Vec<&str> = lines.collect();
        let body = body_lines.concat();

        Ok(HttpRequest {
            headers,
            method: HttpMethod::from_str(first_split[0]),
            path: String::from(first_split[1]),
            standard: String::from(first_split[2]),
            body: Some(body),
        })
    }

    pub fn from_vec(raw: Vec<u8>) -> Result<HttpRequest, HttpRequestParseError> {
        let string: String = String::from_utf8(raw)
            .ok()
            .ok_or(HttpRequestParseError::InvalidRequest)?;
        Self::from_string(string)
    }

    pub fn path(&self) -> String {
        self.path.clone()
    }

    pub fn method(&self) -> HttpMethod {
        self.method.clone()
    }

    pub fn standard(&self) -> String {
        self.standard.clone()
    }

    pub fn headers(&self) -> HashMap<String, String> {
        self.headers.clone()
    }

    pub fn get_acceptable_encodings(&self) -> Vec<&str> {
        if let Some(accepted_encodings) = self.headers.get(ACCEPT_ENCODING) {
            let mut encodings: Vec<&str> = Vec::new();
            let mut split = accepted_encodings.split(",");
            loop {
                match split.next() {
                    None => {
                        break;
                    }
                    Some(enc) => encodings.push(enc.trim()),
                }
            }
            return encodings;
        }

        vec![]
    }

    pub fn body(&self) -> Option<String> {
        self.body.clone()
    }
}

impl HttpMethod {
    pub fn from_str(str: &str) -> HttpMethod {
        match str {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "DELETE" => HttpMethod::DELETE,
            "PUT" => HttpMethod::PUT,
            "PATCH" => HttpMethod::PATCH,
            "OPTIONS" => HttpMethod::OPTIONS,
            "QUERY" => HttpMethod::QUERY,
            "HEAD" => HttpMethod::HEAD,
            "TRACE" => HttpMethod::TRACE,
            "CONNECT" => HttpMethod::CONNECT,
            _ => HttpMethod::GET,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::PUT => "PUT",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::QUERY => "QUERY",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::TRACE => "TRACE",
            HttpMethod::CONNECT => "CONNECT",
        }
    }
}

pub enum HttpRequestParseError {
    EmptyRequest,
    InvalidRequest,
    InvalidHeader,
    InvalidBody,
    Unknown,
}
