use actix_web::HttpResponse;
use utoipa::OpenApi;
use crate::{
    api::sessions,
    types::ApiResponse
};

#[derive(OpenApi)]
#[openapi(
    info(title = "idropr-server", version = "0.0.6"),
    paths(
        crate::api::sessions::sessions_openapi_spec::post_sessions,
        // add more endpoints here...
    ),
    components(schemas(
        sessions::CreateSessionBody,
        sessions::AccessToken,
        ApiResponse<sessions::AccessToken>
    )),
    tags((name="session", description="Session endpoints"))
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