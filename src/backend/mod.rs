pub(crate) mod chatgpt;
pub(crate) mod gemini;

use std::fmt::{Debug, Display, Formatter};

use async_trait::async_trait;
use reqwest::StatusCode;
use tracing::error;

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

#[async_trait]
pub(crate) trait Backend: Send + Sync {
    async fn generate_content(
        &self,
        prompt: Vec<(String, String)>,
    ) -> Result<String, Error>;
}
