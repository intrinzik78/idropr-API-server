use chrono::{Utc,DateTime};

use database::types::DatabaseConnection;
use doc_extractor::enums::ScanStatus;
use sqlx::FromRow;

use crate::enums::Error;

type Result<T> = std::result::Result<T,Error>;

pub struct ScanSession {
    id: i64,
    status: ScanStatus,
    created_at: DateTime<Utc>
}

impl ScanSession {
    pub fn id(&self) -> i64 { self.id }
    pub fn status(&self) -> ScanStatus { self.status }
    pub fn created_at(&self) -> &DateTime<Utc> { &self.created_at }
}

/// async
impl ScanSession {
    pub async fn new(connection: &DatabaseConnection) -> Result<ScanSession> {
        let sql = "INSERT INTO `scan_sessions` (status_id) VALUES(?)";
        let status_id = ScanStatus::Created as u8;

        let insert_id:i64 = sqlx::query(sql)
            .bind(status_id)
            .execute(&connection.pool)
            .await?
            .last_insert_id() as i64;

        if let Some(session) = DatabaseHelper::by_id(insert_id, connection).await? {
            Ok(session)
        } else {
            Err(Error::ScanSessionIdNotCreated)
        }
    }
}

#[derive(Debug,FromRow)]
pub struct DatabaseHelper {
    id: i64,
    status_id: u8,
    created_at: DateTime<Utc>
}

/// sync
impl DatabaseHelper {
    pub fn transform(&self) -> Result<ScanSession> {
        let status:ScanStatus = ScanStatus::from_u8(self.status_id)?;

        let session = ScanSession {
            id: self.id,
            status,
            created_at: self.created_at
        };

        Ok(session)
    }
}

/// async
impl DatabaseHelper {
    pub async fn by_id(id:i64, connection:&DatabaseConnection) -> Result<Option<ScanSession>> {
        let sql = "SELECT id,status_id,created_at FROM `scan_sessions` WHERE id = ?";
        let helper_opt:Option<DatabaseHelper> = sqlx::query_as(sql)
            .bind(id)
            .fetch_optional(&connection.pool)
            .await?;

        if let Some(helper) = helper_opt {
            let scan_session = helper.transform()?;

            Ok(Some(scan_session))
        } else {
            Ok(None)
        }
    }
}