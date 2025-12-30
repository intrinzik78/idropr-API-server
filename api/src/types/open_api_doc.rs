use actix_web::HttpResponse;
use serde::Serialize;
use utoipa::{OpenApi, openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},Modify};
use crate::{
    api::{extractions,locations, secrets, sessions, verifications},
    enums::ActivityType,
    types::ApiResponse
};

#[derive(Default, OpenApi)]
#[openapi(
    modifiers(&Security),
    security(("bearerAuth" = [])),
    info(title = "battle-texas-server", version = "0.0.6"),
    paths(
        extractions::extraction_openapi_spec::post_extraction_session,
        locations::locations_openapi_spec::get_private_location_by_id,
        locations::locations_openapi_spec::get_public_location_by_id,
        locations::locations_openapi_spec::get_public_nearest_locations_by_zipcode,
        sessions::sessions_openapi_spec::post_sessions,
        sessions::sessions_openapi_spec::delete_sessions,
        secrets::secrets_openapi_spec::post_secrets,
        verifications::email::email_verifications_openapi_spec::post_email_verification,
        verifications::email::email_verifications_openapi_spec::patch_email_verification,
        // add more endpoints here...
    ),
    components(
        schemas(
            sessions::CreateSessionBody,
            sessions::AccessToken,
            ApiResponse<sessions::AccessToken>,
            ActivityType
        ),
    ),
    tags(
        (name="sessions", description="session endpoints for user authentication"),
        (name="secrets", description="CRUD management of API secrets"),
        (name="verifications", description="email and sms verification endpoints"),
        (name="locations", description="business location endpoints"),
        (name="extractions", description="document scans and ocr extractions to data types"),
        (name="scans", description="document scans and ocr extractions to data types")
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
            Ok(json) => HttpResponse::Ok().content_type("application/json").body(json),
            Err(_) => HttpResponse::InternalServerError().content_type("text/html").body("failed to generate json")
        }
    }

    pub fn doc() -> utoipa::openapi::OpenApi {
        ApiDoc::openapi()
    }
}

#[derive(Debug,Default,Serialize)]
struct Security;

impl Modify for Security {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(schema) = openapi.components.as_mut() {
            schema.add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}
