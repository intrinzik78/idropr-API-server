/// route collections pass incoming requests to endpoint handlers
use actix_web::{web,Scope};

use crate::{
    api::{HealthCheck,sessions,secrets},
    enums::Role,
    services::RouteLock,
    types::UserPermissions
};

#[derive(Clone,Debug)]
pub struct RouteCollection;

/// main collector
impl RouteCollection {
    /// main route scope builder
    pub fn v1(&self) -> Scope {
        Scope::new("/v1")
            .configure(RouteCollection::health)
            .configure(RouteCollection::sessions)
            .configure(RouteCollection::secrets)
    }
}


impl RouteCollection {
    /// returns server health
    pub fn health(cfg: &mut web::ServiceConfig) {
        cfg.route("/health", web::get().to(HealthCheck::logic));
        //.wrap(RouteLock::default(UserPermissions::default()))
    }

    /// sessions resource and endpoints
    pub fn sessions(cfg: &mut web::ServiceConfig) {
        let user = UserPermissions::from_role(Role::User);

        cfg.service(
            actix_web::web::scope("/sessions")
                .route("", web::post().to(sessions::SessionsPost::logic))
                .route("", web::delete().to(sessions::SessionsDelete::logic).wrap(RouteLock::default(&user)))
        );
    }
    
    /// users resource and endpoints
    pub fn users(_cfg: &mut web::ServiceConfig) {
        todo!()
    }

    /// buckets resource and endpoints
    pub fn buckets(_cfg: &mut web::ServiceConfig) {
        todo!()
    }

    /// images resource and endpoints
    pub fn images(_cfg: &mut web::ServiceConfig) {
        todo!()
    }

    /// secrets resource and endpoings
    pub fn secrets(cfg: &mut web::ServiceConfig) {
        let sysadmin = UserPermissions::from_role(Role::SysAdmin);

        cfg.service(
            actix_web::web::scope("/secrets")
                .wrap(RouteLock::default(&sysadmin))
                .route("", actix_web::web::post().to(secrets::SecretsPost::logic))
                .route("/{id}", actix_web::web::get().to(secrets::SecretsGet::logic))
                .route("/{id}", actix_web::web::put().to(secrets::SecretsPut::logic))
                .route("/{id}", actix_web::web::patch().to(secrets::SecretsPatch::logic))
                .route("/{id}", actix_web::web::delete().to(secrets::SecretsDelete::logic))
        );
    }
}