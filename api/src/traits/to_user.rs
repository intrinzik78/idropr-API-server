use crate::{
    enums::{AuthContext,User},
    types::users::{BusinessUser,CommunityUser,SystemUser}
};

pub trait ToUser {
    fn to_user(&self) -> Option<&Box<User>>;
    fn to_business_user(&self) -> Option<&BusinessUser>;
    fn to_community_user(&self) -> Option<&CommunityUser>;
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
    fn to_business_user(&self) -> Option<&BusinessUser> {
        let auth_context = match *self {
            Some(ac) => ac,
            None => return None 
        };

        let boxed_user = match auth_context {
            AuthContext::Some(b) => b,
            AuthContext::None => return None
        };

        match &**boxed_user {
            User::Business(b) => Some(b),
            _ => None
        }
    }

    #[inline]
    fn to_community_user(&self) -> Option<&CommunityUser> {
        let auth_context = match *self {
            Some(ac) => ac,
            None => return None 
        };

        let boxed_user = match auth_context {
            AuthContext::Some(b) => b,
            AuthContext::None => return None
        };

        match &**boxed_user {
            User::Community(c) => Some(&c),
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