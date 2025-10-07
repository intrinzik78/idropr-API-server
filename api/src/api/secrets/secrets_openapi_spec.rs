use actix_web::{web,Responder};
use crate::{
    enums::ApiResult,
    types::{AppState,ApiResponse,permissions::WereChecked}
};
use super::secrets_post::{SecretsPost,CreateSecretBody};

#[utoipa::path(
    post,
    path = "/v1/secrets",
    operation_id = "createSecret",
    tags = ["Secrets"],
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
        )
    )
)]

pub async fn post_secrets(_permissions: WereChecked, post: web::Json<CreateSecretBody>, shared: web::Data<AppState>) -> impl Responder {
    SecretsPost::logic(_permissions, post, shared).await
}

#[cfg(test)]
mod spec_tests {
    use serde_json::Value;
    use utoipa::OpenApi;
    use super::*;

    /// Build an OpenAPI doc from THIS operation and the concrete schemas it uses.
    #[derive(OpenApi)]
    #[openapi(
        paths(
            super::post_secrets,
        ),
        components(schemas(
            CreateSecretBody,
        )),
        tags((name = "Secrets", description = "Secret management endpoints"))
        // If you define top-level security elsewhere, that's fine.
        // This test asserts the operation-level requirement for bearerAuth.
    )]
    struct ApiDoc;

    #[test]
    fn post_secrets_is_in_spec_with_correct_operation_id_and_request_body() {
        let v: Value = serde_json::to_value(ApiDoc::openapi()).unwrap();
        let post = &v["paths"]["/v1/secrets"]["post"];
        assert!(post.is_object(), "POST /v1/secrets not found");
        assert_eq!(post["operationId"], "createSecret");

        // request body points to CreateSecretBody
        assert_eq!(
            post["requestBody"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/CreateSecretBody",
            "request body should be CreateSecretBody"
        );
    }

    #[test]
    fn post_secrets_requires_bearer_auth() {
        let v: Value = serde_json::to_value(ApiDoc::openapi()).unwrap();
        let sec = &v["paths"]["/v1/secrets"]["post"]["security"];
        assert!(sec.is_array(), "expected security array on POST /v1/secrets");
        let arr = sec.as_array().unwrap();
        assert!(
            !arr.is_empty(),
            "expected POST /v1/secrets to require auth (bearerAuth)"
        );
        // Look for an object like { "bearerAuth": [] }
        let has_bearer = arr.iter().any(|entry| {
            entry.get("bearerAuth").map(|v| v.is_array()).unwrap_or(false)
        });
        assert!(has_bearer, "expected bearerAuth requirement on POST /v1/secrets");
    }

    #[test]
    fn post_secrets_has_201_created_without_body_and_400_with_example() {
        let v: Value = serde_json::to_value(ApiDoc::openapi()).unwrap();
        let post = &v["paths"]["/v1/secrets"]["post"];

        // 201 Created present
        let resp_201 = &post["responses"]["201"];
        assert!(resp_201.is_object(), "expected 201 response object");
        // No content schema defined for 201 since none was specified in the annotation
        let content_201 = &resp_201["content"];
        assert!(
            content_201.is_null(),
            "201 Created should not define a content schema (no body specified)"
        );

        // 400 Bad Request present and carries an example
        let resp_400 = &post["responses"]["400"];
        assert!(resp_400.is_object(), "expected 400 response object");

        // Depending on utoipa version, the example may be nested differently.
        // This checks that *some* example node exists.
        let has_example = resp_400.get("example").is_some()
            || resp_400.get("content").and_then(|c| c.get("application/json")).and_then(|j| j.get("example")).is_some();
        assert!(has_example, "expected 400 response to include an example payload");
    }
}
