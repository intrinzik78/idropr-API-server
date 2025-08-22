use crate::enums::{AuthorizationStatus, User};

/// guaranteed not to return an error
pub trait VerifyPassword {
    fn verify_password(&self, password: &str) -> AuthorizationStatus;
}

impl VerifyPassword for User {
    fn verify_password(&self, password: &str) -> AuthorizationStatus {
        match self {
            User::System(u) => u.verify_password(password),
            User::Business(u) => u.verify_password(password),
            User::Community(u) => u.verify_password(password)
        }
    }
}