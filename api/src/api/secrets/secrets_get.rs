use actix_web::{Responder,web::{Data,Path}};
use serde::Deserialize;

use crate::{enums::ApiResult, types::{AppState, permissions::WereChecked}};

#[derive(Deserialize)]
pub struct ReqPath {
    pub id:String
}

pub struct SecretsGet;


impl SecretsGet {
    pub async fn logic(_permissions: WereChecked, path: Path<ReqPath>, shared: Data<AppState>) -> impl Responder {
        let _database = shared.database();

        let _id = path.into_inner().id;

        ApiResult::success().to_http()
    }
}

