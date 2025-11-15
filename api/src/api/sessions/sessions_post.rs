use actix_web::{web,Responder};
use blake3;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    enums::{AuthorizationStatus, Error, sessions::SessionControllerStatus, User, Uuid},
    traits::VerifyPassword,
    types::{ApiResponse, AppState, sessions::{DatabaseSession, KeySet, Session}}
};

type Result<T> = std::result::Result<T,Error>;

 /// requires a user identifier (email,username) and the associated password
#[derive(Debug,Deserialize,ToSchema)]
pub struct CreateSessionBody {
    pub username: String,
    pub password: String
}

/// bearer token to use in `Authorization: Bearer <token>`
#[derive(Debug,Serialize,ToSchema)]
pub struct AccessToken<'a> {
    /// required to make permissioned requests to the API
    access_token: &'a str,
}

#[derive(Debug)]
pub struct SessionsPost;

impl SessionsPost {

    /// blake 3 keyed hash for storage in database
    #[inline]
    fn hash_token(token: &str, uuid: Uuid) -> Result<blake3::Hash> {
        let uuid = match uuid {
            Uuid::Crypto(buf) => buf,
            _ => return Err(Error::SessionTokenIncorrectType)
        };
        
        let hash = blake3::keyed_hash(&uuid, token.as_bytes());
        
        Ok(hash)
    }

    /// endpoint entry
    pub async fn logic(post: web::Json<CreateSessionBody>, shared: web::Data<AppState>) -> impl Responder {
        // get database connection
        let database = shared.database();

        // extract user from database
        let user =  {
            match User::get_enabled_user(&post.username, database).await {
                Ok(user_opt) => {
                    if let Some(user) = user_opt {
                        user
                    } else {
                        return ApiResponse::unauthorized().error();
                    }
                },
                Err(e) => {
                    // log here
                    println!("{e}");
                    return ApiResponse::unauthorized().error();
                }
            }
        };

        // verify password against hash from database
        if user.verify_password(&post.password).await == AuthorizationStatus::Unauthorized {
            return ApiResponse::unauthorized().error();
        }

        // create a key set
        let key_set = match  KeySet::new() {
            Ok(set) => set,
            Err(_e) => return ApiResponse::unauthorized().error()
        };

        // get session controller
        let session_controller = match shared.sessions() {
            SessionControllerStatus::Enabled(s) => s,
            SessionControllerStatus::Disabled => {
                // log here
                return ApiResponse::unauthorized().error();
            }
        };

        // extract user_id for use in database session
        let user_id = user.id();

        // create sync session
        let session = Session::new(&key_set, user);
        
        // push to controller and accept base64 token
        let token = match session_controller.insert(session, &key_set) {
            Ok(t) => t,
            Err(_e) => {
                // log here
                return ApiResponse::unauthorized().error();
            }
        };

        // hash the token for insertion into the database
        let uuid = session_controller.hash_key().to_owned();
        let hash = match Self::hash_token(&token,uuid) {
            Ok(hashed) => hashed,
            Err(_e) => {
                // log here
                return ApiResponse::unauthorized().error();
            } 
        };

        // create db session ref
        let () = match DatabaseSession::into_db(user_id, &hash, database).await {
            Ok(_insert_id) => (),
            Err(_e) => {
                // log here
                return ApiResponse::unauthorized().error();
            }
        };

        // format and send response
        let response = AccessToken {
            access_token: &token
        };

        ApiResponse::default()
            .with_data(response)
            .ok()

    }
}