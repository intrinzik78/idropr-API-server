use actix_web::{web,Responder};
use serde::Serialize;
use utoipa::ToSchema;
use crate::{
    enums::ApiResult,
    types::{ApiErrorData,AppState,permissions::WereChecked}
};
use super::sessions::{NewScanSession,ScanSessionPost};

#[utoipa::path(
    post,
    path = "/v1/extractions/sessions",
    operation_id = "createScanSession",
    tags = ["extractions","scans"],
    security(("bearerAuth" = [])),
    responses(
        (
            status = 201, description = "resource created", body = ApiResultNewScanSession,
            example = json!({ "Ok": {"code":201,"message":"OK","data":{"session_id":123}}})
        ),
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
                ("server_error" = (value = json!({ "Error": { "code": 500, "message":"server error"} }))),
                ("server_error_with_data" = (value = json!({ "Error": { "code": 500, "message":"server error", "data":{ "code": 1011, "reason": "server error, data was not saved, try again"}} })))
            )
        ),
    )
)]

pub async fn post_extraction_session(permissions: WereChecked, shared: web::Data<AppState>) -> impl Responder {
    create_scan_session(permissions,shared).await
}

pub async fn create_scan_session(permissions: WereChecked, shared: web::Data<AppState>) -> impl Responder {
    ScanSessionPost::private_sessions_response(permissions, shared).await
}

#[derive(Serialize,ToSchema)]
pub struct ApiResultNewScanSession(#[schema(inline)] pub ApiResult<NewScanSession>);

#[derive(Serialize, ToSchema)]
pub struct ApiResultError(#[schema(inline)] pub ApiResult<ApiErrorData>);