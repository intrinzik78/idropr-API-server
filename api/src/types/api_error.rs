use serde::Serialize;
use utoipa::ToSchema;

use crate::enums::ErrorReason;

#[derive(Debug,Serialize,ToSchema)]
pub struct ApiError<ErrorReason>
{
     pub code: u16,
     pub message: String,
     
     #[serde(skip_serializing_if="Option::is_none")]
     pub data: Option<ErrorReason>
}

impl ApiError<ErrorReason> {
      pub fn new(code: u16, message: impl Into<String>) -> Self {
            let data = None;
            Self { code, message: message.into(), data }
      }
    
      pub fn with_data(mut self, d:ErrorReason) -> Self {
            self.data = Some(d);
            self
      }
}