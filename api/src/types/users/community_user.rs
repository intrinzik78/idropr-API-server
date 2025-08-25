use sqlx::FromRow;

use crate::{
    enums::{Error,UserAccountStatus,AuthorizationStatus},
    traits::{ToUserAccountStatus,ToAuthorizationStatus},
    types::{DatabaseConnection, UserPermissions}
};

type Result<T> = std::result::Result<T,Error>;

#[derive(Debug,FromRow)]
struct DatabaseHelper {
    id: i64,
    username: String,
    hash: String,
    user_status_id: i8,
    #[allow(dead_code)]
    user_type_id: i8
}

impl DatabaseHelper {
    /// consumes self and returns the BusinessUser
    async fn transform(self, database: &DatabaseConnection) -> Result<CommunityUser> {
        let status = self.user_status_id.to_user_account_status()?;
        let permissions = UserPermissions::by_user_id(self.id, database).await?;
        
        let user = CommunityUser {
            id: self.id,
            username: self.username,
            hash: self.hash,
            status,
            permissions
        };

        Ok(user)
    }
}

#[derive(Clone,Debug,PartialEq)]
pub struct CommunityUser {
    pub id: i64,
    pub username: String,
    pub hash: String,
    pub status: UserAccountStatus,
    pub permissions: UserPermissions
}


impl CommunityUser {
    /// builds a business user from a database record by user_id
    pub async fn by_id(user_id: i64, database: &DatabaseConnection) -> Result<Option<CommunityUser>> {
        let sql = "SELECT user.id,username.username,user.hash,user.user_status_id,user_type_id FROM `user` JOIN `community_users` ON user.id = community_users.user_id JOIN `username` ON user.id = username.user_id WHERE user.id = ?";
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

    pub fn verify_password(&self, password: &str) -> AuthorizationStatus {
        match bcrypt::verify(password, &self.hash) {
            Ok(b)  => b.to_authorization_status(),
            Err(_e) => AuthorizationStatus::Unauthorized
        }
    }
}