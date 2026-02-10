use chrono::{DateTime,Utc,Local};
use database::types::DatabaseConnection;
use sqlx::prelude::FromRow;

use crate::{
    enums::{Error, RowsAffected, SuppressionStatus},
    traits::{ToAffectedResult, ToLocalTime}
};

type Result<T> = std::result::Result<T,Error>;

#[derive(FromRow)]
struct DatabaseHelper {
    email:String,
    status:u8,
    reason:String,
    notes:Option<String>,
    count:u32,
    first_at: DateTime<Utc>,
    last_at: DateTime<Utc>
}

impl DatabaseHelper {
    fn transform(self) -> Result<SuppressedEmail> {
        Ok(SuppressedEmail { email: self.email,
            status: SuppressionStatus::from_u8(self.status)?,
            reason: self.reason,
            notes: self.notes,
            count: self.count,
            first_at: self.first_at.to_local_time(),
            last_at: self.last_at.to_local_time()
        })
    }
}

pub struct SuppressedEmail {
    email:String,
    status:SuppressionStatus,
    reason:String,
    notes:Option<String>,
    count:u32,
    first_at: DateTime<Local>,
    last_at: DateTime<Local>
}

// sync
impl SuppressedEmail {
    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn status(&self) -> SuppressionStatus {
        self.status.clone()
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub fn notes(&self) -> Option<&String> {
        if self.notes.is_some() {
            self.notes.as_ref()
        } else {
            None
        }
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn first_at(&self) -> &DateTime<Local> {
        &self.first_at
    }

    pub fn last_at(&self) -> &DateTime<Local> {
        &self.last_at
    }

}

// async
impl SuppressedEmail {
    pub async fn by_email(email: &str, database:&DatabaseConnection) -> Result<Option<SuppressedEmail>> {
        let sql = "SELECT email,status,reason,notes,count,first_at,last_at FROM `email_bounce_list` WHERE email = ?";
        let record_opt: Option<DatabaseHelper> = sqlx::query_as(sql)
            .bind(email)
            .fetch_optional(&database.pool)
            .await?;

        if let Some(record) = record_opt {
            Ok(Some(record.transform()?))
        } else {
            Ok(None)
        }
    }

    pub async fn into_db(email: String, status: SuppressionStatus, reason: String, notes: Option<String>, count: u32, database: &DatabaseConnection) -> Result<RowsAffected> {
        let sql = "INSERT INTO `email_bounce_list` (email,status,reason,notes,count) VALUES(?,?,?,?,?)";
        let rows_affected = sqlx::query(sql)
            .bind(&email)
            .bind(status as u8)
            .bind(&reason)
            .bind(notes)
            .bind(count)
            .execute(&database.pool)
            .await?
            .rows_affected()
            .to_affected_result();

        Ok(rows_affected)
    }
}