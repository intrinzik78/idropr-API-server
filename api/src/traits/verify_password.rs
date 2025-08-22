use crate::enums::{AuthorizationStatus, User};

/// guaranteed not to return an error
pub trait VerifyPassword {
    fn verify_password(&self, password: &str) -> AuthorizationStatus;
}

impl VerifyPassword for User {
    fn verify_password(&self, password: &str) -> AuthorizationStatus {
        match self {
            User::System(u) => u.is_authorized(password),
            User::Business(u) => u.is_authorized(password),
            User::Community(u) => u.is_authorized(password)
        }
    }
}