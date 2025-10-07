use serde::Deserialize;

#[derive(Debug,Deserialize)]
pub struct PostmarkResponse {
    #[serde(rename(deserialize = "To"))]
    pub to: String,
    #[serde(rename(deserialize = "SubmittedAt"))]
    pub submitted_at: String,
    #[serde(rename(deserialize = "MessageID"))]
    pub message_id: String,
    #[serde(rename(deserialize = "ErrorCode"))]
    pub error_code: u32,
    #[serde(rename(deserialize = "Message"))]
    pub message: String
}