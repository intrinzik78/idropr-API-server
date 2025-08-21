use sqlx::FromRow;

use crate::{
    enums::{Error,UserAccountStatus,VerificationStatus},
    traits::{ToUserAccountStatus,ToVerificationStatus},
    types::{DatabaseConnection, UserPermissions}
};

type Result<T> = std::result::Result<T,Error>;

#[derive(Debug,FromRow)]
struct DatabaseHelper {
    id: i64,
    business_account_id: i64,
    username: String,
    hash: String,
    user_status_id: i8,
    #[allow(dead_code)]
    user_type_id: i8
}

#[derive(Clone,Debug,PartialEq)]
pub struct BusinessUser {
    pub id: i64,
    pub business_account_id: i64,
    pub username: String,
    pub hash: String,
    pub status: UserAccountStatus,
    pub permissions: UserPermissions
}

impl DatabaseHelper {
    /// consumes self and returns the BusinessUser
    async fn transform(self, database: &DatabaseConnection) -> Result<BusinessUser> {
        let status = self.user_status_id.to_user_account_status()?;
        let permissions = UserPermissions::from_user_id(self.id, database).await?;
        
        let user = BusinessUser {
            id: self.id,
            business_account_id: self.business_account_id,
            username: self.username,
            hash: self.hash,
            status,
            permissions
        };

        Ok(user)
    }
}

impl BusinessUser {
    /// builds a business user from a database record by user_id
    pub async fn by_id(user_id: i64, database: &DatabaseConnection) -> Result<Option<BusinessUser>> {
        let sql = "SELECT user.id,business_account_users.business_account_id,username.username,user.hash,user.user_status_id,user_type_id FROM `user` JOIN `business_account_users` ON user.id = business_account_users.user_id JOIN `username` ON user.id = username.user_id WHERE user.id = ?";
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

    pub fn verify(&self, password: &str) -> VerificationStatus {
        match bcrypt::verify(password, &self.hash) {
            Ok(b)  => b.to_verification_status(),
            Err(_e) => VerificationStatus::Unverified
        }
    }
}