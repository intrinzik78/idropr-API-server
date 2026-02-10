use actix_web::{http::StatusCode, HttpResponse };
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    enums::{Error,ErrorReason},
    types::{ApiError, ApiSuccess}
};

type Result<T> = std::result::Result<T,Error>;

#[derive(Serialize, Debug, ToSchema)]
pub enum ApiResult<T>
where T: Serialize + ToSchema
{
    Ok(ApiSuccess<T>),
    Error(ApiError<ErrorReason>)
}

impl<T> ApiResult<T>
where T: Serialize + ToSchema {
    pub fn ok(code:u16, message:impl Into<String>) -> Self {
        let success = ApiSuccess::new(code,message);

        Self::Ok(success)
    }

    pub fn err(code:u16, message:impl Into<String>) -> Self {
        let err: ApiError<ErrorReason> = ApiError::new(code,message);

        Self::Error(err)
    }

    pub fn with_code(self, code:u16) -> Result<ApiResult<T>> {
        match self {
            Self::Ok(s) => {
                let s = s.with_code(code);
                Ok(Self::Ok(s))
            },
            _ => Err(Error::AccountStatusOutOfBounds)
        }
    }

    pub fn with_data(self, d:T) -> Result<Self> {
        match self {
            Self::Ok(s) => {
                let s = s.with_data(d);
                Ok(Self::Ok(s))
            },
            _ => Err(Error::AccountStatusOutOfBounds)
        }
    }

    pub fn resource_created_with_data(d:T) -> Self {
        let success = ApiSuccess::new(201, "resource created").with_data(d);
        Self::Ok(success)
    }

    /// consume self and turn it into an HTTP response
    pub fn to_http(self) -> HttpResponse {
        let code = match &self {
            ApiResult::Ok(s) => s.code,
            ApiResult::Error(e) => e.code,
        };

        let status = StatusCode::from_u16(code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        // honor 204 semantics
        if status == StatusCode::NO_CONTENT {
            return HttpResponse::build(status).finish();
        }

        HttpResponse::build(status).json(self)
    }

    pub fn with_reason(self,reason:ErrorReason) -> Self {
        match self {
            Self::Error(r) => {
                let r = r.with_data(reason);
                Self::Error(r)
            },
            _ => self
        }
    }



}

impl ApiResult<()> {
    pub fn resource_created() -> Self {
        let success = ApiSuccess::new(201, "resource created");
        Self::Ok(success)
    }

    pub fn no_content() -> Self {
        let success = ApiSuccess::new(204, "no content");
        Self::Ok(success)
    }

    pub fn accepted() -> Self {
        let success = ApiSuccess::new(202, "accepted processing");
        Self::Ok(success)
    }

    pub fn success() -> Self {
        let success = ApiSuccess::new(200, "ok");
        Self::Ok(success)
    }
    
    pub fn bad_request() -> Self { Self::Error(ApiError::new(400, "bad request")) }

    pub fn forbidden() -> Self { Self::Error(ApiError::new(403, "forbidden")) }

    pub fn server_error() -> Self { Self::Error(ApiError::new(500,"internal server error")) }

    pub fn gone() -> Self { Self::Error(ApiError::new(410, "gone")) }

    pub fn not_found() -> Self { Self::Error(ApiError::new(404, "not found")) }

    pub fn rate_limited() -> Self { Self::Error(ApiError::new(429, "rate limited")) }

    pub fn too_large() -> Self { Self::Error(ApiError::new(413, "content too large")) }

    pub fn unauthorized() -> Self { Self::Error(ApiError::new(401, "unauthorized")) }
}