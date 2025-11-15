use actix_web::{http::StatusCode, HttpResponse };
use serde::Serialize;
use utoipa::ToSchema;

use crate::enums::ApiResult;

/// api response wrapper, returning the code,message and optional data
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiResponse <T> 
where T: Serialize + ToSchema
{
     code: u16,
     message: String,
     
     #[serde(skip_serializing_if="Option::is_none")]
      data: Option<T>
}

impl<T> ApiResponse <T> 
where T: Serialize + ToSchema
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
where T: Serialize + ToSchema
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

    /// standard unauthorized response - 401
    pub fn unauthorized() -> Self {
        ApiResponse::default()
            .with_code(401)
            .with_message("unauthorized".to_string())
    }
    
    /// 400, bad request
    pub fn bad_request() -> Self {
        ApiResponse::default()
            .with_code(400)
            .with_message("bad request".to_string())
    }

    /// 403, forbidden
    pub fn forbidden() -> Self {
        ApiResponse::default()
            .with_code(403)
            .with_message("forbidden".to_string())
    }

    /// 410, gone
    pub fn gone() -> Self {
        ApiResponse::default()
            .with_code(410)
            .with_message("gone".to_string())
    }


    /// 404, not found
    pub fn not_found() -> Self {
        ApiResponse::default()
            .with_code(404)
            .with_message("not found".to_string())
    }

    /// 204, success - no content
    pub fn no_content() -> HttpResponse {
        HttpResponse::NoContent().finish()
    }

    /// 201, success - resource created, no content
    pub fn resource_created() -> Self {
        ApiResponse::default()
            .with_code(201)
            .with_message("resource created".to_string())
    }

    /// 202, accepted, processing later, no immediate content
    pub fn processing() -> Self {
        ApiResponse::default()
            .with_code(202)
            .with_message("processing".to_string())
    }

    /// 429, rate limited
    pub fn rate_limited() -> Self {
        ApiResponse::default()
            .with_code(429)
            .with_message("too many requests".to_string())
    }

    /// 500, internal server error response
    pub fn server_error() -> Self {
        ApiResponse::default()
            .with_code(500)
            .with_message("internal server error".to_string())
    }
}