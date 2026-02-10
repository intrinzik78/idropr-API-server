use chrono::{DateTime, Utc, Local};
use sqlx::{FromRow, MySql, Transaction};
use database::types::DatabaseConnection;

use crate::{
    enums::{Error, ExpiredStatus, RowsUpdated, VerificationStatus},
    traits::{ToLocalTime, ToUpdatedResult}, types::email::SuppressedEmail
};

type Result<T> = std::result::Result<T,Error>;

const REFRESH_TIME:i64 = 60;     // 1 minute refresh time
const EXPIRE_TIME:i64 = 60 * 10; // 10 minutes to expire and new verification created

#[derive(Debug)]
pub struct EmailVerification {
    id:i64,
    email:String,
    hash:Vec<u8>,
    verified_at:Option<DateTime<Local>>,
    created_at:DateTime<Local>,
    updated_at:DateTime<Local>
}

/// sync
impl EmailVerification {
    /// getter
    pub fn id(&self) -> i64 {
        self.id
    }

    /// getter
    pub fn email(&self) -> &String {
        &self.email
    }

    /// getter
    pub fn hash(&self) -> &[u8] {
        &self.hash
    }

    /// getter
    pub fn created_at(&self) -> &DateTime<Local> {
        &self.created_at
    }

    /// getter
    pub fn updated_at(&self) -> &DateTime<Local> {
        &self.updated_at
    }

    /// returns whether a valid `verified_at` field entry is present
    pub fn is_verified(&self) -> VerificationStatus {
        match self.verified_at.is_some() {
            true  => VerificationStatus::Verified,
            false => VerificationStatus::Unverified
        }
    }

    /// returns whether the rate limit is exceed for a new verification email to be sent
    pub fn may_refresh(&self) -> bool {
        let now = Local::now();
        let updated_time_delta = now.signed_duration_since(self.updated_at);

        updated_time_delta > chrono::Duration::seconds(REFRESH_TIME)
    }

    /// returns whether a verification email link has expired
    pub fn expired(&self) -> ExpiredStatus {
        let now = Local::now();
        let created_time_delta = now.signed_duration_since(self.created_at);

        match created_time_delta > chrono::Duration::seconds(EXPIRE_TIME) {
            true  => ExpiredStatus::Expired,
            false => ExpiredStatus::NotExpired
        }
    }
}

// async
impl EmailVerification {

    /// fetches an email verification record
    pub async fn by_email(email:&str, database: &DatabaseConnection) -> Result<Option<EmailVerification>> {
        let sql = "SELECT id,email,hash,verified_at,created_at,updated_at FROM `email_verification` WHERE email = ? LIMIT 1";
        let helper_opt: Option<DatabaseHelper> = sqlx::query_as(sql)
            .bind(email)
            .fetch_optional(&database.pool)
            .await?;

        if let Some(helper) = helper_opt {
            Ok(Some(helper.transform()?))
        } else {
            Ok(None)
        }
    }

    /// fetches an email verification record
    pub async fn by_id(id:i64, database: &DatabaseConnection) -> Result<Option<EmailVerification>> {
        let sql = "SELECT id,email,hash,verified_at,created_at,updated_at FROM `email_verification` WHERE id = ? LIMIT 1";
        let helper_opt: Option<DatabaseHelper> = sqlx::query_as(sql)
            .bind(id)
            .fetch_optional(&database.pool)
            .await?;

        if let Some(helper) = helper_opt {
            Ok(Some(helper.transform()?))
        } else {
            Ok(None)
        }
    }

    /// inserts a new email verifcation record
    pub async fn into_db(email:&str, hash:&[u8;32], database: &DatabaseConnection) -> Result<u64> {
        let sql = "INSERT INTO `email_verification` (email,hash) VALUES(?,?)";
        let insert_id = sqlx::query(sql)
            .bind(email)
            .bind(hash.as_slice())
            .execute(&database.pool)
            .await?
            .last_insert_id();

        Ok(insert_id)
    }

    /// inserts a new email verification record within a transaction
    pub async fn into_db_as_transaction(email:&str, hash:&[u8;32], tx: &mut Transaction<'_,MySql>) -> Result<u64> {
        let sql = "INSERT INTO `email_verification` (email,hash) VALUES(?,?)";
        let insert_id = sqlx::query(sql)
            .bind(email)
            .bind(hash.as_slice())
            .execute(&mut **tx)
            .await?
            .last_insert_id();

        Ok(insert_id)
    }

    /// check if email is on the suppression list
    pub async fn is_suppressed(&self, database: &DatabaseConnection) -> Result<bool> {
        let suppression_record = SuppressedEmail::by_email(&self.email, database).await?;
        
        Ok(suppression_record.is_some())
    }

    /// updates the uuid of an existing verifcation record
    pub async fn update_uuid_by_email(email:&str, hash:&[u8;32], database: &DatabaseConnection) -> Result<RowsUpdated> {
        let verified_at: Option<DateTime<Utc>> = None;
        let sql = "UPDATE `email_verification` SET hash = ?, verified_at = ? WHERE email_verification.email = ?";
        let rows_affected = sqlx::query(sql)
            .bind(hash.as_slice())
            .bind(verified_at)
            .bind(email)
            .execute(&database.pool)
            .await?
            .rows_affected()
            .to_updated_result();

        Ok(rows_affected)
    }

    /// updates the uuid of an existing verifcation record
    pub async fn refresh_by_id(id:i64, hash:&[u8;32], database: &DatabaseConnection) -> Result<RowsUpdated> {
        let verified_at: Option<DateTime<Utc>> = None;
        let sql = "UPDATE `email_verification` SET hash = ?, verified_at = ?, created_at = CURRENT_TIMESTAMP() WHERE email_verification.id = ?";
        let rows_affected = sqlx::query(sql)
            .bind(hash.as_slice())
            .bind(verified_at)
            .bind(id)
            .execute(&database.pool)
            .await?
            .rows_affected()
            .to_updated_result();

        Ok(rows_affected)
    }

    /// sets the verified_at field to CURRENT_TIMESTAMP
    pub async fn verify(id:i64, database: &DatabaseConnection) -> Result<RowsUpdated> {
        let sql = "UPDATE `email_verification` SET verified_at = CURRENT_TIMESTAMP() WHERE id = ?";
        let updated = sqlx::query(sql)
            .bind(id)
            .execute(&database.pool)
            .await?
            .rows_affected()
            .to_updated_result();

        Ok(updated)
    }
}


#[derive(Debug,FromRow)]
struct DatabaseHelper {
    pub id: i64,
    pub email: String,
    pub hash: Vec<u8>,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>
}

impl DatabaseHelper {
    fn transform(self) -> Result<EmailVerification> {
        let verified_time = self.verified_at.map(|t| t.to_local_time());

        let record = EmailVerification {
            id: self.id,
            email: self.email,
            hash: self.hash,
            verified_at: verified_time,
            created_at: self.created_at.to_local_time(),
            updated_at: self.updated_at.to_local_time()
        };

        Ok(record)
    }
}