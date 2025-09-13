use actix_web::{web, Responder};
use crate::types::{ApiResponse,AppState};
use super::sessions_post::{AccessToken, CreateSessionBody, SessionsPost};

#[utoipa::path(
    post,
    path = "/v1/sessions",
    operation_id = "createSession",
    tags = ["session"],
    // If your DTOs implement ToSchema, you can reference them by name here:
    request_body = CreateSessionBody,
    responses(
        (
            status = 200, description = "OK", body = ApiResponse<AccessToken>,
            example = json!({"code":200,"message":"OK","data":{"access_token":"eyJ..."}})
        ),
        (status = 401, description = "Unauthorized")
    )
)]

pub async fn post_sessions(post: web::Json<CreateSessionBody>, shared: web::Data<AppState>) -> impl Responder {
    SessionsPost::logic(post, shared).await
}

#[cfg(test)]
mod spec_tests {
    use serde_json::Value;
    use utoipa::OpenApi;

    // Build an OpenAPI doc from THIS wrapper function and the concrete schemas it uses.
    #[derive(OpenApi)]
    #[openapi(
        // reference the wrapper function item we just defined
        paths(super::post_sessions),
        // list the concrete schemas that appear in the operation
        components(schemas(
            // request type
            super::super::sessions_post::CreateSessionBody,
            // response inner payload
            super::super::sessions_post::AccessToken,
            // the generic envelope specialized with AccessToken
            crate::types::ApiResponse<super::super::sessions_post::AccessToken>
        )),
        tags((name = "session", description = "Session endpoints"))
    )]
    struct ApiDoc;

    #[test]
    fn post_sessions_is_in_spec() {
        let v: Value = serde_json::to_value(ApiDoc::openapi()).unwrap();
        let op = &v["paths"]["/v1/sessions"]["post"];
        assert!(op.is_object(), "POST /v1/sessions not found");
        assert_eq!(op["operationId"], "postSession");
        assert_eq!(
            op["requestBody"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/Post",  // replace with '#/components/schemas/SessionLogin' if you renamed it
            "request body should be the login payload"
        );
    }

    #[test]
    fn response_references_apiresponse_of_accesstoken() {
        let v: Value = serde_json::to_value(ApiDoc::openapi()).unwrap();
        let ok_ref = v["paths"]["/v1/sessions"]["post"]["responses"]["200"]["content"]
            ["application/json"]["schema"]["$ref"]
            .as_str()
            .expect("200 response should be a $ref");

        // Utoipa names generics like 'ApiResponse_AccessToken'. Be tolerant of exact spelling.
        assert!(
            ok_ref.contains("ApiResponse") && ok_ref.contains("AccessToken"),
            "expected 200 body to be ApiResponse<AccessToken>, got {ok_ref}"
        );

        // Also verify AccessToken has the access_token field
        let token_props = &v["components"]["schemas"]["AccessToken"]["properties"];
        assert!(token_props.get("access_token").is_some(), "access_token property missing");
    }
}