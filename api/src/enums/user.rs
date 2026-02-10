use database::types::DatabaseConnection;
use sqlx::prelude::FromRow;
use crate::{
    enums::{Error, UserAccountStatus, UserType},
    traits::{ToUserType,User as UserTrait},
    types::{permissions::UserPermissions, users::{StandardUser, SystemUser}}
};

type Result<T> = std::result::Result<T,Error>;

#[derive(Debug,FromRow)]
struct UserDatabaseHelper {
    user_id: i64,
    user_type_id: i8
}

#[repr(i8)]
#[derive(Clone,Debug,PartialEq)]
pub enum User {
    Standard(StandardUser) = 1,
    System(SystemUser)     = 2
}

// async
impl User {

    /// builds a standard user
    async fn standard_user(user_id: i64, account_status: Option<UserAccountStatus>, database: &DatabaseConnection) -> Result<Option<User>> {
        let user_opt = match account_status {
            Some(status) =>  StandardUser::by_id_checked(user_id,status,database).await?,
            None =>                             StandardUser::by_id_unchecked(user_id,database).await?
        };

        match user_opt {
            Some(u) => Ok(Some(User::Standard(u))),
            None => Ok(None)
        }
    }

    /// builds a system user
    async fn system_user(user_id: i64, account_status: Option<UserAccountStatus>, database: &DatabaseConnection) -> Result<Option<User>> {
        let user_opt = match account_status {
            Some(status) =>  SystemUser::by_id_checked(user_id,status,database).await?,
            None =>                             SystemUser::by_id_unchecked(user_id,database).await?
        };

        match user_opt {
            Some(u) => Ok(Some(User::System(u))),
            None => Ok(None)
        }
    }

    /// builds the user with a correct user type
    async fn build(record: &UserDatabaseHelper, account_status:Option<UserAccountStatus>, database: &DatabaseConnection) -> Result<Option<User>> {
        let user_id = record.user_id;
        let user_type = record.user_type_id.to_user_type()?;

        let user = match user_type {
            UserType::Standard => Self::standard_user(user_id,account_status,database).await?,
            UserType::System   => Self::system_user(user_id,account_status,database).await?,
        };

        Ok(user)
    }

    /// try to get user by username, on fail try UserDatabaseHelper::by_email
    pub async fn get_enabled_user(username: &str, database: &DatabaseConnection) -> Result<Option<User>> {
        let account_status = UserAccountStatus::Enabled;
        let account_status_id = account_status as i8;
        let sql = "(
                            SELECT u.id AS user_id, u.user_type_id
                            FROM person p
                            JOIN user   u ON u.id = p.id
                            WHERE p.email = ?
                        )
                        UNION ALL
                        (
                            SELECT u.id, u.user_type_id
                            FROM username un
                            JOIN user     u ON u.id = un.user_id
                            WHERE un.username = ?
                                AND NOT EXISTS (
                                    SELECT 1
                                    FROM person p2
                                    WHERE p2.email = ?
                                )
                                AND u.user_status_id = ?
                        )
                        LIMIT 1";
        let helper_opt:Option<UserDatabaseHelper> = sqlx::query_as(sql)
            .bind(username)
            .bind(username)
            .bind(username)
            .bind(account_status_id)
            .fetch_optional(&database.pool)
            .await?;

        if let Some(record) = helper_opt {
            Self::build(&record,Some(account_status),database).await
        } else {
            Ok(None)
        }
    }

    /// try to get existing user by user_id, error on row not found
    /// CAUTION: does not filter by user status, do not use in workflow that grants permissions
    pub async fn by_id_unchecked(id: i64, database: &DatabaseConnection) -> Result<Option<User>> {
        let sql = "SELECT id AS user_id, user_type_id FROM `user` WHERE user.id = ? LIMIT 1";
        let record:UserDatabaseHelper = sqlx::query_as(sql)
            .bind(id)
            .fetch_one(&database.pool)
            .await?;

        let account_status = None;

        Self::build(&record,account_status,database).await
    }

    /// try to get existing user by user_id, error on row not found, filter by enabled status
    pub async fn by_id_enabled(id: i64, database: &DatabaseConnection) -> Result<Option<User>> {
        let account_status = UserAccountStatus::Enabled;
        let account_status_id = account_status as i8;
        let sql = "SELECT id AS user_id, user_type_id FROM `user` WHERE user.id = ? AND user.user_status_id = ? LIMIT 1";
        let record:UserDatabaseHelper = sqlx::query_as(sql)
            .bind(id)
            .bind(account_status_id)
            .fetch_one(&database.pool)
            .await?;

        Self::build(&record,Some(account_status),database).await
    }

    /// try to get existing user by user_id, error on row not found, filter by disabled status
    pub async fn by_id_disabled(id: i64, database: &DatabaseConnection) -> Result<Option<User>> {
        let account_status = UserAccountStatus::Disabled;
        let account_status_id = account_status as i8;
        let sql = "SELECT id AS user_id, user_type_id FROM `user` WHERE user.id = ? AND user.user_status_id = ? LIMIT 1";
        let record:UserDatabaseHelper = sqlx::query_as(sql)
            .bind(id)
            .bind(account_status_id)
            .fetch_one(&database.pool)
            .await?;

        Self::build(&record,Some(account_status),database).await
    }

    /// returns list of users regardless of UserAccountStatus
    pub async fn list_by_id_unchecked(id_list:Vec<(i64,i8)>, connection: &DatabaseConnection) -> Result<Vec<User>> {
        let mut user_list: Vec<User> = Vec::new();
        let account_status:Option<UserAccountStatus> = None;

        for (user_id,user_type_id) in id_list {
            let user_type = user_type_id.to_user_type()?;

            let user_opt = match user_type {
                UserType::Standard => Self::standard_user(user_id,account_status,connection).await?,
                UserType::System   => Self::system_user(user_id,account_status,connection).await?,
            };

            if let Some(user) = user_opt {
                user_list.push(user);
            }
        }

        Ok(user_list)
    }

   /// returns list of users filtered by UserAccountStatus
    pub async fn list_by_id_checked(id_list:Vec<(i64,i8)>, account_status:UserAccountStatus, connection: &DatabaseConnection) -> Result<Vec<User>> {
        let mut user_list: Vec<User> = Vec::new();
        let account_status = Some(account_status);

        for (user_id,user_type_id) in id_list {
            let user_type = user_type_id.to_user_type()?;

            let user_opt = match user_type {
                UserType::Standard => Self::standard_user(user_id,account_status,connection).await?,
                UserType::System   => Self::system_user(user_id,account_status,connection).await?,
            };

            if let Some(user) = user_opt {
                user_list.push(user);
            }
        }

        Ok(user_list)
    }

    pub fn id(&self) -> i64 {
        match self {
            Self::Standard(s) => s.id(),
            Self::System(s) => s.id()
        }
    }

    pub fn epoch(&self) -> u64 {
        match self {
            Self::Standard(s) => s.epoch(),
            Self::System(s)   => s.epoch()
        }
    }

    pub fn hash(&self) -> &str {
        match self {
            Self::Standard(s) => s.hash(),
            Self::System(s)   => s.hash()
        }
    }

    pub fn permissions(&self) -> UserPermissions {
        match self {
            Self::Standard(s) => s.permissions(),
            Self::System(s)   => s.permissions()
        }
    }

    pub fn username(&self) -> &str {
        match self {
            Self::Standard(s) => s.username(),
            Self::System(s)   => s.username()
        }
    }

    pub fn user_type(&self) -> UserType {
        match self {
            Self::Standard(s) => s.user_type(),
            Self::System(s)   => s.user_type()
        }
    }

    pub fn status(&self) -> UserAccountStatus {
        match self {
            Self::Standard(s) => s.status(),
            Self::System(s)   => s.status()
        }
    }

}
