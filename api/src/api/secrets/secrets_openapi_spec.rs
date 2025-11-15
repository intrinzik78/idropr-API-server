use actix_web::{web,Responder};
use serde::Serialize;
use utoipa::ToSchema;
use crate::{
    enums::ApiResult,
    types::{ApiErrorData,AppState,ApiResponse,permissions::WereChecked}
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
        (status = 201, description = "resource created"),
        (status = 401, description = "Unauthorized"),
        (
            status = 400,
            description = "bad request",
            body = ApiResult<ApiResponse<String>>,  // or ApiResponse<ErrorDetail>
            example = json!({"Error": {"code":400,"message":"duplicate: api name already in use"}})
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

pub async fn post_secrets(_permissions: WereChecked, post: web::Json<CreateSecretBody>, shared: web::Data<AppState>) -> impl Responder {
    SecretsPost::logic(_permissions, post, shared).await
}

#[derive(Serialize, ToSchema)]
pub struct ApiResultError(#[schema(inline)] pub ApiResult<ApiErrorData>);