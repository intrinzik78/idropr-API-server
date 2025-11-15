use crate::{
    enums::{AuthorizationStatus, User},
    traits::ToAuthorizationStatus
};

/// guaranteed not to return an error
pub trait VerifyPassword {
    fn verify_password(&self, password: &str) -> impl std::future::Future<Output = AuthorizationStatus> + Send;
}

impl VerifyPassword for User {
    async fn verify_password(&self, password: &str) -> AuthorizationStatus {
        // retreive user hash
        let hash_ref = self.hash();

        // clone for thread safety
        let password = password.to_owned();
        let hash = hash_ref.to_owned();

        // runs cpu intense bcrypt on a blocking thread
        let join_result = actix_rt::task::spawn_blocking(move || { bcrypt::verify(password, &hash) }).await;

        // unwrap and log errors by layer
        if let Ok(bcrypt_result) = join_result {
            match bcrypt_result {
                Ok(b) => b.to_authorization_status(),
                Err(_e) => {
                    // add log here for inner error
                    AuthorizationStatus::Unauthorized
                }
            }
        } else {
            // add log here for outer error
            AuthorizationStatus::Unauthorized
        }
    }
}


