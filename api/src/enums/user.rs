use sqlx::prelude::FromRow;

use crate::{
    traits::ToUserType,
    enums::{Error,UserType},
    types::{
        DatabaseConnection,
        users::{BusinessUser,CommunityUser,SystemUser}
    }
};

type Result<T> = std::result::Result<T,Error>;

#[derive(Debug,FromRow)]
struct UserDatabaseHelper {
    user_id: i64,
    user_type_id: i8
}

#[derive(Clone,Debug,PartialEq)]
pub enum User {
    Business(BusinessUser),     // 0
    Community(CommunityUser),   // 1
    System(SystemUser),         // 2
}

// async
impl User {

    /// builds a business user
    async fn business_user(user_id: i64, database: &DatabaseConnection) -> Result<Option<User>> {
        let user_opt = BusinessUser::by_id(user_id,database).await?;

        match user_opt {
            Some(u) => Ok(Some(User::Business(u))),
            None => Ok(None)
        }
    }

    /// builds a community user
    async fn community_user(user_id: i64, database: &DatabaseConnection) -> Result<Option<User>> {
        let user_opt = CommunityUser::by_id(user_id,database).await?;

        match user_opt {
            Some(u) => Ok(Some(User::Community(u))),
            None => Ok(None)
        }
    }

    /// builds a system user
    async fn system_user(user_id: i64, database: &DatabaseConnection) -> Result<Option<User>> {
        let user_opt = SystemUser::by_id(user_id,database).await?;

        match user_opt {
            Some(u) => Ok(Some(User::System(u))),
            None => Ok(None)
        }
    }

    /// parses user_type_id to build the correct user type
    async fn parse_user_type(record: &UserDatabaseHelper, database: &DatabaseConnection) -> Result<Option<User>> {
            let user_id = record.user_id;
            let user_type = record.user_type_id.to_user_type()?;

            let user = match user_type {
                UserType::Business => Self::business_user(user_id,database).await?,
                UserType::Community => Self::community_user(user_id,database).await?,
                UserType::System => Self::system_user(user_id,database).await?,
            };

            Ok(user)
    }

    /// try to get user by username, on fail try UserDatabaseHelper::by_email
    pub async fn user_type_by_username(username: &str, database: &DatabaseConnection) -> Result<Option<User>> {
        let sql = "SELECT user_id,user_type_id FROM `user` JOIN `username` ON user.id = username.user_id WHERE username.username = ?";
        let helper_opt:Option<UserDatabaseHelper> = sqlx::query_as(sql)
            .bind(username)
            .fetch_optional(&database.pool)
            .await?;

        if let Some(record) = helper_opt {
            Self::parse_user_type(&record,database).await
        } else {
            Ok(None)
        }
    }

    /// try to get user by user's email
    pub async fn user_type_by_email(email: &str, database: &DatabaseConnection) -> Result<Option<User>> {
        let sql = "SELECT user.id as user_id, user_type_id FROM `user` JOIN `person` ON person.id = user.id WHERE person.email = ?";
        let helper_opt:Option<UserDatabaseHelper> = sqlx::query_as(sql)
            .bind(email)
            .fetch_optional(&database.pool)
            .await?;

        if let Some(record) = helper_opt {
            Self::parse_user_type(&record,database).await
        } else {
            Ok(None)
        }
    }
}

// sync
impl User {
    pub fn user_id(&self) -> i64 {
        match self {
            User::Business(b)   => b.id,
            User::Community(c) => c.id,
            User::System(s)       => s.id
        }
    }
}