/// route collections pass incoming requests to endpoint handlers
/// 
/// middleware possibilities
/// - session management
/// - rate limiting
/// - permission checks
use actix_web::{web,Scope};

use crate::api::public::HealthCheck;

#[derive(Clone,Debug)]
pub struct RouteCollection;

impl RouteCollection {
    /// public get endpoints
    pub fn public_get_collection(cfg: &mut web::ServiceConfig) {
        cfg.route("/healthz", web::get().to(HealthCheck::logic));
    }

    /// public post endpoints
    pub fn public_post_collection(_cfg: &mut web::ServiceConfig) {

    }

    /// private get endpoints
    pub fn private_get_collection(_cfg: &mut web::ServiceConfig) {

    }

    /// private post endpoints
    pub fn private_post_collection(_cfg: &mut web::ServiceConfig) {

    }

    pub fn public(&self) -> Scope {
        Scope::new("/public")
            .configure(RouteCollection::public_get_collection)
    }

    pub fn private(&self) -> Scope {
        Scope::new("/private")
    }
}