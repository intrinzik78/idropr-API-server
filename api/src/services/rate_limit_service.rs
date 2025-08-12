use std::rc::Rc;
use actix_web::{
    body::{EitherBody, BoxBody},
    dev::{ConnectionInfo, Service, ServiceRequest, ServiceResponse, Transform},
    web::Data,
    HttpResponse,
    Error
};
use futures::future::{ok, LocalBoxFuture, Ready};
use rate_limit::enums::Decision;
use std::task::{Context, Poll};

use crate::{enums::RateLimiterStatus, types::AppState};

/// target for the middleware service
#[derive(Debug,Default)]
pub struct RateLimitMiddleware;

impl<S,B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = RateLimitService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimitService {
            service: Rc::new(service),
        })
    }
}

#[derive(Debug)]
pub struct RateLimitService<S> {
    service: Rc<S>,
}

impl<S> RateLimitService<S> {
    fn logic(shared: &Data<AppState>,connection: &ConnectionInfo) -> Decision {

        // extract rate limiter or return early if disabled
        let rate_limit_handle = match shared.rate_limiter() {
            RateLimiterStatus::Enabled(limiter) => limiter,
            RateLimiterStatus::Disabled => return Decision::Approved
        };

        // Decision or None
        let decision_opt= connection
            .realip_remote_addr()
            .or(connection.peer_addr())
            .and_then(|ip| rate_limit_handle.try_connect(ip).ok());

        // deny on None (no valid ip found)
        if let Some(decision) = decision_opt {
            decision
        } else {
            Decision::Denied
        }
    }
}

impl<S, B> Service<ServiceRequest> for RateLimitService<S>
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
        let rate_limiter_status = req
            .app_data()
            .map_or(Decision::Denied, |shared: &Data<AppState>| {
                let connection = req.connection_info().clone();
                RateLimitService::<S>::logic(shared, &connection)
            });

        // return early with a Forbidden response
        if rate_limiter_status == Decision::Denied {
            // map fail into BoxBody
            let res = req
                .into_response(HttpResponse::Forbidden().body("Forbidden"))
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
