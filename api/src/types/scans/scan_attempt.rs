use chrono::{DateTime,Utc};
use doc_extractor::enums::{ScanMode,ScanStatus};
use sqlx::FromRow;

use crate::enums::Error;

type Result<T> = std::result::Result<T,Error>;

pub struct ScanAttempt {
    doc_id: i64,
    batch_row_id: Option<i64>,
    status: ScanStatus,
    mode: ScanMode,
    custom_id: String,
    created_at: DateTime<Utc>,
    error_message: String
}
/// sync
impl ScanAttempt {
    pub fn doc_id(&self) -> i64 { self.doc_id }

    pub fn batch_row_id(&self) -> Option<i64> {
        if let Some(v) = self.batch_row_id {
            Some(v)
        } else {
            None
        }
    }

    pub fn status(&self) -> ScanStatus { self.status }

    pub fn mode(&self) -> ScanMode { self.mode }

    pub fn custom_id(&self) -> &str {&self.custom_id}

    pub fn created_at(&self) -> &DateTime<Utc> { &self.created_at }

    pub fn error_message(&self) -> &str { &self.error_message }

}

/// async
impl ScanAttempt {
    
}

#[derive(FromRow)]
struct DatabaseHelper {
    pub doc_id: i64,
    pub batch_row_id: Option<i64>,
    pub status_id: u8,
    pub mode_id: u8,
    pub custom_id: String,
    pub created_at: DateTime<Utc>,
    pub error_message: String
}

impl DatabaseHelper {
    pub fn _transform(self) -> Result<ScanAttempt> {
        let result = ScanAttempt {
            doc_id: self.doc_id,
            batch_row_id: self.batch_row_id,
            status: ScanStatus::from_u8(self.status_id)?,
            mode: ScanMode::from_u8(self.mode_id)?,
            custom_id: self.custom_id,
            created_at: self.created_at,
            error_message: self.error_message
        };

        Ok(result)
    }
}

