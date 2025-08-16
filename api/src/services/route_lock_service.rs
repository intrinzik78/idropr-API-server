use std::rc::Rc;
use actix_web::{
    body::{EitherBody, BoxBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    web::{Data},
    HttpResponse,
    Error
};
use futures::future::{ok, LocalBoxFuture, Ready};
use std::task::{Context, Poll};

use crate::{
    enums::{Permission,SessionControllerStatus},
    types::{AppState, AuthorizationToken, SoftwareAccess}
};

/// target for the middleware service
#[derive(Debug)]
pub struct RouteLock {
    required_rights: SoftwareAccess
}

impl RouteLock {
    pub fn default(required_rights: SoftwareAccess) -> RouteLock {
        RouteLock { required_rights }
    }
}

impl<S,B> Transform<S, ServiceRequest> for RouteLock
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = RouteLockService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RouteLockService {
            service: Rc::new(service),
            required_rights: Rc::new(self.required_rights)
        })
    }
}

#[derive(Debug)]
pub struct RouteLockService<S> {
    pub service: Rc<S>,
    pub required_rights: Rc<SoftwareAccess>
}

impl<S> RouteLockService<S> {
    fn logic(shared: &Data<AppState>, token: &str, required_rights: &SoftwareAccess) -> Permission {
        
        // extract rate limiter or return early if disabled
        let session_controller = match shared.sessions() {
            SessionControllerStatus::Enabled(sessions) => sessions,
            SessionControllerStatus::Disabled => return Permission::Granted
        };

        println!("{:?}", required_rights);

        match session_controller.permission_check(token, required_rights) {
            Ok(permission) => permission,
            Err(_) => Permission::None
        }
    }
}

impl<S, B> Service<ServiceRequest> for RouteLockService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    // wraps ServiceResponse<B> in an EitherBody
    // success: B, fail: BoxBody
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let required_rights = &self.required_rights;

        let permission_status = match AuthorizationToken::extract(&req) {
            Ok(token) => {
                req
                .app_data()
                .map_or(Permission::None, |shared: &Data<AppState>| RouteLockService::<S>::logic(shared, &token, required_rights))
            },
            Err(_) => Permission::None
        };

        println!("{:?}", permission_status);

        // return early with a Forbidden response
        if permission_status == Permission::None {
            // map fail into BoxBody
            let res = req
                .into_response(HttpResponse::Unauthorized()
                .body("Unauthorized"))
                .map_into_right_body();

            return Box::pin(async move { Ok(res) });
        }

        // return the result of the success branch
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;

            // map the success branch into the B
            Ok(res.map_into_left_body())
        })
    }
}
