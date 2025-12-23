pub(crate) mod chatgpt;
pub(crate) mod gemini;

use std::fmt::{Debug, Display, Formatter};

use async_trait::async_trait;
use reqwest::StatusCode;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use tracing::error;
use crate::backend::Error::Reqwest;

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    SerdeJson(serde_json::Error),
    HttpStatus(StatusCode),
    BadRequest,
    BadResponse,
    Other(String),
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

fn get_client() -> ClientWithMiddleware {
    let client = reqwest::Client::new();
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(client)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();
    client
}

fn map_client_error(error: reqwest_middleware::Error) -> Error {
    match error {
        reqwest_middleware::Error::Middleware(err) => Error::Other(format!("{:?}", err)),
        reqwest_middleware::Error::Reqwest(request_error) => Error::Reqwest(request_error),
    }
}
