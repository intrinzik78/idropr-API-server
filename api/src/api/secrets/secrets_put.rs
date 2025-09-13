use actix_web::{Responder,web::Data};
use serde::Deserialize;

use crate::{types::{ApiResponse,AppState}};

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct ReqPath {
    pub id:String
}

pub struct SecretsPut;


impl SecretsPut {
    pub async fn logic(_shared: Data<AppState>) -> impl Responder {
        ApiResponse::success()
    }
}

