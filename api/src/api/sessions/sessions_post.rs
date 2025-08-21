use actix_web::{web,Responder};
use serde::{Deserialize, Serialize};

use crate::{enums::{Error, SessionControllerStatus, User, VerificationStatus}, traits::VerifyPassword, types::{ApiResponse, AppState, DatabaseConnection, KeySet, Session}};

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
    async fn search_for_user(username: &str, database: &DatabaseConnection) -> Result<Option<User>,Error> {
        let user_opt = User::user_type_by_username(username, database).await?;

        if user_opt.is_some() {
            Ok(user_opt)
        } else {
            let email_opt = User::user_type_by_email(username, database).await?;
            Ok(email_opt)
        }
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
        if user.verify_password(&post.password) == VerificationStatus::Unverified {
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

        // create session
        let session = Session::new(&key_set, user);

        // push to controller and accept base64 token
        let token = match session_controller.insert(session, &key_set) {
            Ok(t) => t,
            Err(_e) => {
                // log here
                return ApiResponse::unauthorized().ok();
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