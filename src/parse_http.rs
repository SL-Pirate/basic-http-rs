use http_parser::{CallbackResult, HttpParserCallback, HttpParserType, ParseAction};
use std::borrow::Cow;
use std::collections::HashMap;

pub struct HttpParser {
    path: Option<String>,
}

pub struct HttpResponse {
    status_code: u16,
    status: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

const CONTENT_LENGTH_HEADER: &str = "Content-Length";
const CONNECTION_HEADER: &str = "Connection";

impl HttpResponse {
    pub fn ok(headers: HashMap<String, String>, body: Option<String>) -> HttpResponse {
        HttpResponse {
            status: "OK".to_string(),
            status_code: 200,
            headers,
            body,
        }
    }

    pub fn new(
        status_code: u16,
        status: String,
        headers: HashMap<String, String>,
        body: Option<String>,
    ) -> HttpResponse {
        HttpResponse {
            status_code,
            status,
            headers,
            body,
        }
    }

    fn get_content_len(&self) -> usize {
        if let Some(body) = self.body.clone() {
            return body.into_bytes().len();
        }

        0
    }
}

impl HttpParser {
    pub fn new() -> HttpParser {
        HttpParser { path: None }
    }

    pub fn get_path(&mut self, req: Vec<u8>) -> Option<String> {
        let mut request_parser = http_parser::HttpParser::new(HttpParserType::Request);
        request_parser.execute(self, &*req);

        self.path.clone()
    }

    /// Only supports body type string atm
    pub fn parse_response(&mut self, http_response: HttpResponse) -> Vec<u8> {
        let status_code = http_response.status_code;
        let content_len = http_response.get_content_len();
        let status = http_response.status;
        let mut res = format!("HTTP/1.1 {status_code} {status}\r\n");
        for (header, value) in http_response.headers.clone() {
            res.push_str(format!("{header}: {value}\r\n").as_str())
        }
        if !http_response.headers.contains_key(CONTENT_LENGTH_HEADER) {
            res.push_str(format!("{CONTENT_LENGTH_HEADER}: {content_len}\r\n").as_str())
        }
        if !http_response.headers.contains_key(CONNECTION_HEADER) {
            res.push_str(format!("{CONNECTION_HEADER}: close\r\n").as_str())
        }
        res.push_str("\r\n");
        if let Some(body) = http_response.body {
            res.push_str(body.as_str());
        }

        res.into_bytes()
    }
}

impl HttpParserCallback for HttpParser {
    fn on_url(&mut self, _: &mut http_parser::HttpParser, data: &[u8]) -> CallbackResult {
        match String::from_utf8_lossy(data) {
            Cow::Borrowed(val) => {
                self.path = Some(val.to_string());
            }
            Cow::Owned(val) => {
                self.path = Some(val);
            }
        };

        Ok(ParseAction::None)
    }
}
