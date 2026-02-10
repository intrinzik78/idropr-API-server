use actix_web::Responder;

use crate::enums::ApiResult;

pub struct SecretsDelete;

impl SecretsDelete {
    pub async fn logic() -> impl Responder {
        ApiResult::success().to_http()
    }
}