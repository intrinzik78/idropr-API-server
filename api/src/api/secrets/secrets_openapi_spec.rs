use actix_web::{web::{Data, Json},Responder};
use serde::Serialize;
use utoipa::ToSchema;
use crate::{
    enums::ErrorReason,
    types::{ApiError,AppState, permissions::WereChecked}
};
use super::secrets_post::{SecretsPost,CreateSecretBody};

#[utoipa::path(
    post,
    path = "/v1/secrets",
    operation_id = "createSecret",
    tags = ["secrets"],
    security(("bearerAuth" = [])),
    request_body = CreateSecretBody,
    responses(
        (status = 204,description = "no content"),
        (status = 401, description = "unauthorized",),
        (status = 403, description = "forbidden"),
        (
            status = 404,
            description = "not found",
            content_type = "application/json",
            body = ApiResultError,
            example = json!({"Error": {"code":404,"message":"not found", "data":{ "code": 1008, "reason": "record does not exist"}}})
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
                ("server_error" = (value = json!({ "Error": { "code": 500, "message":"server error"} }))),
                ("server_error_with_data" = (value = json!({ "Error": { "code": 500, "message":"server error", "data":{ "code": 1011, "reason": "server error, data was not saved, try again"}} })))
            )
        ),
    )
)]

pub async fn post_secrets(_permissions: WereChecked, post: Json<CreateSecretBody>, shared: Data<AppState>) -> impl Responder {
    SecretsPost::logic(_permissions, post, shared).await
}

#[derive(Serialize, ToSchema)]
pub enum ApiResultError { Error(ApiError<ErrorReason>) }