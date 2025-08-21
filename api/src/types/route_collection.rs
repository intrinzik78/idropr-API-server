/// route collections pass incoming requests to endpoint handlers
use actix_web::{web,Scope};

use crate::{
    api::{
        HealthCheck,
        sessions
    },
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
        cfg.route("/sessions", web::post().to(sessions::SessionsPost::logic));
        
        let permissions = UserPermissions::default().with_sessions_delete();
        cfg.route("/sessions", web::delete().to(sessions::SessionsDelete::logic).wrap(RouteLock::default(permissions)));
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
}