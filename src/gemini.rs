use std::fmt::{Debug, Display, Formatter};

use reqwest::StatusCode;
use serde_json::{json, Value};
use tracing::error;

const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1/models/gemini-pro:generateContent";

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    SerdeJson(serde_json::Error),
    HttpStatus(StatusCode),
    BadResponse,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {

}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::Reqwest(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::SerdeJson(value)
    }
}

struct Part {
    text: String,
}

struct Content {
    parts: Vec<Part>,
    role: String,
}

struct Candidate {
    content: Content,
    // finish reason
    // index
    // safety ratings
}

struct Response {
    candidates: Vec<Candidate>,
    // prompt feedback; safety ratings
}

pub struct Gemini {
    api_key: String,
}

impl Gemini {
    pub(crate) fn new(api_key: &str) -> Self {
        Gemini { api_key: api_key.to_string() }
    }

    pub(crate) async fn generate_content(&self, prompt: Vec<(String, String)>) -> Result<String, Error> {
        let client = reqwest::Client::new();

        let full_url = format!("{}?key={}", BASE_URL, self.api_key);

        let request = build_request(prompt);

        let response = client.post(full_url)
            .body(request.to_string())
            .send()
            .await?;
        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            error!("Bad HTTP content: {}", text);
            return Err(Error::HttpStatus(status))
        }

        let value: Value = serde_json::from_str(&text)?;
        let Ok(result) = parse_response(&value) else {
            error!("Bad response JSON: {}", text);
            return Err(Error::BadResponse)
        };

        let text = result.candidates[0].content.parts[0].text.clone();

        Ok(text)
    }
}

fn build_request(prompt: Vec<(String, String)>) -> Value {
    let mut arr = Vec::new();

    for (role, text) in prompt.into_iter() {
        let obj = json!({
            "role": role,
            "parts": [
                { "text": text }
            ]
        });
        arr.push(obj);
    }

    let contents = Value::Array(arr);

    json!({
        "contents": contents
    })
}

fn parse_response(value: &Value) -> Result<Response, Error> {
    let candidates = value.get("candidates").ok_or(Error::BadResponse)?
        .as_array().ok_or(Error::BadResponse)?;
    let candidates: Vec<Candidate> = candidates.iter().map(|c| {
        let content = c.get("content")?;
        let parts = content.get("parts")?
            .as_array()?;
        let parts: Vec<Part> = parts.iter().map(|p| {
            let text = p.get("text")?
                .as_str()?
                .to_string();
            Some(Part { text })
        }).collect::<Option<_>>()?;
        let role = content.get("role")?
            .as_str()?
            .to_string();
        Some(Candidate {
            content: Content { parts, role },
        })
    }).collect::<Option<_>>().ok_or(Error::BadResponse)?;

    Ok(Response {
        candidates,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_build_request() {
        let prompt = vec![("role1".to_string(), "text1".to_string())];
        let request = build_request(prompt);

        let json = request.to_string();

        assert_eq!("{\"contents\":[{\"parts\":[{\"text\":\"text1\"}],\"role\":\"role1\"}]}", json);
    }

    #[test]
    fn test_parse_response() {
        let response_str = r#"{ "candidates": [ { "content": { "parts": [ { "text": "Hello" } ], "role": "model" }, "finishReason": "STOP" } ] }"#;

        let value = serde_json::from_str(&response_str).unwrap();

        let response = parse_response(&value).unwrap();

        assert_eq!(1, response.candidates.len());
        let cand = &response.candidates[0];
        assert_eq!("model", cand.content.role);
        assert_eq!(1, cand.content.parts.len());
        assert_eq!("Hello", cand.content.parts[0].text);
    }
}
