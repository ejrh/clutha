use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct InputMessage {
    pub(crate) content: String,
    pub(crate) role: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub(crate) enum Input {
    Message(InputMessage),
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Request {
    pub(crate) model: String,
    pub(crate) input: Vec<Input>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub(crate) enum Content {
    OutputText { text: String },
    Refusal { refusal: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Output {
    //type, id, status
    pub(crate) role: String,
    pub(crate) content: Vec<Content>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ResponseStatus {
    Completed,
    Failed,
    InProgress,
    Cancelled,
    Queued,
    Incomplete,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Response {
    pub(crate) output: Vec<Output>,
    pub(crate) status: ResponseStatus,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_response() {
        let response_str = r#"{
            "status": "completed",
            "error": null,
            "output": [{
                "type": "message",
                "id": "msg_67ccd3acc8d48190a77525dc6de64b4104becb25c45c1d41",
                "status": "completed",
                "role": "assistant",
                "content": [{
                    "type": "output_text",
                    "text": "Hello",
                    "annotations": []
                }]
            }]
        }"#;

        let response = serde_json::from_str::<Response>(&response_str).unwrap();

        assert_eq!(1, response.output.len());
        let output = &response.output[0];
        assert_eq!("assistant", output.role);
        assert_eq!(1, output.content.len());
        let Content::OutputText { ref text} = output.content[0]
        else { panic!() };
        assert_eq!("Hello", text);
    }
}
