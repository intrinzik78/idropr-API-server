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
    enums::{AuthContext, Permission, sessions::{RefreshStatus,SessionControllerStatus}},
    types::{AppState, AuthorizationToken, permissions::{NeedCheck, PermissionCheck, UserPermissions}}
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
    async fn logic(shared: Data<AppState>, token_b64: &str, required_permissions: UserPermissions) -> Result<PermissionCheck,crate::enums::Error> {
        let failed_check =  PermissionCheck { permission: Permission::Denied, auth_context: AuthContext::None, refresh_status: RefreshStatus::None };

        // extract database from shared data
        let database = shared.database();

        // extract rate limiter or return early if disabled
        let session_controller = match shared.sessions() {
            SessionControllerStatus::Enabled(sessions) => sessions,
            SessionControllerStatus::Disabled => return Ok(failed_check)
        };
        
        // compare the user epoch to the memory epoch and refresh if out-of-sync
        let () = session_controller.epoch_check(token_b64, database).await?;

        // run the full permission check on the memory session
        let mut permissions_check = session_controller.permission_check(token_b64, required_permissions)?;

        // verify the memory session against the database session if the session is stale
        if permissions_check.refresh_status == RefreshStatus::Refresh {
            permissions_check.permission = session_controller.refresh(token_b64, database).await?;
        }

        Ok(permissions_check)
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
            // unwrap token
            let token_b64 = token_res?;
            
            // unwrap shared data
            let shared = shared_res?;
            
            // run permission check, unwrap or handle response
            let check = match RouteLockService::<S>::logic(shared, &token_b64, required_permissions).await {
                Ok(c) => c,
                Err(_e) => {
                    // log error here
                    println!("{_e}");
                    // return unauthorized response on check-error
                    let res = req
                        .into_response(HttpResponse::Unauthorized()
                        .body("Unauthorized"))
                        .map_into_right_body();
                    
                    return Ok(res)
                }
            };

            // fail: short circuit, success: forward permissions and context to endpoint
            if check.permission  == Permission::Denied {
                // short circuit on fail
                let res = req
                    .into_response(HttpResponse::Unauthorized()
                    .body("Unauthorized"))
                    .map_into_right_body();
                
                return Ok(res)
            } else {
                // forward context and permissions to enpdoint
                req.extensions_mut().insert(NeedCheck(required_permissions));
                req.extensions_mut().insert(check.auth_context);
            }

            // build future and map the response into the success body
            let res = service.call(req)
                .await?
                .map_into_left_body();

            // map response into success branch
            Ok(res)
        })
    }
}
