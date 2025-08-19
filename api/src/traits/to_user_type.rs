use crate::enums::{Error, User, UserType};

type Result<T> = std::result::Result<T,Error>;

pub trait ToUserType {
    fn to_user_type(&self) -> Result<UserType>;
}

impl ToUserType for i8 {
    fn to_user_type(&self) -> Result<UserType> {
        let user_type = match self {
            ..0 => return Err(Error::UserTypeOutOfBounds),
            0   => UserType::Business,
            1   => UserType::Community,
            2   => UserType::System,
            3.. => return Err(Error::UserTypeOutOfBounds)
        };

        Ok(user_type)
    }
}

impl ToUserType for User {
    fn to_user_type(&self) -> Result<UserType> {
        let user_type = match self {
            User::Business(_) => UserType::Business,
            User::Community(_) => UserType::Community,
            User::System(_) => UserType::System
        };

        Ok(user_type)
    }
}