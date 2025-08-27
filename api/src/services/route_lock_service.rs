use std::rc::Rc;
use actix_web::{
    body::{BoxBody, EitherBody}, dev::{Service, ServiceRequest, ServiceResponse, Transform}, web::Data, Error, HttpMessage, HttpResponse
};
use futures::future::{ok, LocalBoxFuture, Ready};
use std::task::{Context, Poll};

use crate::{
    enums::{Permission,SessionControllerStatus},
    types::{AppState, AuthorizationToken, NeedCheck, UserPermissions}
};

/// target for the middleware service
#[derive(Clone,Debug)]
pub struct RouteLock {
    required_permissions: UserPermissions
}

impl RouteLock {
    pub fn default(required_permissions: &UserPermissions) -> RouteLock {
        let required_permissions = required_permissions.to_owned();
        RouteLock { required_permissions }
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
            required_permissions: Rc::new(self.required_permissions.clone())
        })
    }
}

#[derive(Debug)]
pub struct RouteLockService<S> {
    pub service: Rc<S>,
    pub required_permissions: Rc<UserPermissions>
}

impl<S> RouteLockService<S> {
    fn logic(shared: &Data<AppState>, token: &str, required_permissions: &UserPermissions) -> Permission {
        
        // extract rate limiter or return early if disabled
        let session_controller = match shared.sessions() {
            SessionControllerStatus::Enabled(sessions) => sessions,
            SessionControllerStatus::Disabled => return Permission::Granted
        };

        match session_controller.permission_check(token, required_permissions) {
            Ok(permission) => permission,
            Err(_) => Permission::Denied
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
        let required_permissions = &self.required_permissions;

        let permission_status = match AuthorizationToken::extract(&req) {
            Ok(token) => {
                req
                .app_data()
                .map_or(Permission::Denied, |shared: &Data<AppState>| RouteLockService::<S>::logic(shared, &token, required_permissions))
            },
            Err(_) => Permission::Denied
        };

        // return early with a Forbidden response
        if permission_status == Permission::Denied {
            // map fail into BoxBody
            let res = req
                .into_response(HttpResponse::Unauthorized()
                .body("Unauthorized"))
                .map_into_right_body();

            return Box::pin(async move { Ok(res) });
        } else {
            let checked_permissions = NeedCheck(required_permissions.clone());
            req.extensions_mut().insert(checked_permissions);
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
