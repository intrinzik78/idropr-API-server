use actix_web::{web::Data,HttpRequest,Responder};

use crate::{types::{ApiResponse,AppState}};

pub struct SecretsGet;

impl SecretsGet {
    pub async fn logic(_req: HttpRequest, _shared: Data<AppState>) -> impl Responder {
        ApiResponse::success()
    }
}