use actix_web::{web,HttpRequest,Responder};
use crate::{
    enums::ApiResult,
    types::{ApiResponse,AppState,permissions::WereChecked}
};
use super::sessions_post::{AccessToken, CreateSessionBody, SessionsPost};
use super::sessions_delete::SessionsDelete;

#[utoipa::path(
    post,
    path = "/v1/sessions",
    operation_id = "createSession",
    tags = ["sessions"],
    security(), 
    request_body = CreateSessionBody,
    responses(
        (
            status = 200, description = "OK", body = ApiResult<ApiResponse<AccessToken>>,
            example = json!({ "Ok": {"code":200,"message":"OK","data":{"access_token":"eyJ..."}}})
        ),
        (status = 401, description = "Unauthorized")
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
        (status = 201, description = "resource created"),
        (status = 401, description = "Unauthorized"),
        (
            status = 500, description = "Internal server error",
            example = json!({ "Error": {"code":500,"message":"internal server error"}})
        )
    )
)]

pub async fn delete_sessions(_permissions: WereChecked, req: HttpRequest, shared: web::Data<AppState>) -> impl Responder {
    SessionsDelete::logic(_permissions, req, shared).await
}

#[cfg(test)]
mod spec_tests {
    use serde_json::Value;
    use utoipa::OpenApi;
    use super::*;

    /// Build an OpenAPI doc from THESE operations and the concrete schemas they use.
    #[derive(OpenApi)]
    #[openapi(
        paths(
            super::post_sessions,
            super::delete_sessions, // include DELETE so it's in the spec
        ),
        components(schemas(
            CreateSessionBody,
            AccessToken,
            ApiResponse<AccessToken>
        )),
        tags((name = "Sessions", description = "Session endpoints"))
        // If you set top-level security in your real doc, it's fine; these tests
        // assert per-operation overrides/requirements below.
    )]
    struct ApiDoc;

    #[test]
    fn post_sessions_is_in_spec_with_correct_operation_id_and_body_ref() {
        let v: Value = serde_json::to_value(ApiDoc::openapi()).unwrap();
        let post = &v["paths"]["/v1/sessions"]["post"];
        assert!(post.is_object(), "POST /v1/sessions not found");
        assert_eq!(post["operationId"], "createSession");

        // request body points to CreateSessionBody
        assert_eq!(
            post["requestBody"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/CreateSessionBody",
            "request body should be CreateSessionBody"
        );

        // 200 response uses ApiResponse<AccessToken>
        let ok_ref = post["responses"]["200"]["content"]["application/json"]["schema"]["$ref"]
            .as_str()
            .expect("200 response should be a $ref");
        assert!(
            ok_ref.contains("ApiResponse") && ok_ref.contains("AccessToken"),
            "expected 200 body to be ApiResponse<AccessToken>, got {ok_ref}"
        );

        // AccessToken has access_token field
        let token_props = &v["components"]["schemas"]["AccessToken"]["properties"];
        assert!(token_props.get("access_token").is_some(), "access_token property missing");
    }

    #[test]
    fn post_sessions_is_public_security_override() {
        let v: Value = serde_json::to_value(ApiDoc::openapi()).unwrap();
        let sec = &v["paths"]["/v1/sessions"]["post"]["security"];
        // security: [] means explicitly no auth required
        assert!(sec.is_array(), "expected security array on POST /v1/sessions");
        assert!(
            sec.as_array().unwrap().is_empty(),
            "expected POST /v1/sessions to override security with an empty array"
        );
    }

    #[test]
    fn delete_sessions_is_in_spec_with_correct_operation_id_and_no_content_body() {
        let v: Value = serde_json::to_value(ApiDoc::openapi()).unwrap();
        let del = &v["paths"]["/v1/sessions"]["delete"];
        assert!(del.is_object(), "DELETE /v1/sessions not found");
        assert_eq!(del["operationId"], "deleteSession");

        // 201 is present and MUST NOT have a content schema
        let resp_201 = &del["responses"]["201"];
        assert!(resp_201.is_object(), "expected 201 response object");
        let content = &resp_201["content"];
        assert!(
            content.is_null(), // if the key is absent, serde_json gives Null
            "201 'resource created' must not define a content schema"
        );
    }

    #[test]
    fn delete_sessions_requires_bearer_auth() {
        let v: Value = serde_json::to_value(ApiDoc::openapi()).unwrap();
        let sec = &v["paths"]["/v1/sessions"]["delete"]["security"];
        assert!(sec.is_array(), "expected security array on DELETE /v1/sessions");
        let arr = sec.as_array().unwrap();
        assert!(
            !arr.is_empty(),
            "expected DELETE /v1/sessions to require auth (bearerAuth)"
        );
        // Look for an object like { "bearerAuth": [] }
        let has_bearer = arr.iter().any(|entry| {
            entry.get("bearerAuth").map(|v| v.is_array()).unwrap_or(false)
        });
        assert!(has_bearer, "expected bearerAuth requirement on DELETE /v1/sessions");
    }
}