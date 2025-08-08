use actix_web::Scope;

#[derive(Clone,Debug)]
pub struct RouteCollection;

impl RouteCollection {
    pub fn public(&self) -> Scope {
        Scope::new("/public")
    }

    pub fn private(&self) -> Scope {
        Scope::new("/private")
    }
}