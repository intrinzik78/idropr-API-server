use crate::{
    enums::{AuthContext,User},
    types::users::{StandardUser,SystemUser}
};

pub trait ToUser {
    fn to_user(&self) -> Option<&Box<User>>;
    fn to_standard_user(&self) -> Option<&StandardUser>;
    fn to_system_user(&self) -> Option<&SystemUser>;
}

impl ToUser for Option<&AuthContext> {

    #[inline]
    fn to_user(&self) -> Option<&Box<User>> {
        let auth_context = match *self {
            Some(ac) => ac,
            None => return None
        };

        match auth_context {
            AuthContext::Some(b) => Some(b),
            AuthContext::None => return None
        }
    }

    #[inline]
    fn to_standard_user(&self) -> Option<&StandardUser> {
        let auth_context = match *self {
            Some(ac) => ac,
            None => return None
        };

        let boxed_user = match auth_context {
            AuthContext::Some(b) => b,
            AuthContext::None => return None
        };

        match &**boxed_user {
            User::Standard(s) => Some(s),
            _ => None
        }
    }

    #[inline]
    fn to_system_user(&self) -> Option<&SystemUser> {
        let auth_context = match *self {
            Some(ac) => ac,
            None => return None
        };

        let boxed_user = match auth_context {
            AuthContext::Some(b) => b,
            AuthContext::None => return None
        };

        match &**boxed_user {
            User::System(s) => Some(&s),
            _ => None
        }
    }
}
