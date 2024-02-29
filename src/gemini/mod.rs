mod model;

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
        let Ok(result) = model::parse_response(&value) else {
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
}
