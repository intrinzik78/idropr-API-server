use database::types::DatabaseConnection;
use sqlx::FromRow;

use crate::{
    enums::{Error,UserAccountStatus, UserType},
    traits::{ToUserAccountStatus,User},
    types::permissions::UserPermissions
};

type Result<T> = std::result::Result<T,Error>;

#[derive(Debug,FromRow)]
struct DatabaseHelper {
    id: i64,
    epoch:u64,
    username: String,
    hash: String,
    user_status_id: i8,
}

impl DatabaseHelper {
    /// consumes self and returns the SystemUser
    async fn transform(self, database: &DatabaseConnection) -> Result<SystemUser> {
        let status = self.user_status_id.to_user_account_status()?;
        let permissions = UserPermissions::by_user_id(self.id, database).await?;
        
        let user = SystemUser {
            id: self.id,
            epoch: self.epoch,
            username: self.username,
            hash: self.hash,
            status,
            permissions
        };

        Ok(user)
    }
}

#[derive(Clone,Debug,PartialEq)]
pub struct SystemUser {
    id: i64,
    epoch: u64,
    username: String,
    hash: String,
    status: UserAccountStatus,
    permissions: UserPermissions
}

impl User<SystemUser> for SystemUser {
    fn id(&self) -> i64 { self.id }
    
    fn epoch(&self) -> u64 { self.epoch }
    
    fn hash(&self) -> &str { &self.hash }

    fn permissions(&self) -> UserPermissions { self.permissions }
    
    fn username(&self) -> &str { &self.username }
    
    fn status(&self) -> UserAccountStatus { self.status }
    
    fn user_type(&self) -> UserType { UserType::System }

    fn new(builder:super::Builder) -> Result<Self> {
        type E = Error;
        let id                  = builder.id.ok_or(E::RequiredUserBuildDataMissing)?;
        let epoch               = builder.epoch.ok_or(E::RequiredUserBuildDataMissing)?;
        let username         = builder.username.ok_or(E::RequiredUserBuildDataMissing)?;
        let hash             = builder.hash.ok_or(E::RequiredUserBuildDataMissing)?;
        let status= builder.user_status.ok_or(E::RequiredUserBuildDataMissing)?;
        let permissions = builder.permissions.ok_or(E::RequiredUserBuildDataMissing)?;

        let system_user:SystemUser = SystemUser {
            id,
            epoch,
            username,
            hash,
            status,
            permissions
        };

        Ok(system_user)
    }

    /// builds a system user from a database record by user_id
    async fn by_id_unchecked(user_id: i64, database: &DatabaseConnection) -> Result<Option<SystemUser>> {
        let sql = "SELECT user.id,user.epoch,username.username,user.hash,user.user_status_id,user_type_id FROM `user` JOIN `system_users` ON user.id = system_users.user_id JOIN `username` ON user.id = username.user_id WHERE user.id = ?";
        let helper_opt:Option<DatabaseHelper> = sqlx::query_as(sql)
            .bind(user_id)
            .fetch_optional(&database.pool)
            .await?;

        if let Some(helper) = helper_opt {
            let user = helper.transform(database).await?;
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    /// builds a system user from a database record by user_id
    /// filters by any UserAccountStatus variant
    async fn by_id_checked(user_id: i64,account_status: UserAccountStatus, database: &DatabaseConnection) -> Result<Option<SystemUser>> {
        let account_status_id = account_status as i8;
        let sql = "SELECT user.id,user.epoch,username.username,user.hash,user.user_status_id,user_type_id FROM `user` JOIN `system_users` ON user.id = system_users.user_id JOIN `username` ON user.id = username.user_id WHERE user.id = ? AND user.user_status_id = ?";
        let helper_opt:Option<DatabaseHelper> = sqlx::query_as(sql)
            .bind(user_id)
            .bind(account_status_id)
            .fetch_optional(&database.pool)
            .await?;

        if let Some(helper) = helper_opt {
            let user = helper.transform(database).await?;
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }
}