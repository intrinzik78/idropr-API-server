#[cfg(test)]
mod open_api_session_tests {
    use serde_json::Value;
    use crate::types::open_api_doc::ApiDoc;

    // use serde_json::Value;
    // use crate::types::open_api_doc::ApiDoc;

    fn openapi_json() -> Value {
        let doc = ApiDoc::doc();
        serde_json::to_value(&doc).expect("serialize openapi doc")
    }

    fn get_op<'a>(v: &'a Value, path: &str, method: &str) -> &'a Value {
        &v["paths"][path][method]
    }

    fn is_security_override_empty(security: &Value) -> bool {
        // Accept both [] and [{}] since utoipa may emit either to override global security
        match security {
            Value::Array(arr) if arr.is_empty() => true,
            Value::Array(arr) if arr.len() == 1 => arr[0].as_object().map(|m| m.is_empty()).unwrap_or(false),
            _ => false,
        }
    }

    #[test]
    fn debug_dump_paths() {
        let v = openapi_json();
        let paths = v.get("paths").and_then(|p| p.as_object())
            .expect("OpenAPI has no paths object");
        eprintln!("paths present:\n{:#?}", paths.keys().collect::<Vec<_>>());
        assert!(!paths.is_empty(), "no paths were emitted at all");
    }
    #[test]
    fn locations_paths_present() {
        println!("TESTING");
        let v = openapi_json();
        assert!(v["paths"]["/v1/locations/{id}"].is_object(), "public path missing");
        assert!(v["paths"]["/v1/locations/{id}/private"].is_object(), "private path missing");
    }

    #[test]
    fn locations_operation_ids_and_tags() {
        let v = openapi_json();

        let pub_get = get_op(&v, "/v1/locations/{id}", "get");
        assert_eq!(pub_get["operationId"], "getPublicLocationById");
        assert!(pub_get["tags"].as_array().unwrap().contains(&Value::String("locations".into())));

        let priv_get = get_op(&v, "/v1/locations/{id}/private", "get");
        assert_eq!(priv_get["operationId"], "getPrivateLocationById");
        assert!(priv_get["tags"].as_array().unwrap().contains(&Value::String("locations".into())));
    }

    #[test]
    fn locations_id_parameter_shape() {
        let v = openapi_json();

        for (path, method) in [
            ("/v1/locations/{id}", "get"),
            ("/v1/locations/{id}/private", "get"),
        ] {
            let op = get_op(&v, path, method);
            let params = op["parameters"].as_array().expect("parameters array");
            let id = params.iter().find(|p| p["name"] == "id").expect("id param");
            assert_eq!(id["in"], "path");
            assert_eq!(id["required"], true);
            assert_eq!(id["schema"]["type"], "integer");
            assert_eq!(id["schema"]["format"], "int64");
        }
    }

    #[test]
    fn public_location_is_unauthenticated_via_override() {
        let v = openapi_json();
        // global security should exist (secure-by-default)
        assert!(v["security"].is_array(), "global security missing (expected secure-by-default)");

        let pub_get = get_op(&v, "/v1/locations/{id}", "get");
        let security = &pub_get["security"];
        let test  ="{}";
        assert!(is_security_override_empty(security),"public GET should override global security with [] or [{test}], got: {security:#}");
    }

    #[test]
    fn private_location_requires_bearer_auth() {
        let v = openapi_json();

        // securitySchemes.bearerAuth exists
        let bearer = &v["components"]["securitySchemes"]["bearerAuth"];
        assert_eq!(bearer["type"], "http");
        assert_eq!(bearer["scheme"], "bearer");
        assert_eq!(bearer["bearerFormat"], "JWT");

        // operation requires bearerAuth
        let priv_get = get_op(&v, "/v1/locations/{id}/private", "get");
        let sec = priv_get["security"].as_array().expect("security array on private GET");
        assert!(sec.iter().any(|entry| entry.get("bearerAuth").is_some()),
            "private GET must require bearerAuth, got: {sec:#?}");
    }

    #[test]
    fn response_examples_exist() {
        let v = openapi_json();

        // public 200 example present
        let pub_200 = get_op(&v, "/v1/locations/{id}", "get")["responses"]["200"]["content"]["application/json"]["example"].clone();
        assert!(pub_200.is_object(), "public 200 example missing");

        // private 401 example present
        let priv_401 = get_op(&v, "/v1/locations/{id}/private", "get")["responses"]["401"]["content"]["application/json"]["example"].clone();
        assert!(priv_401.is_object(), "private 401 example missing");

        // rate limit example present on public 429 OR in examples block
        let pub_429 = get_op(&v, "/v1/locations/{id}", "get")["responses"]["429"]["content"]["application/json"].clone();
        assert!(pub_429.get("example").is_some() || pub_429.get("examples").is_some(),
            "public 429 should include example(s)");
    }

    #[test]
    fn wrappers_are_registered() {
        let v = openapi_json();
        let schemas = &v["components"]["schemas"];
        for name in [
            "ApiResultPublicLocation",
            "ApiResultPrivateLocation",
            "ApiResultPublicLocationsList", // <-- add this
            "ApiResultError",
        ] {
            assert!(schemas.get(name).is_some(), "schema {name} missing");
        }
    }

    #[test]
    fn locations_list_path_present() {
        let v = openapi_json();
        assert!(v["paths"]["/v1/locations"].is_object(), "GET /v1/locations path missing");
        assert!(v["paths"]["/v1/locations"]["get"].is_object(), "GET operation missing");
    }

    #[test]
    fn locations_list_security_is_empty_override() {
        let v = openapi_json();
        let sec = &v["paths"]["/v1/locations"]["get"]["security"];
        // Utoipa encodes security([]) as [{}]
        assert!(sec.is_array(), "security must be an array");
        assert_eq!(sec.as_array().unwrap().len(), 1, "security override length");
        assert!(sec[0].as_object().unwrap().is_empty(), "security override should be empty object");
    }

    #[test]
    fn locations_list_param_and_200_schema() {
        let v = openapi_json();
        let get = &v["paths"]["/v1/locations"]["get"];

        // param presence
        let params = get["parameters"].as_array().expect("parameters array");
        let has_zip = params.iter().any(|p| {
            p["name"] == "nearest_zipcode" && p["in"] == "query"
        });
        assert!(has_zip, "nearest_zipcode query param missing");

        // 200 schema
        let sch_ref = &get["responses"]["200"]["content"]["application/json"]["schema"]["$ref"];
        assert!(sch_ref.is_string(), "200 schema should be a $ref");
        assert!(sch_ref.as_str().unwrap().ends_with("/ApiResultPublicLocationsList"),
            "200 schema should reference ApiResultPublicLocationsList");
    }

    #[test]
    fn locations_list_429_error_enveloped() {
        let v = openapi_json();
        let example = &v["paths"]["/v1/locations"]["get"]["responses"]["429"]["content"]["application/json"]["examples"]["global_rate_limit"]["value"];
        assert_eq!(example["Error"]["code"], 429, "429 example must be enveloped Error with code 429");
    }

    #[test]
    fn public_location_500_is_enveloped_error() {
        let v = openapi_json();

        let op = &v["paths"]["/v1/locations/{id}"]["get"];
        assert!(op.is_object(), "operation missing");

        // 500 exists and has application/json
        let r500 = &op["responses"]["500"];
        assert!(r500.is_object(), "500 response missing");

        let content = &r500["content"]["application/json"]["schema"]["$ref"];
        assert!(content.is_string(), "500 must declare a schema $ref");
        assert_eq!(
            content.as_str().unwrap(),
            "#/components/schemas/ApiResultError",
            "500 must use ApiResultError envelope"
        );

        // (optional but nice) example present and enveloped
        let eg = &r500["content"]["application/json"]["examples"]["server_error"]["value"]["Error"]["code"];
        assert_eq!(eg.as_i64().unwrap_or_default(), 500, "500 example must be enveloped Error with code 500");
    }

    #[test]
    fn private_location_500_is_enveloped_error() {
        let v = openapi_json();
        let op = &v["paths"]["/v1/locations/{id}/private"]["get"];

        let r500 = &op["responses"]["500"];
        assert!(r500.is_object(), "private 500 response missing");

        let schema_ref = &r500["content"]["application/json"]["schema"]["$ref"];
        assert_eq!(
            schema_ref,
            "#/components/schemas/ApiResultError",
            "private 500 must use ApiResultError envelope"
        );

        let eg = &r500["content"]["application/json"]["examples"]["server_error"]["value"]["Error"]["code"];
        assert_eq!(eg.as_i64().unwrap_or_default(), 500, "private 500 Error.code must be 500");
    }

    #[test]
    fn locations_list_400_error_enveloped() {
        let v = openapi_json();
        let r400 = &v["paths"]["/v1/locations"]["get"]["responses"]["400"];

        let schema_ref = &r400["content"]["application/json"]["schema"]["$ref"];
        assert_eq!(
            schema_ref,
            "#/components/schemas/ApiResultError",
            "400 schema must be ApiResultError"
        );

        let example = &r400["content"]["application/json"]["examples"]["bad_request"]["value"]["Error"];
        assert_eq!(example["code"], 400, "400 Error.code must be 400");
        assert!(example["data"]["code"].is_number(), "400 Error.data.code must exist");
    }

    #[test]
    fn private_location_401_is_enveloped_error() {
        let v = openapi_json();
        let resp = &v["paths"]["/v1/locations/{id}/private"]["get"]["responses"]["401"];

        let schema_ref = &resp["content"]["application/json"]["schema"]["$ref"];
        assert_eq!(
            schema_ref,
            "#/components/schemas/ApiResultError",
            "401 schema must be ApiResultError envelope"
        );

        let e = &resp["content"]["application/json"]["example"]["Error"];
        assert_eq!(e["code"], 401);
        assert!(e["message"].is_string());
    }

}