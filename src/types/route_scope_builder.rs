use actix_web::Scope;

#[derive(Debug)]
pub struct RouteScope;

impl RouteScope {
    pub fn public() -> Scope {
        Scope::new("/public")
    }
}