#[cfg(test)]
mod open_api_session_tests {
    use serde_json::Value;
    use crate::types::open_api_doc::ApiDoc;

    /// Build the OpenAPI as serde_json::Value for easy assertions
    fn openapi_json() -> Value {
        let doc = ApiDoc::doc();
        serde_json::to_value(&doc).expect("serialize openapi to value")
    }

    /// helper: returns true if security: [ {} ] (an array with one empty object)
    fn is_empty_security(sec: &Value) -> bool {
        match sec {
            Value::Array(items) if items.len() == 1 => {
                matches!(&items[0], Value::Object(map) if map.is_empty())
            }
            _ => false,
        }
    }

    /// helper: returns true if security: [ { "bearerAuth": [] } ] (or includes such an entry)
    fn contains_bearer_auth(sec: &Value) -> bool {
        match sec {
            Value::Array(items) => items.iter().any(|it| {
                match it {
                    Value::Object(map) => map.contains_key("bearerAuth"),
                    _ => false,
                }
            }),
            _ => false,
        }
    }

    #[test]
    fn sessions_paths_present() {
        let v = openapi_json();
        let paths = v["paths"].as_object().expect("paths object missing");

        assert!(
            paths.contains_key("/v1/sessions"),
            "missing /v1/sessions path. Present: {:?}",
            paths.keys().collect::<Vec<_>>()
        );

        let sess = &v["paths"]["/v1/sessions"];
        assert!(
            sess["post"].is_object(),
            "POST /v1/sessions missing (createSession op). Found keys: {:?}",
            sess.as_object().map(|o| o.keys().collect::<Vec<_>>())
        );
        assert!(
            sess["delete"].is_object(),
            "DELETE /v1/sessions missing (deleteSession op). Found keys: {:?}",
            sess.as_object().map(|o| o.keys().collect::<Vec<_>>())
        );
    }

    #[test]
    fn post_sessions_operation_basics() {
        let v = openapi_json();
        let post = &v["paths"]["/v1/sessions"]["post"];

        // tags & operationId
        assert_eq!(post["operationId"], "createSession", "operationId mismatch");
        let tags = post["tags"].as_array().expect("tags array missing");
        assert!(tags.iter().any(|t| t == "sessions"), "tag 'sessions' missing");

        // security: explicitly public -> should be [ {} ]
        let sec = &post["security"];
        assert!(
            is_empty_security(sec),
            "expected security to be [{{}}] for POST /v1/sessions, got: {sec}"
        );

        // requestBody -> CreateSessionBody
        let rb_schema_ref = &post["requestBody"]["content"]["application/json"]["schema"]["$ref"];
        assert_eq!(
            rb_schema_ref,
            "#/components/schemas/CreateSessionBody",
            "POST /v1/sessions requestBody must reference CreateSessionBody"
        );

        // responses: 200, 401, 429 present
        let resp = &post["responses"];
        assert!(resp.get("200").is_some(), "200 response missing");
        assert!(resp.get("401").is_some(), "401 response missing");
        assert!(resp.get("429").is_some(), "429 response missing");

        // 200 response schema -> ApiResultToken
        let ok_ref = &resp["200"]["content"]["application/json"]["schema"]["$ref"];
        assert_eq!(
            ok_ref,
            "#/components/schemas/ApiResultToken",
            "200 schema must be ApiResultToken"
        );

        // 429 example key present (global_rate_limit)
        let rl = &resp["429"]["content"]["application/json"]["examples"]["global_rate_limit"]["value"];
        assert!(
            rl.is_object(),
            "expected an example named 'global_rate_limit' under 429"
        );
    }

    #[test]
    fn delete_sessions_operation_basics() {
        let v = openapi_json();
        let del = &v["paths"]["/v1/sessions"]["delete"];

        // tags & operationId
        assert_eq!(del["operationId"], "deleteSession", "operationId mismatch");
        let tags = del["tags"].as_array().expect("tags array missing");
        assert!(tags.iter().any(|t| t == "sessions"), "tag 'sessions' missing");

        // security: requires bearerAuth
        let sec = &del["security"];
        assert!(
            contains_bearer_auth(sec),
            "expected security to include bearerAuth for DELETE /v1/sessions, got: {sec}"
        );

        // responses: 204, 401, 429, 500 present
        let resp = &del["responses"];
        for code in ["204", "401", "429", "500"] {
            assert!(
                resp.get(code).is_some(),
                "DELETE /v1/sessions response {code} missing"
            );
        }

        // 204 description sanity
        assert_eq!(
            resp["204"]["description"], "no content",
            "204 description should be 'no content'"
        );

        // 500 has example 'server_error'
        let ex = &resp["500"]["content"]["application/json"]["examples"]["server_error"]["value"];
        assert!(
            ex.is_object(),
            "expected example 'server_error' on 500 response"
        );
    }

    #[test]
    fn components_for_sessions_present() {
        let v = openapi_json();
        let comps = &v["components"]["schemas"];

        for needed in [
            "CreateSessionBody",
            "ApiResultToken",
            "ApiResultError",
            "AccessToken",
        ] {
            assert!(
                comps.get(needed).is_some(),
                "components.schemas.{needed} missing"
            );
        }
    }
}
