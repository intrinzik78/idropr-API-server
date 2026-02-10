use crate::enums::{Error, User, UserType};

type Result<T> = std::result::Result<T,Error>;

pub trait ToUserType {
    fn to_user_type(&self) -> Result<UserType>;
}

impl ToUserType for i8 {
    #[inline]
    fn to_user_type(&self) -> Result<UserType> {
        let user_type = match self {
            ..1 => return Err(Error::UserTypeOutOfBounds),
            1   => UserType::Standard,
            2   => UserType::System,
            3.. => return Err(Error::UserTypeOutOfBounds)
        };

        Ok(user_type)
    }
}

impl ToUserType for User {
    #[inline]
    fn to_user_type(&self) -> Result<UserType> {
        let user_type = match self {
            User::Standard(_) => UserType::Standard,
            User::System(_) => UserType::System
        };

        Ok(user_type)
    }
}
