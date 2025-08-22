use crate::enums::AuthorizationStatus;

pub trait ToAuthorizationStatus {
    fn to_authorization_status(&self) -> AuthorizationStatus;
}

impl ToAuthorizationStatus for bool {
    fn to_authorization_status(&self) -> AuthorizationStatus {
        match self {
            true  => AuthorizationStatus::Authorized,
            false => AuthorizationStatus::Unauthorized
        }
    }
}