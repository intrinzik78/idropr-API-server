use blake3::Hash;
use chrono::{DateTime,Utc};
use database::types::DatabaseConnection;
use doc_extractor::{
    enums::{FileType,ScanStatus},
    types::UploadedFile
};
use std::path::PathBuf;
use sqlx::prelude::FromRow;

use crate::{
    enums::{Error, RowsUpdated}, traits::{ToBase64, ToUpdatedResult}
};

type Result<T> = std::result::Result<T,Error>;

#[derive(Debug)]
pub struct Extraction {
    id: i64,
    session_id: i64,
    storage_key: String,
    file_hash: Hash,
    status: ScanStatus,
    final_attempt_id: Option<i64>,
    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
    file_type: FileType,
    byte_size: u64
}

// sync
impl Extraction {
    pub fn id(&self) -> i64 { self.id }

    pub fn session_id(&self) -> i64 { self.session_id }

    pub fn storage_key(&self) -> &str { &self.storage_key }

    pub fn file_hash(&self) -> &Hash { &self.file_hash }

    pub fn status(&self) -> ScanStatus { self.status }

    pub fn final_attempt_id(&self) -> Option<i64> { self.final_attempt_id }

    pub fn created_at(&self) -> &DateTime<Utc> { &self.created_at }

    pub fn expires_at(&self) -> Option<&DateTime<Utc>> { self.expires_at.as_ref() }

    pub fn deleted_at(&self) -> Option<&DateTime<Utc>> { self.deleted_at.as_ref() }

    pub fn file_type(&self) -> FileType { self.file_type }

    pub fn byte_size(&self) -> u64 { self.byte_size }

    pub fn blob_path(&self, blobs_dir: &str) -> Result<PathBuf> {
        let ext = self.file_type.to_ext();
        let name = self.storage_key.to_base64_url();
        Ok(PathBuf::from(format!("{blobs_dir}/{name}.{ext}")))
    }
}

// async
impl Extraction {

    pub async fn by_id(id:i64, connection:&DatabaseConnection) -> Result<Option<Extraction>> {
        DatabaseHelper::by_id(id, connection).await
    }

    pub async fn into_db(upload:&UploadedFile, connection:&DatabaseConnection) -> Result<i64> {
        // set status to 'created'
        let status_id = ScanStatus::Created as u8;
        
        // extract mime type
        let mime_type = upload.file_type.to_mime();

        // extract storage_key + validate bin(16)
        let storage_key = upload.storage_key.as_slice();
        
        // extract hash + validate bin(32)
        let file_hash: &[u8] = upload.hash.as_bytes().as_slice();

        if file_hash.len() != 32 {
            return Err(Error::FileHashBadLength(file_hash.len()))
        }

        let sql = "INSERT INTO `scanned_doc` (session_id,storage_key,file_hash,status_id,mime_type,byte_size) VALUES(?,?,?,?,?,?)";
        let insert_id: i64 = sqlx::query(sql)
            .bind(upload.session_id)
            .bind(storage_key)
            .bind(file_hash)
            .bind(status_id)
            .bind(mime_type)
            .bind(upload.bytes)
            .execute(&connection.pool)
            .await?
            .last_insert_id() as i64;

        Ok(insert_id)
    }

    pub async fn by_session_id(session_id:i64, connection:&DatabaseConnection) -> Result<Vec<Extraction>> {
        DatabaseHelper::by_session_id(session_id, connection).await
    }

    pub async fn by_session_id_with_status(session_id:i64, status: ScanStatus, connection:&DatabaseConnection) -> Result<Vec<Extraction>> {
        DatabaseHelper::by_session_id_with_status(session_id, status, connection).await
    }

    pub async fn set_status(status: ScanStatus, id:i64, connection:&DatabaseConnection) -> Result<RowsUpdated> {
        let status_id = status as u8;
        let sql = "UPDATE `scanned_doc` SET status_id = ? WHERE id = ?";
        let updated_rows = sqlx::query(sql)
            .bind(status_id)
            .bind(id)
            .execute(&connection.pool)
            .await?
            .rows_affected()
            .to_updated_result();

        Ok(updated_rows)
    }
}

#[derive(Debug,FromRow)]
struct DatabaseHelper {
    id: i64,
    session_id: i64,
    storage_key: Vec<u8>,
    file_hash: Vec<u8>,
    status_id: u8,
    final_attempt_id: Option<i64>,
    created_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
    mime_type: String,
    byte_size: u64
}

impl DatabaseHelper {
    fn transform(self) -> Result<Extraction> {
        // parse database record
        let file_hash:Hash = {
            // verify hash has enough bytes to populate 
            if self.file_hash.len() != 32 {
                return Err(Error::FileHashBadLength(self.file_hash.len()));
            }
            let mut bytes:[u8;32] = [0;32];
            bytes.copy_from_slice(&self.file_hash);

            Hash::from_bytes(bytes)
        };
        let file_type = FileType::from_mime(&self.mime_type)?;
        let status = ScanStatus::from_u8(self.status_id)?;
        let storage_key = {
            if self.storage_key.len() != 16 {
                return Err(Error::StorageKeyBadLength(self.storage_key.len()));
            }
            String::from_utf8(self.storage_key)?
        };
        
        // build + return Extraction
        let extraction = Extraction {
            id: self.id,
            session_id: self.session_id,
            storage_key,
            file_hash,
            status,
            final_attempt_id: self.final_attempt_id,
            created_at: self.created_at,
            expires_at: self.expires_at,
            deleted_at: self.deleted_at,
            file_type,
            byte_size: self.byte_size
        };

        Ok(extraction)
    }

    pub async fn by_id(id:i64, connection:&DatabaseConnection) -> Result<Option<Extraction>> {
        let sql = "SELECT id,session_id,storage_key,file_hash,status_id,final_attempt_id,created_at,expires_at,deleted_at,mime_type,byte_size FROM `scanned_doc` WHERE id = ? LIMIT 1";
        let helper_opt:Option<DatabaseHelper> = sqlx::query_as(sql)
            .bind(id)
            .fetch_optional(&connection.pool)
            .await?;

        if let Some(helper) = helper_opt {
            let extraction = Some(helper.transform()?);
            Ok(extraction)
        } else {
            Ok(None)
        }
    }

    pub async fn by_session_id(id:i64, connection:&DatabaseConnection) -> Result<Vec<Extraction>> {
        let sql = "SELECT id,session_id,storage_key,file_hash,status_id,final_attempt_id,created_at,expires_at,deleted_at,mime_type,byte_size FROM `scanned_doc` WHERE session_id = ? LIMIT 1000";
        let raw_list:Vec<DatabaseHelper> = sqlx::query_as(sql)
            .bind(id)
            .fetch_all(&connection.pool)
            .await?;

        let list: Vec<Extraction> = raw_list
            .into_iter()
            .map(|d| d.transform())
            .collect::<Result<_>>()?;

        Ok(list)
    }

    pub async fn by_session_id_with_status(id:i64, status:ScanStatus, connection:&DatabaseConnection) -> Result<Vec<Extraction>> {
        let sql = "SELECT id,session_id,storage_key,file_hash,status_id,final_attempt_id,created_at,expires_at,deleted_at,mime_type,byte_size FROM `scanned_doc` WHERE session_id = ? AND status_id = ? LIMIT 1500";
        let raw_list:Vec<DatabaseHelper> = sqlx::query_as(sql)
            .bind(id)
            .bind(status as u8)
            .fetch_all(&connection.pool)
            .await?;

        let list: Vec<Extraction> = raw_list
            .into_iter()
            .map(|d| d.transform())
            .collect::<Result<_>>()?;

        Ok(list)
    }
}

