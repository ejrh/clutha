mod model;

use std::fmt::{Debug, Display, Formatter};

use reqwest::StatusCode;
use tracing::error;

use crate::gemini::model::*;

const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1";
const DEFAULT_MODEL: &str = "models/gemini-2.5-flash-lite";
const GENERATE_METHOD: &str = "generateContent";

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    SerdeJson(serde_json::Error),
    HttpStatus(StatusCode),
    BadRequest,
    BadResponse,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

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
    model: String,
}

impl Gemini {
    pub(crate) fn new(api_key: &str) -> Self {
        Gemini {
            api_key: api_key.to_string(),
            model: DEFAULT_MODEL.to_string(),
        }
    }

    pub(crate) async fn generate_content(
        &self,
        prompt: Vec<(String, String)>,
    ) -> Result<String, Error> {
        let client = reqwest::Client::new();

        let full_url = format!("{}/{}:{}?key={}", BASE_URL, self.model, GENERATE_METHOD, self.api_key);

        let request = build_request(prompt);

        let Ok(request_str) = serde_json::to_string(&request) else {
            error!("Couldn't serialise request: {:?}", request);
            return Err(Error::BadRequest);
        };

        let response = client
            .post(full_url)
            .body(request_str.clone())
            .send()
            .await?;
        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            error!("Bad HTTP content: {}", text);
            error!("Request was: {}", request_str);
            return Err(Error::HttpStatus(status));
        }

        // println!("REQUEST\n{}", request_str);
        // println!("RESPONSE\n{}", text);

        let Ok(response) = serde_json::from_str::<GenerateContentResponse>(&text) else {
            error!("Bad response JSON: {}", text);
            return Err(Error::BadResponse);
        };

        let text = response.candidates[0].content.parts[0].text.clone();

        Ok(text)
    }
}

fn build_request(prompt: Vec<(String, String)>) -> GenerateContentRequest {
    let mut contents = Vec::new();

    for (role, text) in prompt.into_iter() {
        let part = Part { text };
        let content = Content {
            parts: vec![part],
            role,
        };
        contents.push(content);
    }

    let safety_settings = vec![
        // HarmCategory.HARM_CATEGORY_HATE_SPEECH,
        // HarmCategory.HARM_CATEGORY_SEXUALLY_EXPLICIT,
        // HarmCategory.HARM_CATEGORY_DANGEROUS_CONTENT,
        // HarmCategory.HARM_CATEGORY_HARASSMENT,
        // HarmCategory.HARM_CATEGORY_CIVIC_INTEGRITY,
    ];

    GenerateContentRequest { contents, safety_settings }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_build_request() {
        let prompt = vec![("role1".to_string(), "text1".to_string())];
        let request = build_request(prompt);

        let json = serde_json::to_string(&request).unwrap();

        assert_eq!(
            "{\"contents\":[{\"parts\":[{\"text\":\"text1\"}],\"role\":\"role1\"}]}",
            json
        );
    }
}
