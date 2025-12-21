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
    #[serde(rename = "finishReason")]
    pub(crate) finish_reason: FinishReason,
    #[serde(default)]
    pub(crate) safety_ratings: Vec<SafetyRating>,
    // index
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FinishReason {
    FinishReasonUnspecified,
    Stop,
    MaxTokens,
    Safety,
    Recitation,
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct SafetyRating {
    pub(crate) category: HarmCategory,
    pub(crate) probability: HarmProbability,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum HarmCategory {
    HarmCategoryUnspecified,
    HarmCategoryDerogatory,
    HarmCategoryToxicity,
    HarmCategoryViolence,
    HarmCategorySexual,
    HarmCategoryMedical,
    HarmCategoryDangerous,
    HarmCategoryHarassment,
    HarmCategoryHateSpeech,
    HarmCategorySexuallyExplicit,
    HarmCategoryDangerousContent,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum HarmProbability {
    HarmProbabilityUnspecified,
    Negligible,
    Low,
    Medium,
    High,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct GenerateContentResponse {
    pub(crate) candidates: Vec<Candidate>,
    // prompt feedback; safety ratings
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum HarmBlockThreshold {
    HarmBlockThresholdUnspecified,
    BlockLowAndAbove,
    BlockMediumAndAbove,
    BlockOnlyHigh,
    BlockNone,
    Off,
}
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct SafetySetting {
    pub(crate) category: HarmCategory,
    pub(crate) threshold: HarmBlockThreshold,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct GenerateContentRequest {
    pub(crate) contents: Vec<Content>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) safety_settings: Vec<SafetySetting>,
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

    #[test]
    fn test_parse_safety() {
        let response_str = r#"{ "candidates": [ { "content": { "parts": [ { "text": "Hello" } ], "role": "model" }, "finishReason": "SAFETY" } ] }"#;

        let response = serde_json::from_str::<GenerateContentResponse>(&response_str).unwrap();

        assert_eq!(1, response.candidates.len());
        let cand = &response.candidates[0];
        assert_eq!(FinishReason::Safety, cand.finish_reason);
    }
}
