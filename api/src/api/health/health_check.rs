use actix_web::{HttpResponse, Responder};
pub struct HealthCheck;

impl HealthCheck {
    pub async fn logic() -> impl Responder {
        HttpResponse::Ok()
    }
}