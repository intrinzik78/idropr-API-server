use actix_web::{Responder, web};
use serde::Serialize;
use utoipa::ToSchema;
use crate::{
    enums::ApiResult,
    types::{ApiErrorData, AppState}
};
use super::email_post::{EmailVerificationPost,CreateEmailVerification};
use super::email_patch::{PatchEmailVerification,PatchReqPath};

#[utoipa::path(
    post,
    path = "/v1/verifications/email",
    operation_id = "createEmailVerification",
    tags = ["email","verifications"],
    security([]),
    request_body = EmailVerificationPost,
    responses(
        (status = 201, description = "resource created", body=SuccessMessage),
        (status = 400, description = "bad request",
            content_type = "application/json",
            body = ApiResultError,
            examples(
                ("already_verified" = (value = json!({ "Error": { "code": 400, "message":"bad request", "data":{ "code":1000,"reason": "email service rejected request" }} }))),
                ("provider_rejected" = (value = json!({ "Error": { "code": 400, "message":"bad request", "data":{ "code":1000,"reason": "email service rejected request" }} })))
            )
        ),
        (status = 403, description = "forbidden",
            content_type = "application/json",
            body = ApiResultError,
            example = json!({"Error": { "code": 403, "message":"forbidden", "data":{ "code":1002,"reason": "email address is suppressed"}}})
        ),
        (status = 429, description = "rate limited",
            content_type = "application/json",
            body = ApiResultError,
            examples(
                ("global_rate_limit" = (value = json!({ "Error": { "code": 429, "message":"rate limited"} }))),
                ("verification_rate_limited" = (value = json!({ "Error": { "code": 429, "message":"rate limited", "data":{ "code":1003,"reason": "new verification requested too quickly" }} })))
            )

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
    security([]), 
    params(
        ("uuid" = String, Path, description = "Verification UUID"),
        ("id"   = i64,    Path, description = "Verification ID", format = "int64")
    ),
    operation_id = "patchEmailVerification",
    tags = ["email","verifications"],
    responses(
        (status = 201, description = "resource created", body=SuccessMessage),
        (status = 400, description = "bad request",
            content_type = "application/json",
            body = ApiResultError,
            example = json!({ "already_verified": { "code": 1001, "reason": "email verified, no further action necessary" } }),
        ),
        (
            status = 401, description = "unauthorized",
            content_type = "application/json",
            body = ApiResultError,
            example = json!({ "Error": { "code": 401, "message": "Unauthorized","data":{"code":1007,"reason":"verification failed"}} })
        ),
        (status = 410, description = "gone",
            content_type = "application/json",
            body = ApiResultError,
            examples(
                ("link_expired" = (value = json!({ "Error": { "code": 410, "message":"gone", "data":{ "code":1005,"reason": "verification link has expired" }} }))),
                ("record_not_found" = (value = json!({ "Error": { "code": 410, "message":"gone", "data":{ "code":1006,"reason": "record does not exist" }} })))
            )
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

pub async fn patch_email_verification(path:web::Path<PatchReqPath>, shared: web::Data<AppState>) -> impl Responder {
    PatchEmailVerification::response(path, shared).await
}

#[derive(Serialize, ToSchema)]
pub struct ApiResultError(#[schema(inline)] pub ApiResult<ApiErrorData>);

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct SuccessMessage(#[schema(inline)] pub ApiResult<String>);