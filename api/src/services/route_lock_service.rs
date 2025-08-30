use std::rc::Rc;
use actix_web::{
    body::{BoxBody, EitherBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    web::Data,
    Error,
    HttpMessage,
    HttpResponse
};
use futures::future::{ok, LocalBoxFuture, Ready};
use std::task::{Context, Poll};

use crate::{
    enums::{AuthContext, Permission, RefreshStatus, SessionControllerStatus},
    types::{AppState, AuthorizationToken, NeedCheck, PermissionCheck, UserPermissions}
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
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B,BoxBody>>;
    type Error = Error;
    type Transform = RouteLockService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RouteLockService {
            service: Rc::new(service),
            required_permissions: Rc::new(self.required_permissions)
        })
    }
}

#[derive(Debug)]
pub struct RouteLockService<S> {
    pub service: Rc<S>,
    pub required_permissions: Rc<UserPermissions>
}

impl<S> RouteLockService<S> {
    async fn logic(shared: Data<AppState>, token: &str, required_permissions: UserPermissions) -> PermissionCheck {
        let failed_check =  PermissionCheck { permission: Permission::Denied, auth_context: AuthContext::None, refresh_status: RefreshStatus::None };

        // extract rate limiter or return early if disabled
        let session_controller = match shared.sessions() {
            SessionControllerStatus::Enabled(sessions) => sessions,
            SessionControllerStatus::Disabled => return failed_check
        };

        let mut permissions_check = match session_controller.permission_check(token, required_permissions) {
            Ok(perm_check) => perm_check,
            Err(_) => return failed_check
        };

        if permissions_check.refresh_status == RefreshStatus::Refresh {
            // extract user container
            let user_id = match &permissions_check.auth_context {
                AuthContext::Some(boxed_user) => boxed_user.user_id(),
                AuthContext::None => return failed_check
            };

            // extract database from shared data
            let database = shared.database();

            // retrieve session from database
            permissions_check.permission = match session_controller.refresh(user_id, token, database).await {
                Ok(permission) => permission,
                Err(_e) => return failed_check
            };
        }

        permissions_check
    }
    
}

impl<S, B> Service<ServiceRequest> for RouteLockService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
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

    fn call(& self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let required_permissions = *self.required_permissions;
        let token_res = AuthorizationToken::extract(&req)
            .map (|s| s.to_owned())
            .map_err(|_e| actix_web::error::ErrorUnauthorized("Unauthorized"));
        let shared_res = req
            .app_data::<Data<AppState>>()
            .cloned()
            .ok_or(actix_web::error::ErrorUnauthorized("Unauthorized"));

        Box::pin(async move {
            let token = token_res?;
            let shared = shared_res?;
            
            let check = RouteLockService::<S>::logic(shared, &token, required_permissions).await;

            if check.permission  == Permission::Denied {
                let res = req
                    .into_response(HttpResponse::Unauthorized()
                    .body("Unauthorized"))
                    .map_into_right_body();
                
                return Ok(res)
            } else {
                req.extensions_mut().insert(NeedCheck(required_permissions));
                req.extensions_mut().insert(check.auth_context);
            }

            // build future
            let res = service.call(req).await?;

            // map response into success branch
            Ok(res.map_into_left_body())
        })
    }
}
