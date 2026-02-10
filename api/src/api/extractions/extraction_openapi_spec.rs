use actix_multipart::Multipart;
use actix_web::{web,Responder};
use actix_web::web::{Data, Path as ActixPath};
use serde::Serialize;
use utoipa::ToSchema;
use crate::enums::ErrorReason;
use crate::{
    enums::ApiResult,
    types::{ApiError,AppState,permissions::WereChecked}
};
use super::sessions::{NewScanSession,ScanSessionPost,BatchIngestResponse,ReqPath};

#[utoipa::path(
    post,
    path = "/v1/extractions/sessions",
    operation_id = "createScanSession",
    tags = ["extractions","scans"],
    security(("bearerAuth" = [])),
    responses(
        (
            status = 201, description = "resource created", body = ApiResultNewScanSession,
            example = json!({ "Ok": {"code":201,"message":"resource created","data":{"session_id":123}}})
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


#[utoipa::path(
    post,
    path = "/v1/extractions/sessions/{session_id}/batch",
    operation_id = "uploadScanBatch",
    tags = ["extractions","scans"],
    security(("bearerAuth" = [])),
    params(
        ("session_id" = i64, Path, description = "Scan session id")
    ),
    request_body(
        content_type = "multipart/form-data",
        description = "Upload one or more files as repeated `file` parts. Session id is provided in the path."
    ),
    responses(
        (
            status = 201,
            description = "files uploaded",
            content_type = "application/json",
            body = ApiResultBatchIngestResponse,
            example = json!({
                "Ok": {
                    "code": 201,
                    "message": "resource created",
                    "data": {
                        "session_id": 123,
                        "uploaded": [
                            {
                                "client_file_name": "scan_001.jpg",
                                "id": 1001,
                                "deduped": false,
                                "bytes": 48123,
                                "mime_type": "image/jpeg"
                            },
                            {
                                "client_file_name": "scan_002.jpg",
                                "id": 1002,
                                "deduped": true,
                                "bytes": 51234,
                                "mime_type": "image/jpeg"
                            }
                        ]
                    }
                }
            })
        ),
        (
            status = 400,
            description = "bad request (multipart missing fields / too many parts / session not found / invalid file)",
            content_type = "application/json",
            body = ApiResultError,
            examples(
                ("missing_field_name" = (value = json!({ "Error": { "code": 400, "message": "bad request", "data": { "code": 1014, "reason": "upload aborted, cannot parse multi-part upload, it did not have a field name indicated  in the payload" } } }))),
                ("missing_file_data"  = (value = json!({ "Error": { "code": 400, "message": "bad request", "data": { "code": 1015, "reason": "upload failed, one or more uploaded files had no data, 0 byte sized files are rejected" } } }))),
                ("missing_file_ext"   = (value = json!({ "Error": { "code": 400, "message": "bad request", "data": { "code": 1016, "reason": "upload failed, one or more uploaded files was missing a file extension" } } }))),
                ("missing_file_name"  = (value = json!({ "Error": { "code": 400, "message": "bad request", "data": { "code": 1017, "reason": "upload failed, one or more uploaded files had an empty filename" } } }))),
                ("too_many_parts"     = (value = json!({ "Error": { "code": 400, "message": "bad request", "data": { "code": 1020, "reason": "upload failed, batch size too large (max=25 files per upload)" } } })))
            )
        ),
        (
            status = 401,
            description = "unauthorized",
            content_type = "application/json",
            body = ApiResultError,
            example = json!({ "Error": { "code": 401, "message": "Unauthorized" } })
        ),
        (
            status = 404,
            description = "not found (session id)",
            content_type = "application/json",
            body = ApiResultError,
            examples(
                ("session_not_found"  = (value = json!({ "Error": { "code": 404, "message": "not found", "data": { "code": 1022, "reason": "scan session not found" } } })))
            )
        ),
        (
            status = 413,
            description = "payload too large (per-file or total)",
            content_type = "application/json",
            body = ApiResultError,
            examples(
                ("total_too_large" = (value = json!({ "Error": { "code": 413, "message": "payload too large", "data": { "code": 1018, "reason": "upload aborted, total upload size of file(s) was too large" } } }))),
                ("file_too_large"  = (value = json!({ "Error": { "code": 413, "message": "payload too large", "data": { "code": 1019, "reason": "upload aborted, one or more uploaded files exceeded the upload limit" } } })))
            )
        ),
        (
            status = 429,
            description = "rate limited",
            content_type = "application/json",
            body = ApiResultError,
            examples(
                ("global_rate_limit" = (value = json!({ "Error": { "code": 429, "message":"rate limited"} })))
            )
        ),
        (
            status = 500,
            description = "server error",
            content_type = "application/json",
            body = ApiResultError,
            examples(
                ("server_error" = (value = json!({ "Error": { "code": 500, "message":"server error"} })))
            )
        ),
    )
)]
pub async fn post_extraction_session_batch_upload(
    permissions: WereChecked,
    path: ActixPath<ReqPath>,
    payload: Multipart,
    shared: Data<AppState>,
) -> impl Responder {
    ScanSessionPost::private_mulitpart_upload_response(permissions, path, payload, shared).await
}

#[derive(Serialize, ToSchema)]
pub struct ApiResultBatchIngestResponse(#[schema(inline)] pub ApiResult<BatchIngestResponse>);



#[utoipa::path(
    post,
    path = "/v1/extractions/sessions/{session_id}/process",
    operation_id = "processScanSession",
    tags = ["extractions","scans"],
    security(("bearerAuth" = [])),
    params(
        ("session_id" = i64, Path, description = "Scan session id to enqueue for processing")
    ),
    responses(
        (
            status = 202,
            description = "processing queued / accepted",
            content_type = "application/json",
            body = ApiResultProcessing,
            example = json!({ "Ok": { "code": 202, "message": "processing" } })
        ),
        (
            status = 401,
            description = "unauthorized",
            content_type = "application/json",
            body = ApiResultError,
            example = json!({ "Error": { "code": 401, "message": "Unauthorized" } })
        ),
        (
            status = 404,
            description = "not found (session id)",
            content_type = "application/json",
            body = ApiResultError,
            example = json!({
                "Error": {
                    "code": 404,
                    "message": "not found",
                    "data": { "code": 1022, "reason": "scan session not found" }
                }
            })
        ),
        (
            status = 429,
            description = "rate limited",
            content_type = "application/json",
            body = ApiResultError,
            examples(
                ("global_rate_limit" = (value = json!({ "Error": { "code": 429, "message":"rate limited"} })))
            )
        ),
        (
            status = 500,
            description = "server error",
            content_type = "application/json",
            body = ApiResultError,
            examples(
                ("server_error" = (value = json!({ "Error": { "code": 500, "message":"server error"} })))
            )
        )
    )
)]
pub async fn post_extraction_session_process(
    permissions: WereChecked,
    path: ActixPath<ReqPath>,
    shared: Data<AppState>,
) -> impl Responder {
    ScanSessionPost::private_process_response(permissions, path, shared).await
}


#[derive(Serialize, ToSchema)]
pub enum ApiResultError { Error(ApiError<ErrorReason>) }


#[derive(Serialize, ToSchema)]
pub struct NoData {}

#[derive(Serialize, ToSchema)]
pub struct ApiResultProcessing(#[schema(inline)] pub ApiResult<NoData>);
