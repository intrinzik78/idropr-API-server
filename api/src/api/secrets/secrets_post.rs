use actix_web::{web::{Data,Json}, Responder};
use serde::Deserialize;

use crate::{
    enums::Error,
    types::{ApiResponse, AppState, DatabaseConnection, DecryptedSecret, EncryptedSecret}
};

type Result<T> = std::result::Result<T,Error>;

#[derive(Debug,PartialEq)]
enum Available {
    No,
    Yes
}

#[derive(Deserialize)]
pub struct Post {
    name: String,
    description: String,
    api_key: Option<String>,
    api_secret: Option<String>
}

pub struct SecretsPost;

impl SecretsPost {
    pub async fn logic(post: Json<Post>, shared: Data<AppState>) -> impl Responder {
        // extract database
        let database = shared.database();

        // extract master password from settings for encryption
        let master_password = shared.settings().master_password.clone();
        
        // check for name availability
        let available = Self::try_name(&post.name, database).await.unwrap_or(Available::No);

        // return early on name-take
        if available == Available::No {
            return ApiResponse::bad_request().with_message("api name in use, try update or another name".to_string()).error();
        }

        let encrypted_result = DecryptedSecret::new(&post.name, &post.description, &post.api_key, &post.api_secret).encrypt(master_password).await;
        let encrypted = match encrypted_result {
            Ok(e) => e,
            Err(_e) => {
                // log here
                return ApiResponse::server_error().error();
            }
        };

        // build transaction
        let mut tx = match database.pool.begin().await {
            Ok(t) => t,
            Err(_) => {
                // add log here
                return ApiResponse::server_error().error();
            }
        };

        // try insert
        let _insert_id = match encrypted.into_db_as_transaction(&mut tx).await {
            Ok(id) => id,
            Err(_e) => {
                // log here
                return ApiResponse::server_error().error();
            }
        };

        match tx.commit().await {
            Ok(_) => ApiResponse::success(),
            Err(_e) => {
                // log here
                ApiResponse::server_error().error()
            }
        }

    }

    async fn try_name(name: &str, database: &DatabaseConnection) -> Result<Available> {
        let result = EncryptedSecret::by_name(name, database).await?.is_none();

        match result {
            true => Ok(Available::Yes),
            false => Ok(Available::No)
        }
    }
}