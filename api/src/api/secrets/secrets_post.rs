use actix_web::{web::{Data,Json}, Responder};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    enums::Error,
    types::{ApiErrorData,ApiResponse, AppState, permissions::WereChecked, secrets::DecryptedSecret}
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
            Ok(()) => return ApiResponse::resource_created().ok(),
            Err(e) => {
                let error_opt = e.to_api_error_message();

                if let Some(d) = error_opt {
                    match e {
                        E::LocationRecordNotFoundById       => ApiResponse::<ApiErrorData>::default().with_code(400).with_data(d).error(),
                        E::DatabaseTransactionVerification  => ApiResponse::<ApiErrorData>::default().with_code(500).with_data(d).error(),
                        _ => ApiResponse::server_error().error()
                    }
                } else {
                    ApiResponse::server_error().error()
                }
            }
        }
    }
}