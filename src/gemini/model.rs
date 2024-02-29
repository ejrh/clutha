use serde_json::Value;

use crate::gemini::Error;

pub(crate) struct Part {
    pub(crate) text: String,
}

pub(crate) struct Content {
    pub(crate) parts: Vec<Part>,
    role: String,
}

pub(crate) struct Candidate {
    pub(crate) content: Content,
    // finish reason
    // index
    // safety ratings
}

pub(crate) struct Response {
    pub(crate) candidates: Vec<Candidate>,
    // prompt feedback; safety ratings
}

pub(crate) fn parse_response(value: &Value) -> Result<Response, Error> {
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
