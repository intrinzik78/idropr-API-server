use actix_web::{web::{Data,Json}, Responder};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    enums::{ApiResult, Error},
    types::{AppState, permissions::WereChecked, secrets::DecryptedSecret}
};

#[derive(Debug,Deserialize,ToSchema)]
pub struct CreateSecretBody {
    name: String,
    description: String,
    api_key: Option<String>,
    api_secret: Option<String>
}

pub struct SecretsPost;

impl SecretsPost {
    pub async fn logic(_permissions: WereChecked, post: Json<CreateSecretBody>, shared: Data<AppState>) -> impl Responder {
        type E = Error;

        let controller = shared.secrets();
        let database = shared.database();
        let secret = DecryptedSecret::new(&post.name, &post.description, &post.api_key, &post.api_secret);

        match controller.new_secret(secret, database).await {
            Ok(()) => return ApiResult::no_content().to_http(),
            Err(e) => {
                let error_opt = e.to_api_error_message();

                if let Some(d) = error_opt {
                    match e {
                        E::LocationRecordNotFoundById       => ApiResult::not_found().with_reason(d).to_http(),
                        E::DatabaseTransactionVerification  => ApiResult::server_error().with_reason(d).to_http(),
                        _ => ApiResult::server_error().to_http()
                    }
                } else {
                    ApiResult::server_error().to_http()
                }
            }
        }
    }
}