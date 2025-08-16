/// route collections pass incoming requests to endpoint handlers
/// 
/// middleware possibilities
/// - session management
/// - rate limiting
/// - permission checks
use actix_web::{web,Scope};

use crate::{api::HealthCheck, services::RouteLock, types::SoftwareAccess};

#[derive(Clone,Debug)]
pub struct RouteCollection;

impl RouteCollection {
    /// main route scope builder
    pub fn v1(&self) -> Scope {
        Scope::new("/v1")
            .configure(RouteCollection::health)
    }

    /// returns server health
    pub fn health(cfg: &mut web::ServiceConfig) {
        cfg.route("/health", web::get().to(HealthCheck::logic).wrap(RouteLock::default(SoftwareAccess::default())));
    }
}