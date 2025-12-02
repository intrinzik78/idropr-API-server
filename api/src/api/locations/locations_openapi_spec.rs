use actix_web::{HttpRequest, Responder, web::{Data, Path, Query}};
use serde::Serialize;
use utoipa::ToSchema;
use crate::{
    api::locations::LocationsGet,
    enums::{ActivityType,ApiResult,Error},
    types::{ApiErrorData, ApiResponse, AppState, permissions::WereChecked}
};
use super::locations_get::{PublicLocation, PrivateLocation, GetReqPath, GetReqParams};

#[utoipa::path(
    get,
    path = "/v1/locations/{id}/private",
    operation_id = "getPrivateLocationById",
    security(("bearerAuth" = [])),
    tags = ["locations"],
    params( ("id" = i64, Path, description = "Location ID") ),
    responses(
        (
            status = 200, description = "OK",
            body = ApiResultPrivateLocation,
            example = json!({ "Ok": { "code": 200, "message": "OK", "data": { "address_1": "123 Some Road..." } } })
        ),
        (
            status = 401, description = "unauthorized",
            content_type = "application/json",
            body = ApiResultError,
            example = json!({ "Error": { "code": 401, "message": "Unauthorized" } })
        ),
        (
            status = 403, description = "forbidden",
            content_type = "application/json",
            body = ApiResultError,
            examples(
                ("insufficient_permissions" = (value = json!({ "Error": { "code": 403, "message": "forbidden", "data": { "code": 1009, "reason": "insufficient permissions for requested resource" } }})))
            )
        ),
        (
            status = 404, description = "not found",
            content_type = "application/json",
            body = ApiResultError,
            examples(
                ("not_found" = (value = json!({ "Error": { "code": 404, "message": "not found", "data": { "code": 1008, "reason": "resource does not exist" } }})))

            )
        ),
        (
            status = 429, description = "rate limited",
            content_type = "application/json",
            body = ApiResultError,
            examples(
                ("global_rate_limit" = (value = json!({ "Error": { "code": 429, "message":"rate limited"} }))),
                ("verification_rate_limit" = (value = json!({ "Error": { "code": 429, "message":"rate limited", "data":{"code": 1003, "reason": "new verification requested too quickly"}} })))
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
pub async fn get_private_location_by_id(_permissions: WereChecked,req: HttpRequest,path: Path<GetReqPath>,shared: Data<AppState>) -> impl Responder {
    LocationsGet::private_response(_permissions, req, path, shared).await
}

#[utoipa::path(
    get,
    path = "/v1/locations/{id}",
    operation_id = "getPublicLocationById",
    tags = ["locations"],
    params( ("id" = i64, Path, description = "Location ID") ),
    security([]),
    responses(
        (status = 200, description = "OK",
            body = ApiResultPublicLocation,
            example = json!({"Ok":{"code":200,"message":"OK","data":{"address_1":"123 Some Road..."}}})
        ),
        (
            status = 403, description = "forbidden",
            content_type = "application/json",
            body = ApiResultError,
            examples(
                ("insufficient_permissions" = (value = json!({ "Error": { "code": 403, "message": "forbidden", "data": { "code": 1009, "reason": "insufficient permissions for requested resource" } }})))
            )
        ),
        (status = 404, description = "not found",
            content_type = "application/json",
            body = ApiResultError,
            examples(
              ("not_found" = (value = json!({"Error":{"code":404,"message":"not found","data":{"code":1008,"reason":"resource does not exist"}}})))
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
pub async fn get_public_location_by_id(path: Path<GetReqPath>,shared: Data<AppState>) -> impl Responder {
    LocationsGet::public_response(path, shared).await
}


#[utoipa::path(
    get,
    path = "/v1/locations",
    operation_id = "listNearestPublicLocationByZipcode",
    tags = ["locations"],
    params(
        ("nearest_zipcode" = Option<String>, Query,
            description = "zipcode string; required for zipcode-based nearest search.",
            example = "77002",
            pattern = r"^[0-9]{5}(?:-[0-9]{4})?$"
        ),
        ("lat" = Option<f32>, Query,
            description = "latitude coordinate; lon also required for coordinate-based nearest search.",
            example = 29.7604
        ),
        ("lon" = Option<f32>, Query,
            description = "longitude coordinate; lat also required for coordinate-based nearest search.",
            example = -95.3698
        ),
        ("activity" = Option<ActivityType>, Query,
            description = "activity filter",
            example = "paintball"
        )
    ),
    security([]),
    responses(
        (
            status = 200, description = "OK",
            body = ApiResultPublicLocationsList,
            example = json!({"Ok":{"code":200,"message":"OK","data":[{"address_1":"123 Some Road..."}]}})
        ),
        (
            status = 400, description = "bad request",
            content_type = "application/json",
            body = ApiResultError,
            examples(
              ("bad_request" = (value = json!({"Error":{"code":400,"message":"bad request","data":{"code":1010,"reason":"missing query parameter"}}})))
            )
        ),
        (
            status = 403, description = "forbidden",
            content_type = "application/json",
            body = ApiResultError,
            examples(
                ("insufficient_permissions" = (value = json!({ "Error": { "code": 403, "message": "forbidden", "data": { "code": 1009, "reason": "insufficient permissions for requested resource" } }})))
            )
        ),
        (status = 404, description = "not found",
            content_type = "application/json",
            body = ApiResultError,
            examples(
              ("not_found" = (value = json!({"Error":{"code":404,"message":"not found","data":{"code":1008,"reason":"resource does not exist"}}})))
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
pub async fn get_public_nearest_locations_by_zipcode(params:Query<GetReqParams>, shared: Data<AppState>) -> impl Responder {
    if params.nearest_zipcode.is_none() {
        let e = Error::MissingLocationQueryParam("zipcode".to_string())
            .to_api_error_message()
            .expect("unreachable");
        
        return ApiResponse::bad_request().with_code(e.code).with_message(e.reason).error();
    }

    // filter on params given
    if params.activity.is_some() {
        LocationsGet::public_nearest_activity_response(params, shared).await
    } else if params.lat.is_some() && params.lon.is_some() {
        todo!()
    } else {
        ApiResponse::server_error().error()
    }
}

#[derive(Serialize, ToSchema)]
pub struct ApiResultPublicLocation(#[schema(inline)] pub ApiResult<PublicLocation>);

#[derive(Serialize, ToSchema)]
pub struct ApiResultPrivateLocation(#[schema(inline)] pub ApiResult<PrivateLocation>);

#[derive(Serialize, ToSchema)]
pub struct ApiResultPublicLocationsList(#[schema(inline)] pub ApiResult<Vec<PublicLocation>>);

#[derive(Serialize, ToSchema)]
pub struct ApiResultError(#[schema(inline)] pub ApiResult<ApiErrorData>);