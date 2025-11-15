#[cfg(test)]
mod open_api_verifications_tests {
    use serde_json::Value;
    use crate::types::open_api_doc::ApiDoc;

    fn openapi_json() -> Value {
        let doc = ApiDoc::doc();
        serde_json::to_value(&doc).expect("serialize openapi to value")
    }

    fn has_tag(op: &Value, tag: &str) -> bool {
        op["tags"]
            .as_array()
            .map(|a| a.iter().any(|t| t == tag))
            .unwrap_or(false)
    }

    /// Anonymous is acceptable if security is omitted, an empty array, or `[{}]`
    fn is_anonymous_security(op: &Value) -> bool {
        match op.get("security") {
            None => true,
            Some(Value::Array(a)) if a.is_empty() => true,
            Some(Value::Array(a)) if a.len() == 1 && a[0] == serde_json::json!({}) => true,
            _ => false,
        }
    }

    #[test]
    fn verifications_paths_present() {
        let v = openapi_json();
        let paths = v["paths"].as_object().expect("paths object missing");
        let uuid = "{uuid}";
        let id = "{id}";

        assert!(
            paths.contains_key("/v1/verifications/email"),
            "missing /v1/verifications/email path"
        );
        assert!(
            paths.contains_key("/v1/verifications/email/{uuid}/{id}"),
            "missing /v1/verifications/email/{uuid}/{id} path"
        );

        assert!(
            v["paths"]["/v1/verifications/email"]["post"].is_object(),
            "POST /v1/verifications/email missing"
        );
        assert!(
            v["paths"]["/v1/verifications/email/{uuid}/{id}"]["patch"].is_object(),
            "PATCH /v1/verifications/email/{uuid}/{id} missing"
        );
    }

    #[test]
    fn post_email_verification_operation() {
        let v = openapi_json();
        let post = &v["paths"]["/v1/verifications/email"]["post"];

        assert_eq!(post["operationId"], "createEmailVerification", "operationId mismatch");
        assert!(has_tag(post, "email"), "tag 'email' missing");
        assert!(has_tag(post, "verifications"), "tag 'verifications' missing");

        // Anonymous allowed
        assert!(is_anonymous_security(post), "POST /v1/verifications/email should be anonymous (no auth)");

        // requestBody schema
        let rb = &post["requestBody"]["content"]["application/json"]["schema"]["$ref"];
        assert_eq!(
            rb,
            "#/components/schemas/EmailVerificationPost",
            "POST email verification requestBody must reference EmailVerificationPost"
        );

        // Responses present
        let resp = &post["responses"];
        for code in ["201", "400", "403", "429", "500"] {
            assert!(resp.get(code).is_some(), "POST response {code} missing");
        }

        // 201 success body -> SuccessMessage
        let s201_ref = &resp["201"]["content"]["application/json"]["schema"]["$ref"];
        assert_eq!(
            s201_ref,
            "#/components/schemas/SuccessMessage",
            "201 schema must be SuccessMessage"
        );

        // 400 examples
        let bad = &resp["400"]["content"]["application/json"];
        let bad_schema_ref = &bad["schema"]["$ref"];
        assert_eq!(
            bad_schema_ref,
            "#/components/schemas/ApiResultError",
            "400 schema must be ApiResultError"
        );
        let bad_ex = &bad["examples"];
        assert!(bad_ex.get("already_verified").is_some(), "missing already_verified example");
        assert!(bad_ex.get("provider_rejected").is_some(), "missing provider_rejected example");

        // 403 example
        let fbd = &resp["403"]["content"]["application/json"];
        let fbd_schema_ref = &fbd["schema"]["$ref"];
        assert_eq!(fbd_schema_ref, "#/components/schemas/ApiResultError", "403 schema must be ApiResultError");
        assert!(
            fbd.get("example").is_some(),
            "403 should include an example payload"
        );

        // 429 examples
        let rl = &resp["429"]["content"]["application/json"];
        let rl_schema_ref = &rl["schema"]["$ref"];
        assert_eq!(rl_schema_ref, "#/components/schemas/ApiResultError", "429 schema must be ApiResultError");
        let rl_ex = &rl["examples"];
        assert!(rl_ex.get("global_rate_limit").is_some(), "missing global_rate_limit example");
        assert!(rl_ex.get("verification_rate_limited").is_some(), "missing verification_rate_limited example");
    }

    #[test]
    fn patch_email_verification_operation() {
        let v = openapi_json();
        let patch = &v["paths"]["/v1/verifications/email/{uuid}/{id}"]["patch"];
        let uuid = "{uuid}";
        let id = "{id}";

        assert_eq!(patch["operationId"], "patchEmailVerification", "operationId mismatch");
        assert!(has_tag(patch, "email"), "tag 'email' missing");
        assert!(has_tag(patch, "verifications"), "tag 'verifications' missing");

        // Anonymous allowed
        assert!(is_anonymous_security(patch), "PATCH /v1/verifications/email/{uuid}/{id} should be anonymous (no auth)");

        // Params: uuid (string), id (int64)
        let params = patch["parameters"].as_array().expect("parameters array missing");

        let uuid = params.iter().find(|p| p["name"] == "uuid").expect("uuid param missing");
        assert_eq!(uuid["in"], "path");
        assert_eq!(uuid["required"], true);
        assert_eq!(uuid["schema"]["type"], "string");

        let id = params.iter().find(|p| p["name"] == "id").expect("id param missing");
        assert_eq!(id["in"], "path");
        assert_eq!(id["required"], true);
        assert_eq!(id["schema"]["type"], "integer");
        assert_eq!(id["schema"]["format"], "int64");

        // Responses present
        let resp = &patch["responses"];
        for code in ["201", "400", "401", "410", "429", "500"] {
            assert!(resp.get(code).is_some(), "PATCH response {code} missing");
        }

        // 201 success -> SuccessMessage
        let s201_ref = &resp["201"]["content"]["application/json"]["schema"]["$ref"];
        assert_eq!(
            s201_ref,
            "#/components/schemas/SuccessMessage",
            "201 schema must be SuccessMessage"
        );

        // 400 schema + example
        let bad = &resp["400"]["content"]["application/json"];
        let bad_ref = &bad["schema"]["$ref"];
        assert_eq!(bad_ref, "#/components/schemas/ApiResultError", "400 schema must be ApiResultError");
        assert!(
            bad.get("example").is_some(),
            "400 should include 'already_verified' example"
        );

        // 401 schema + example
        let unauth = &resp["401"]["content"]["application/json"];
        let ua_ref = &unauth["schema"]["$ref"];
        assert_eq!(ua_ref, "#/components/schemas/ApiResultError", "401 schema must be ApiResultError");
        assert!(
            unauth.get("example").is_some(),
            "401 should include an example payload"
        );

        // 410 has two named examples
        let gone = &resp["410"]["content"]["application/json"];
        let gone_ref = &gone["schema"]["$ref"];
        assert_eq!(gone_ref, "#/components/schemas/ApiResultError", "410 schema must be ApiResultError");
        let gex = &gone["examples"];
        assert!(gex.get("link_expired").is_some(), "missing link_expired example");
        assert!(gex.get("record_not_found").is_some(), "missing record_not_found example");

        // 429 schema + example
        let rl = &resp["429"]["content"]["application/json"];
        let rl_ref = &rl["schema"]["$ref"];
        assert_eq!(rl_ref, "#/components/schemas/ApiResultError", "429 schema must be ApiResultError");
        assert!(
            rl.get("examples").is_some(),
            "429 should include 'global_rate_limit' example"
        );

        // 500 schema + example
        let se = &resp["500"]["content"]["application/json"];
        let se_ref = &se["schema"]["$ref"];
        assert_eq!(se_ref, "#/components/schemas/ApiResultError", "500 schema must be ApiResultError");
        assert!(
            se.get("examples").is_some(),
            "500 should include 'server_error' example"
        );
    }

    #[test]
    fn components_for_verifications_present() {
        let v = openapi_json();
        let comps = &v["components"]["schemas"];

        for needed in [
            "EmailVerificationPost",
            "ApiResultError",
            "ApiErrorData",
            "SuccessMessage",
        ] {
            assert!(comps.get(needed).is_some(), "components.schemas.{needed} missing");
        }
    }
}
