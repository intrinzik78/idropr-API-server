use std::str::FromStr;

use chrono::{DateTime,Utc};
use database::types::DatabaseConnection;
use postmark::{enums::SendStatus, types::PostmarkResponse};

use crate::enums::Error;

type Result<T> = std::result::Result<T,Error>;

pub struct PostmarkLog {
    pub id: u64,
    pub status_id: u8,
    pub postmark_message_id: String,
    pub error_code: u32,
    pub error_message: Option<String>,
    pub submitted_at: DateTime<Utc>
}

impl PostmarkLog {
    pub async fn new(email_id: u64, status: SendStatus, postmark_response: &PostmarkResponse, database: &DatabaseConnection) -> Result<u64> {
        let status = status as u32;
        let code = postmark_response.error_code;
        let date: DateTime<Utc> = DateTime::from_str(&postmark_response.submitted_at)?;
        let sql = "INSERT INTO `email_postmark_log` (email_id,status_id,postmark_message_id,error_code,error_message,submitted_at) VALUES(?,?,?,?,?,?)";
        let insert_id = sqlx::query(sql)
            .bind(email_id)
            .bind(status)
            .bind(&postmark_response.message_id)
            .bind(code)
            .bind(&postmark_response.message)
            .bind(date)
            .execute(&database.pool)
            .await?
            .last_insert_id();

        Ok(insert_id)
    }
}