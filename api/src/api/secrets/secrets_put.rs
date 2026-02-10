use actix_web::{Responder,web::Data};
use serde::Deserialize;

use crate::{
    enums::ApiResult,
    types::AppState
};

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct ReqPath {
    pub id:String
}

pub struct SecretsPut;


impl SecretsPut {
    pub async fn logic(_shared: Data<AppState>) -> impl Responder {
        ApiResult::success().to_http()
    }
}

