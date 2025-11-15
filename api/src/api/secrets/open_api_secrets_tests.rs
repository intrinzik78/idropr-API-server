#[cfg(test)]
mod open_api_secrets_tests {
    use serde_json::Value;
    use crate::types::open_api_doc::ApiDoc;

    fn openapi_json() -> Value {
        let doc = ApiDoc::doc();
        serde_json::to_value(&doc).expect("serialize openapi to value")
    }

    fn contains_bearer_auth(sec: &Value) -> bool {
        match sec {
            Value::Array(items) => items.iter().any(|it| {
                matches!(it, Value::Object(map) if map.contains_key("bearerAuth"))
            }),
            _ => false,
        }
    }

    #[test]
    fn secrets_path_and_post_present() {
        let v = openapi_json();
        let paths = v["paths"].as_object().expect("paths object missing");

        assert!(
            paths.contains_key("/v1/secrets"),
            "missing /v1/secrets path. Present: {:?}",
            paths.keys().collect::<Vec<_>>()
        );

        let sec = &v["paths"]["/v1/secrets"];
        assert!(
            sec["post"].is_object(),
            "POST /v1/secrets missing. Found keys: {:?}",
            sec.as_object().map(|o| o.keys().collect::<Vec<_>>())
        );
    }

    #[test]
    fn secrets_post_operation_basics() {
        let v = openapi_json();
        let post = &v["paths"]["/v1/secrets"]["post"];

        // operationId & tags
        assert_eq!(post["operationId"], "createSecret", "operationId mismatch");
        let tags = post["tags"].as_array().expect("tags array missing");
        assert!(tags.iter().any(|t| t == "secrets"), "tag 'secrets' missing");

        // security: must require bearerAuth
        let sec = &post["security"];
        assert!(
            contains_bearer_auth(sec),
            "expected security to include bearerAuth for POST /v1/secrets, got: {sec}"
        );

        // requestBody -> CreateSecretBody
        let rb_schema_ref = &post["requestBody"]["content"]["application/json"]["schema"]["$ref"];
        assert_eq!(
            rb_schema_ref,
            "#/components/schemas/CreateSecretBody",
            "POST /v1/secrets requestBody must reference CreateSecretBody"
        );

        // responses present: 201, 401, 400, 429, 500
        let resp = &post["responses"];
        for code in ["201", "401", "400", "429", "500"] {
            assert!(
                resp.get(code).is_some(),
                "POST /v1/secrets response {code} missing"
            );
        }
    }

    #[test]
    fn secrets_post_response_shapes() {
        let v = openapi_json();
        let resp = &v["paths"]["/v1/secrets"]["post"]["responses"];

        // 201 is a bare description (no enforced schema)
        assert_eq!(resp["201"]["description"], "resource created");

        // 400 should reference ApiResult_ApiResponse_String
        let bad_ref = &resp["400"]["content"]["application/json"]["schema"]["$ref"];
        assert_eq!(
            bad_ref,
            "#/components/schemas/ApiResult_ApiResponse_String",
            "400 schema must be ApiResult_ApiResponse_String (ApiResult<ApiResponse<String>>)"
        );

        // 429 has example 'global_rate_limit'
        let rl = &resp["429"]["content"]["application/json"]["examples"]["global_rate_limit"]["value"];
        assert!(
            rl.is_object(),
            "expected example 'global_rate_limit' under 429"
        );

        // 500 has example 'server_error'
        let se = &resp["500"]["content"]["application/json"]["examples"]["server_error"]["value"];
        assert!(
            se.is_object(),
            "expected example 'server_error' under 500"
        );
    }

    #[test]
    fn components_for_secrets_present() {
        let v = openapi_json();
        let comps = &v["components"]["schemas"];

        // Base shapes referenced by secrets
        for needed in [
            "CreateSecretBody",
            "ApiResult_ApiResponse_String",
            "ApiResultError",
            "ApiErrorData",
        ] {
            assert!(
                comps.get(needed).is_some(),
                "components.schemas.{needed} missing"
            );
        }

        // security scheme exists & named correctly
        let schemes = &v["components"]["securitySchemes"];
        assert!(
            schemes.get("bearerAuth").unwrap().is_object(),
            "securitySchemes.bearerAuth missing"
        );
    }
}
