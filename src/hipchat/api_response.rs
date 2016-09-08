use super::ApiRequest;
use serde_json;
use std::collections::BTreeMap;
use hyper;
use std::io::Read;

pub struct ApiResponse {
    request: ApiRequest,
    content: String,
    parsed_response: ParsedResponse,
}

#[derive(Deserialize)]
struct ParsedResponse {
    links: BTreeMap<String, String>,
    maxResults: u32,
    startIndex: u32,
}

impl ApiResponse {
    pub fn new(request: ApiRequest, mut res: hyper::client::Response) -> ApiResponse {
        let mut content = String::new();
        let size_read = res.read_to_string(&mut content);
        let decoded: ParsedResponse = serde_json::from_str(&content).unwrap();
        ApiResponse {
            request: request,
            content: content,
            parsed_response: decoded,
        }
    }

    pub fn get_next_request(&self) -> Option<ApiRequest> {
        let next: Option<&String> = self.parsed_response.links.get("next");
        next.map(|url| {
            ApiRequest::from_past_request(url.clone(), &self.request)
        })
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }
}
