use std::borrow::Cow;
use form_urlencoded;

/// route collections pass incoming requests to endpoint handlers
use actix_web::{Scope, guard::{self, GuardContext}, web};

use crate::{
    api::{HealthCheck,locations,sessions,secrets,verifications},
    enums::Role,
    services::RouteLock,
    types::permissions::UserPermissions
};

#[derive(Clone,Debug)]
pub struct RouteCollection;

/// main collector
impl RouteCollection {
    /// main route scope builder
    pub fn v1(&self) -> Scope {
        Scope::new("/v1")
            .configure(RouteCollection::health)
            .configure(RouteCollection::locations)
            .configure(RouteCollection::sessions)
            .configure(RouteCollection::secrets)
            .configure(RouteCollection::email_verification)
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
        type P = sessions::SessionsPost;
        type D = sessions::SessionsDelete;

        let user = UserPermissions::from_role(Role::User);

        cfg.service(
            actix_web::web::scope("/sessions")
                .route("", web::post().to(P::logic))
                .route("", web::delete().to(D::logic).wrap(RouteLock::default(&user)))
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

    /// email verification resource and endpoints
    pub fn email_verification(cfg: &mut web::ServiceConfig) {
        type V = verifications::email::CreateEmailVerification;
        type P = verifications::email::PatchEmailVerification;

        // let sysadmin = UserPermissions::from_role(Role::SysAdmin);

        cfg.service(
            web::scope("/verifications")
                // .wrap(RouteLock::default(&sysadmin))
                .route("/email", web::post().to(V::response))
                .route("/email/{uuid}/{id}", web::patch().to(P::response))
        );
    }

    /// images resource and endpoints
    pub fn images(_cfg: &mut web::ServiceConfig) {
        todo!()
    }

    /// secrets resource and endpoings
    pub fn locations(cfg: &mut web::ServiceConfig) {
        type G = locations::LocationsGet;

        const BASE:&str = "/locations";

        let required_permissions = UserPermissions::from_role(Role::User);
        
        cfg.service(
            web::scope(BASE)
                // PUBLIC nearest location with parameters
                .service(
                    web::resource("")
                        // nearest by zipcode: GET /locations?zipcode=77002&activity=gokarts
                        .route(
                            web::route()
                                .guard(guard::Get())
                                .guard(HasKeyValue("nearest_zipcode"))
                                .guard(HasKeyValue("activity"))
                                .to(G::public_nearest_activity_response)
                        )
                        // nearest by zipcode: GET /locations?zipcode=77002
                        .route(
                            web::route()
                                .guard(guard::Get())
                                .guard(HasKeyValue("nearest_zipcode"))
                                .to(G::public_nearest_zipcode_response)
                        )
                        // nearest by lat/lon: GET /locations?lat=...&lon=...
                        .route(
                            web::route()
                                .guard(guard::Get())
                                .guard(HasKeyValue("lat"))
                                .guard(HasKeyValue("lon"))
                                .to(G::public_nearest_point_response)
                        )
                        // default list (no matching query params): GET /locations
                        // .route(
                        //     web::route()
                        //         .guard(guard::Get())
                        //         .to(G::public_list)
                        // )
                )
                // PUBLIC
                .service(
                    web::resource("/{id}")
                        .name("locations.public.get")
                        .route(web::get().to(G::public_response))
                )
                // PRIVATE, RW Location rights required
                .service(
                    web::resource("/{id}/private")
                        .name("locations.private.get")
                        .wrap(RouteLock::default(&required_permissions))
                        .route(web::get().to(G::private_response))
                )
        );
    }

    /// secrets resource and endpoings
    pub fn secrets(cfg: &mut web::ServiceConfig) {
        let sysadmin = UserPermissions::from_role(Role::SysAdmin);

        cfg.service(
            web::scope("/secrets")
                .wrap(RouteLock::default(&sysadmin))
                .route("", actix_web::web::post().to(secrets::SecretsPost::logic))
                .route("/{id}", actix_web::web::get().to(secrets::SecretsGet::logic))
                .route("/{id}", actix_web::web::put().to(secrets::SecretsPut::logic))
                .route("/{id}", actix_web::web::patch().to(secrets::SecretsPatch::logic))
                .route("/{id}", actix_web::web::delete().to(secrets::SecretsDelete::logic))
        );
    }
}

struct HasKeyValue(&'static str);

impl guard::Guard for HasKeyValue {
    fn check(&self, ctx: &GuardContext) -> bool {
        let q = match ctx.head().uri.query() {
            Some(q) => q,
            None => return false
        };

        form_urlencoded::parse(q.as_bytes()).any(|(k, _v)| k == Cow::Borrowed(self.0))
    }
}