use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(title = "idropr-server", version = "0.0.6"),
    paths(
        crate::api::sessions::sessions_openapi_spec::post_sessions,
        // add more endpoints here...
    ),
    components(schemas(
        crate::api::sessions::Post,
        crate::api::sessions::AccessToken,
        crate::types::ApiResponse<crate::api::sessions::AccessToken>
    )),
    tags((name="session", description="Session endpoints"))
)]
pub struct ApiDoc;