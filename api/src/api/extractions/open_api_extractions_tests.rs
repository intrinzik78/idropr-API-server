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
}
