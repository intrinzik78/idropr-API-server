use actix_web::{web,Responder};
use crate::{
    enums::ApiResult,
    types::{ApiErrorData, ApiResponse, AppState}
};
use super::email_post::{EmailVerificationPost,CreateEmailVerification};

#[utoipa::path(
    post,
    path = "/v1/verifications/email",
    operation_id = "createEmailVerification",
    tags = ["email,verifications"],
    request_body = CreateEmailVerification,
    responses(
        (status = 201, description = "resource created"),
        (status = 400, description = "bad request",
            content_type = "application/json",
            body = ApiResult<ApiResponse<ApiErrorData>>,
            examples(
                ("already_verified" = (value = json!({ "Error": { "code": 1001, "reason": "email verified, no further action necessary" } }))),
                ("provider_rejected" = (value = json!({ "Error": { "code": 1004, "reason": "email service rejected request" } })))
            )
        ),
        (status = 403, description = "forbidden",
            content_type = "application/json",
            body = ApiResult<ApiResponse<ApiErrorData>>,
            example = json!({"email_surpressed": {"code":1002,"reason":"email address is suppressed"}})
        ),
        (status = 429, description = "rate limited",
            content_type = "application/json",
            body = ApiResult<ApiResponse<ApiErrorData>>,
            example = json!({"rate_limited": {"code":1003,"reason":"new verification requested too quickly"}})
        ),
        (status = 500, description = "server error")
    )
)]

pub async fn post_email_verification(post: web::Json<EmailVerificationPost>, shared: web::Data<AppState>) -> impl Responder {
    CreateEmailVerification::response(post, shared).await
}

#[utoipa::path(
    patch,
    path = "/v1/verifications/email/{uuid}/{id}",
    operation_id = "patchEmailVerification",
    tags = ["email,verifications"],
    responses(
        (status = 201, description = "resource created"),
        (status = 400, description = "bad request",
            content_type = "application/json",
            body = ApiResult<ApiResponse<ApiErrorData>>,
            example = json!({ "already_verified": { "code": 1001, "reason": "email verified, no further action necessary" } }),
        ),
        (status = 410, description = "gone",
            content_type = "application/json",
            body = ApiResult<ApiResponse<ApiErrorData>>,
            examples(
                ("link_expired"     = (value = json!({ "Error": { "code": 1005, "reason": "verification link has expired"} }))),
                ("record_not_found" = (value = json!({ "Error": { "code": 1006, "reason": "record does not exist" } }))),
            )
        ),
        (status = 401, description = "unauthorized",
            content_type = "application/json",
            body = ApiResult<ApiResponse<ApiErrorData>>,
            example = json!({"Error": {"code":1007,"reason":"verification failed"}})
        ),
        (status = 500, description = "server error")
    )
)]

pub async fn patch_email_verification(post: web::Json<EmailVerificationPost>, shared: web::Data<AppState>) -> impl Responder {
    CreateEmailVerification::response(post, shared).await
}