use actix_web::HttpResponse;
use utoipa::OpenApi;
use crate::{
    api::sessions,
    api::secrets,
    api::verifications,
    types::ApiResponse
};

#[derive(OpenApi)]
#[openapi(
    info(title = "battle-texas-server", version = "0.0.6"),
    paths(
        sessions::sessions_openapi_spec::post_sessions,
        sessions::sessions_openapi_spec::delete_sessions,
        secrets::secrets_openapi_spec::post_secrets,
        verifications::email::email_verifications_openapi_spec::post_email_verification,
        verifications::email::email_verifications_openapi_spec::patch_email_verification,
        // add more endpoints here...
    ),
    components(schemas(
        sessions::CreateSessionBody,
        sessions::AccessToken,
        ApiResponse<sessions::AccessToken>
    )),
    tags(
        (name="sessions", description="Session endpoints"),
        (name="secrets", description="Secrets endpoints"),
        (name="verifications", description="Verifications endpoints")
    )
)]
pub struct ApiDoc;

impl ApiDoc {
    pub async fn yaml() -> HttpResponse {
        let doc = ApiDoc::openapi().to_yaml();
        
        match doc {
            Ok(yaml) => HttpResponse::Ok().content_type("application/yaml").body(yaml),
            Err(_) => HttpResponse::InternalServerError().content_type("text/html").body("failed to generate yaml")
        }
    }

    pub async fn json() -> HttpResponse {
        let doc = ApiDoc::openapi().to_pretty_json();
        
        match doc {
            Ok(json) => HttpResponse::Ok().content_type("application/yaml").body(json),
            Err(_) => HttpResponse::InternalServerError().content_type("text/html").body("failed to generate yaml")
        }
    }
}