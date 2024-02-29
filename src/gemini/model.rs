use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Part {
    pub(crate) text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Content {
    pub(crate) parts: Vec<Part>,
    pub(crate) role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Candidate {
    pub(crate) content: Content,
    // finish reason
    // index
    // safety ratings
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct GenerateContentResponse {
    pub(crate) candidates: Vec<Candidate>,
    // prompt feedback; safety ratings
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct GenerateContentRequest {
    pub(crate) contents: Vec<Content>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_response() {
        let response_str = r#"{ "candidates": [ { "content": { "parts": [ { "text": "Hello" } ], "role": "model" }, "finishReason": "STOP" } ] }"#;

        let response = serde_json::from_str::<GenerateContentResponse>(&response_str).unwrap();

        assert_eq!(1, response.candidates.len());
        let cand = &response.candidates[0];
        assert_eq!("model", cand.content.role);
        assert_eq!(1, cand.content.parts.len());
        assert_eq!("Hello", cand.content.parts[0].text);
    }
}
