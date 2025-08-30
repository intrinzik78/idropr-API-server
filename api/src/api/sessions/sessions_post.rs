use actix_web::{web,Responder};
use blake3;
use serde::{Deserialize, Serialize};

use crate::{
    enums::{AuthorizationStatus, Error, SessionControllerStatus, User, Uuid},
     traits::{ToBase64,VerifyPassword},
     types::{ApiResponse, AppState, DatabaseConnection, DatabaseSession, KeySet, Session}
};

type Result<T> = std::result::Result<T,Error>;

#[derive(Debug,Deserialize)]
pub struct Post {
    pub username: String,
    pub password: String
}

#[derive(Debug,Serialize)]
pub struct DataContainer<'a> {
    token: &'a str
}

#[derive(Debug)]
pub struct SessionsPost;

impl SessionsPost {

    /// search for user by username first, then fallback to search by email
    async fn search_for_user(username: &str, database: &DatabaseConnection) -> Result<Option<User>> {
        let user_opt = User::user_type_by_username(username, database).await?;

        if user_opt.is_some() {
            Ok(user_opt)
        } else {
            let email_opt = User::user_type_by_email(username, database).await?;
            Ok(email_opt)
        }
    }

    /// blake 3 keyed hash for storage in database
    async fn hash_token(token: &str, uuid: Uuid) -> Result<String> {
        let uuid = match uuid {
            Uuid::Crypto(buf) => buf,
            _ => return Err(Error::SessionTokenIncorrectType)
        };
        
        let hash = blake3::keyed_hash(&uuid, token.as_bytes()).as_bytes().to_base64_url();
        
        Ok(hash)
    }

    /// endpoint entry
    pub async fn logic(post: web::Json<Post>, shared: web::Data<AppState>) -> impl Responder {
        // get database connection
        let database = shared.database();

        // extract user from database
        let user =  {
            match SessionsPost::search_for_user(&post.username, database).await {
                Ok(user_opt) => {
                    if let Some(user) = user_opt {
                        user
                    } else {
                        return ApiResponse::unauthorized().ok();
                    }
                },
                Err(_e) => {
                    // log here
                    return ApiResponse::unauthorized().ok();
                }
            }
        };

        // verify password against hash from database
        if user.verify_password(&post.password) == AuthorizationStatus::Unauthorized {
            return ApiResponse::unauthorized().ok();
        }

        // create a key set
        let key_set = match  KeySet::new() {
            Ok(set) => set,
            Err(_e) => return ApiResponse::unauthorized().ok()
        };

        // get session controller
        let session_controller = match shared.sessions() {
            SessionControllerStatus::Enabled(s) => s,
            SessionControllerStatus::Disabled => {
                // log here
                return ApiResponse::unauthorized().ok();
            }
        };

        // extract user_id for use in database session
        let user_id = user.user_id();

        // create sync session
        let session = Session::new(&key_set, user);
        
        // push to controller and accept base64 token
        let token = match session_controller.insert(session, &key_set) {
            Ok(t) => t,
            Err(_e) => {
                // log here
                return ApiResponse::unauthorized().ok();
            }
        };

        // instantiate transaction
        let mut tx = match database.pool.begin().await {
            Ok(t) => t,
            Err(_e) => {
                // log here
                return ApiResponse::unauthorized().ok();
            } 
        };

        // hash the token for insertion into the database
        let uuid = session_controller.hash_key().to_owned();
        let hash = match Self::hash_token(&token,uuid).await {
            Ok(hashed) => hashed,
            Err(_e) => {
                // log here
                return ApiResponse::unauthorized().ok();
            } 
        };

        // create db session ref
        let _ = match DatabaseSession::into_db(user_id, &hash, &mut tx).await {
            Ok(_insert_id) => (),
            Err(_e) => {
                // log here
                return ApiResponse::unauthorized().ok();
            }
        };

        //commit transaction
        let () = match tx.commit().await {
            Ok(_) => {},
            Err(_e) => {
                // log here
                match session_controller.delete(&token) {
                    _ => {
                        // log again here
                        return ApiResponse::unauthorized().ok()
                    }
                }
            }
        };

        // format and send response
        let response = DataContainer {
            token: &token
        };

        ApiResponse::default()
            .with_data(response)
            .ok()

    }
}