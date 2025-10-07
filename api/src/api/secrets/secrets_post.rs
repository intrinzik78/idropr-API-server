use actix_web::{web::{Data,Json}, Responder};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    enums::Error,
    types::{ApiResponse, AppState, secrets::DecryptedSecret, permissions::WereChecked}
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
        let controller = shared.secrets();
        let database = shared.database();
        let secret = DecryptedSecret::new(&post.name, &post.description, &post.api_key, &post.api_secret);

        match controller.new_secret(secret, database).await {
            Ok(()) => ApiResponse::resource_created().ok(),
            Err(Error::DuplicateSecretNameExists) => ApiResponse::bad_request().with_message("duplicate: api name already in use".to_string()).error(),
            Err(e) => {
                // add log here
                println!("{e}");
                ApiResponse::server_error().error()
            }
        }
    }
}