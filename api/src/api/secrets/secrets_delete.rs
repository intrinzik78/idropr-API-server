use actix_web::Responder;

use crate::types::ApiResponse;

pub struct SecretsDelete;

impl SecretsDelete {
    pub async fn logic() -> impl Responder {
        ApiResponse::success()
    }
}