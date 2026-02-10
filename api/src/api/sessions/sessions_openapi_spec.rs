use actix_web::{web,HttpRequest,Responder};
use serde::Serialize;
use utoipa::ToSchema;
use crate::{
    enums::{ApiResult, ErrorReason},
    types::{ApiError, AppState, permissions::WereChecked}
};
use super::sessions_post::{AccessToken, CreateSessionBody, SessionsPost};
use super::sessions_delete::SessionsDelete;

#[utoipa::path(
    post,
    path = "/v1/sessions",
    operation_id = "createSession",
    tags = ["sessions"],
    security([]), 
    request_body = CreateSessionBody,
    responses(
        (
            status = 200, description = "OK", body = ApiResultToken,
            example = json!({ "Ok": {"code":200,"message":"OK","data":{"access_token":"eyJ..."}}})
        ),
        (status = 429, description = "rate limited",
            content_type = "application/json",
            body = ApiResultError,
            examples(
                ("global_rate_limit" = (value = json!({ "Error": { "code": 429, "message":"rate limited"} })))
            )

        ),
        (
            status = 401, description = "unauthorized",
            content_type = "application/json",
            body = ApiResultError,
            example = json!({ "Error": { "code": 401, "message": "Unauthorized" } })
        ),
        (status = 403, description = "forbidden")
    )
)]

pub async fn post_sessions(post: web::Json<CreateSessionBody>, shared: web::Data<AppState>) -> impl Responder {
    SessionsPost::logic(post, shared).await
}

#[utoipa::path(
    delete,
    path = "/v1/sessions",
    operation_id = "deleteSession",
    tags = ["sessions"],
    security(("bearerAuth" = [])),
    responses(
        (status = 204, description = "no content"),
        (
            status = 401, description = "unauthorized",
            content_type = "application/json",
            body = ApiResultError,
            example = json!({ "Error": { "code": 401, "message": "Unauthorized" } })
        ),
        (status = 429, description = "rate limited",
            content_type = "application/json",
            body = ApiResultError,
            examples(
                ("global_rate_limit" = (value = json!({ "Error": { "code": 429, "message":"rate limited"} })))
            )

        ),
        (status = 500, description = "server error",
            content_type = "application/json",
            body = ApiResultError,
            examples(
                ("server_error" = (value = json!({ "Error": { "code": 500, "message":"server error"} })))
            )

        ),
    )
)]

pub async fn delete_sessions(_permissions: WereChecked, req: HttpRequest, shared: web::Data<AppState>) -> impl Responder {
    SessionsDelete::logic(_permissions, req, shared).await
}

#[derive(Serialize, ToSchema)]
pub struct ApiResultToken(#[schema(inline)] pub ApiResult<AccessToken<'static>>);

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiResultError {
    #[serde(rename = "Error")]
    pub error: ApiError<ErrorReason>
}