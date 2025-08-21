use actix_web::{http::StatusCode, HttpResponse };
use serde::Serialize;

use crate::enums::ApiResult;

#[derive(Debug,Serialize)]
pub struct ApiResponse <T> 
where T: Serialize
{
     code: u16,
     message: String,
     
     #[serde(skip_serializing_if="Option::is_none")]
      data: Option<T>
}

impl<T> ApiResponse <T> 
where T:Serialize
{
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

    /// consume self and turn it into an HTTP response
    pub fn ok(self) -> HttpResponse {
        // build status code object
        let status = StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        // build HttpResponse
        HttpResponse::build(status).json(ApiResult::Ok(self))
    }

    /// consume self and turn it into an HTTP response
    pub fn error(self) -> HttpResponse {
        // build status code object
        let status = StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        // build and JSON-encode
        HttpResponse::build(status).json(ApiResult::Error(self))
    }

}

impl <T> Default for ApiResponse <T>
where T: Serialize
{
    fn default() -> Self {
        let code: u16 = 200;
        let message: String = String::from("ok");
        let data = None;
        
        ApiResponse { code, message, data }
    }
}

/// commonly used and standard response types
impl ApiResponse<()> {

    /// 200,OK success response shortcut
    pub fn success() -> HttpResponse {
        ApiResponse::<()>::default().ok()
    }

    /// standard unauthorized response
    pub fn unauthorized() -> Self {
        ApiResponse::default()
            .with_code(401)
            .with_message("unauthorized".to_string())
    }
    
    /// standard bad_request response
    pub fn bad_request() -> Self {
        ApiResponse::default()
            .with_code(400)
            .with_message("bad request".to_string())
    }

    /// standard forbidden response
    pub fn forbidden() -> Self {
        ApiResponse::default()
            .with_code(403)
            .with_message("forbidden".to_string())
    }

    /// standard 404 / not found response
    pub fn not_found() -> Self {
        ApiResponse::default()
            .with_code(404)
            .with_message("not found".to_string())
    }

    /// standard no-content response
    pub fn no_content() -> Self {
        ApiResponse::default()
            .with_code(204)
            .with_message("no content".to_string())
    }

    /// standard rate-limited response
    pub fn rate_limited() -> Self {
        ApiResponse::default()
            .with_code(429)
            .with_message("too many requests".to_string())
    }

    /// standard 500 server error response
    pub fn server_error() -> Self {
        ApiResponse::default()
            .with_code(500)
            .with_message("internal server error".to_string())
    }
}