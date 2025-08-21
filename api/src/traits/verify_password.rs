use crate::enums::{User, VerificationStatus};

/// guaranteed not to return an error
pub trait VerifyPassword {
    fn verify_password(&self, password: &str) -> VerificationStatus;
}

impl VerifyPassword for User {
    fn verify_password(&self, password: &str) -> VerificationStatus {
        match self {
            User::System(u) => u.verify(password),
            User::Business(u) => u.verify(password),
            User::Community(u) => u.verify(password)
        }
    }
}