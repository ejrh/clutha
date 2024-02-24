use std::fmt::{Debug, Display, Formatter};
use reqwest::StatusCode;
use serde_json::{json, Value};
use crate::gemini::Error::BadResponse;

const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1/models/gemini-pro:generateContent";

#[derive(Debug)]
pub enum Error {
    ReqwestError(reqwest::Error),
    SerdeJsonError(serde_json::Error),
    BadHttpStatus(StatusCode),
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
        Error::ReqwestError(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::SerdeJsonError(value)
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

pub(crate) async fn generate_content(api_key: &str, prompt: &str) -> Result<String, Error> {
    let client = reqwest::Client::new();

    let full_url = format!("{}?key={}", BASE_URL, api_key);

    let payload = json!({
        "contents": [
            {
                "parts": [
                    { "text": prompt }
                ]
            }
        ]
    });

    let response = client.post(full_url)
        .body(payload.to_string())
        .send()
        .await?;
    let status = response.status();
    let text = response.text().await?;

    if !status.is_success() {
        return Err(Error::BadHttpStatus(status))
    }

    let value: Value = serde_json::from_str(&text)?;
    let Ok(result) = parse_response(&value) else {
        println!("Bad reponse JSON: {}", text);
        return Err(BadResponse)
    };

    let text = result.candidates[0].content.parts[0].text.clone();

    Ok(text)
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
