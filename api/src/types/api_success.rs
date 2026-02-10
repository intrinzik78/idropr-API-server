use serde::Serialize;
use utoipa::ToSchema;

/// api response wrapper, returning the code,message and optional data
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiSuccess <T> 
where T: Serialize + ToSchema
{
     pub code: u16,
     pub message: String,
     
     #[serde(skip_serializing_if="Option::is_none")]
      pub data: Option<T>
}

impl<T> ApiSuccess <T> 
where T: Serialize + ToSchema
{
    pub fn new(code: u16, message: impl Into<String>) -> Self {
        let data = None;
        Self { code, message: message.into(), data }
    }

    /// set custom code on response
    pub fn with_code(mut self, new_code: u16) -> Self {
        self.code = new_code;

        self
    }

    /// set custom message on response
    pub fn with_message(mut self, new_message: String) -> Self {
        self.message = new_message;

        self
    }

    /// include a serialized data body
    pub fn with_data(mut self, new_data:T) -> Self {
        self.data = Some(new_data);

        self
    }
}

