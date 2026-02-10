use chrono::{Utc,DateTime};

use database::types::DatabaseConnection;
use doc_extractor::enums::ScanStatus;
use sqlx::FromRow;

use crate::{
    enums::{Error, RowsUpdated},
    traits::ToUpdatedResult
};

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

    pub async fn by_id(id:i64, connection: &DatabaseConnection) -> Result<Option<ScanSession>> {
        DatabaseHelper::by_id(id, connection).await
    }

    pub async fn exists(id:i64, connection: &DatabaseConnection) -> Result<bool> {
        DatabaseHelper::exists(id, connection).await
    }

    pub async fn by_status(status: ScanStatus, connection: &DatabaseConnection) -> Result<Vec<ScanSession>> {
        DatabaseHelper::by_status(status,connection).await
    }

    pub async fn update_status_by_id(id:i64, status:ScanStatus, connection: &DatabaseConnection) -> Result<RowsUpdated> {
        let status_id = status as u8;
        println!("{status_id}");
        let sql = "UPDATE `scan_sessions` SET status_id = ? WHERE id = ? LIMIT 1";
        let updated = sqlx::query(sql)
            .bind(status_id)
            .bind(id)
            .execute(&connection.pool)
            .await?
            .rows_affected()
            .to_updated_result()
            .require_one(Error::ScanSessionStatusNotUpdated)?;

        Ok(updated)
    }
}

#[derive(Debug,FromRow)]
struct DatabaseHelper {
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
        let sql = "SELECT id,status_id,created_at FROM `scan_sessions` WHERE id = ? LIMIT 1";
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

    pub async fn exists(id:i64, connection:&DatabaseConnection) -> Result<bool> {
        let sql = "SELECT id,status_id,created_at FROM `scan_sessions` WHERE id = ? LIMIT 1";
        let exists:Option<DatabaseHelper> = sqlx::query_as(sql)
            .bind(id)
            .fetch_optional(&connection.pool)
            .await?;

        Ok(exists.is_some())
    }

    pub async fn by_status(status:ScanStatus, connection: &DatabaseConnection) -> Result<Vec<ScanSession>> {
        let status = status as u8;
        let sql = "SELECT id,status_id,created_at FROM `scan_sessions` WHERE status_id = ?";
        let scans:Vec<DatabaseHelper> = sqlx::query_as(sql)
            .bind(status)
            .fetch_all(&connection.pool)
            .await?;

        let mut sessions:Vec<ScanSession> = Vec::with_capacity(scans.len());

        for scan in scans.iter() {
            sessions.push(scan.transform()?);
        }

        Ok(sessions)
    }
}