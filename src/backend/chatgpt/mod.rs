mod model;

use std::fmt::{Debug, Display, Formatter};
use async_trait::async_trait;
use reqwest::StatusCode;
use tracing::error;

use crate::backend::{Backend, Error};
use crate::backend::chatgpt::model::{Content, Input, InputMessage, Request, Response};

const BASE_URL: &str = "https://api.openai.com/v1/responses";
const DEFAULT_MODEL: &str = "gpt-3.5-turbo";

pub struct ChatGpt {
    api_key: String,
    model: String,
}

impl ChatGpt {
    pub(crate) fn new(api_key: &str) -> Self {
        ChatGpt {
            api_key: api_key.to_string(),
            model: DEFAULT_MODEL.to_string(),
        }
    }

    fn build_request(&self, prompt: Vec<(String, String)>) -> Request {
        let mut input = Vec::new();

        for (role, text) in prompt.into_iter() {
            let role = if role == "model" {
                "assistant".to_string()
            } else {
                role
            };
            input.push(Input::Message(InputMessage {
                content: text,
                role: role,
            }));
        }

        Request {
            model: self.model.clone(),
            input,
        }
    }
}

#[async_trait]
impl Backend for ChatGpt {
    async fn generate_content(
        &self,
        prompt: Vec<(String, String)>,
    ) -> Result<String, Error> {
        let client = reqwest::Client::new();

        let full_url = format!("{}", BASE_URL);

        let request = self.build_request(prompt);

        let Ok(request_str) = serde_json::to_string(&request) else {
            error!("Couldn't serialise request: {:?}", request);
            return Err(Error::BadRequest);
        };

        let response = client
            .post(full_url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
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

        println!("REQUEST\n{}", request_str);
        println!("RESPONSE\n{}", text);

        let Ok(response) = serde_json::from_str::<Response>(&text) else {
            error!("Bad response JSON: {}", text);
            return Err(Error::BadResponse);
        };

        let Content::OutputText { ref text } = response.output[0].content[0]
        else { panic!() };

        Ok(text.clone())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_build_request() {
        let chatgpt = ChatGpt::new("");
        let prompt = vec![("role1".to_string(), "text1".to_string())];
        let request = chatgpt.build_request(prompt);

        let json = serde_json::to_string(&request).unwrap();

        assert_eq!(
            "{\"model\":\"gpt-3.5-turbo\",\"input\":[{\"type\":\"message\",\"content\":\"text1\",\"role\":\"role1\"}]}",
            json
        );
    }
}
