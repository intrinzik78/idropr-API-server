use std::{ffi::OsStr, path::{Path,PathBuf}};

use actix_multipart::Multipart;
use actix_web::{web::{Data,Path as ActixPath},HttpResponse};
use blake3::Hasher;
use database::types::DatabaseConnection;
use doc_extractor::{
    enums::{FileType, ScanStatus},
    types::UploadedFile
};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};

use tokio::{fs, fs::File, io::AsyncWriteExt};
use utoipa::ToSchema;

use crate::{
    enums::{ApiResult, Error, Uuid}, traits::ToBase64,
    types::{AppState, permissions::WereChecked, scans::{Extraction,ScanController,ScanSession}}
};

type Result<T> = std::result::Result<T,Error>;

const MAX_TOTAL_BYTES:u64 = 250_000_000;
const MAX_BYTES_PER_FILE:u64 = 25_000_000;
const MAX_PARTS_PER_REQUEST:usize = 49;

#[derive(Serialize,ToSchema)]
pub struct NewScanSession {
    session_id: i64
}

#[derive(Deserialize,ToSchema)]
pub struct ReqPath {
    session_id: i64
}

#[derive(Serialize, ToSchema)]
pub struct BatchIngestItem {
    pub client_file_name: String,
    pub id: i64,
    pub deduped: bool,
    pub bytes: u64,
    pub mime_type: String
}

#[derive(Serialize, ToSchema)]
pub struct BatchIngestResponse {
    pub session_id: i64,
    pub uploaded: Vec<BatchIngestItem>
}

pub struct ScanSessionPost;

impl ScanSessionPost {

    fn get_extension_from_filename(filename: &str) -> Result<String> {
        let ext = Path::new(&filename.to_ascii_lowercase())
            .extension()
            .and_then(OsStr::to_str)
            .ok_or(Error::UploadMissingFileExt)?
            .to_string();
        
        Ok(ext)
    }

    async fn batch_upload_logic(session_id:i64, database:&DatabaseConnection, scanner: &ScanController, payload: &mut Multipart) -> Result<Vec<BatchIngestItem>> {
        
        // validate scan session exists
        if !ScanSession::exists(session_id, database).await? {
            return Err(Error::ScanSessionNotFound);
        }

        let mut files: Vec<BatchIngestItem> = Vec::new();

        // track total upload size + total parts
        let mut total_bytes:u64 = 0;
        let mut total_files:usize = 0;

        while let Some(mut field) = payload.try_next().await.map_err(Error::Multipart)? {
            // extract the field name
            let field_name = field.name().ok_or(Error::UploadMissingFieldName)?;
            
            match field_name {
                "file" => {
                    // track + return early on too many parts
                    total_files += 1;

                    if total_files > MAX_PARTS_PER_REQUEST {
                        return Err(Error::UploadTooManyParts {
                            max: MAX_PARTS_PER_REQUEST,
                        });
                    }

                    // extract the client provided file name for error tracking
                    let client_file_name = {
                        field.content_disposition()
                            .and_then(|d| d.get_filename().map(|s| s.to_string()))
                            .ok_or(Error::UploadMissingFileName)?
                    };

                    // extract file extension
                    let ext_str = Self::get_extension_from_filename(&client_file_name)?;
                    let file_type = FileType::from_ext(&ext_str)?;

                    // build path and storage key
                    let storage_key = match Uuid::crypto16()? {
                        Uuid::Crypto16(b) => b,
                        _ => return Err(Error::WrongUuidTypeForImageStorage)
                    };
                    let file_name = storage_key.to_base64_url();
                    let path_str = format!("/var/data/temp/{file_name}.part");
                    let path = PathBuf::from(path_str);

                    // create temporary file
                    let mut file_handle = File::create_new(&path).await?;

                    // track individual file size
                    let mut bytes:u64 = 0;

                    // build hasher
                    let mut hasher = Hasher::new();

                    // stream chunks to file
                    while let Some(chunk) = field.try_next().await.map_err(Error::Multipart)? {
                        // stream to disk
                        file_handle.write_all(&chunk).await?;

                        // error on file size too large
                        bytes += chunk.len() as u64;
                        
                        if bytes > MAX_BYTES_PER_FILE {
                            fs::remove_file(&path).await?;
                            return Err(Error::UploadedFileTooLarge);
                        }

                        // track total upload size + error on total size too large
                        total_bytes += chunk.len() as u64;

                        if total_bytes > MAX_TOTAL_BYTES {
                            fs::remove_file(&path).await?;
                            return Err(Error::UploadTooLargeTotal);
                        }

                        hasher.update(&chunk);
                    }

                    // finalize the write
                    file_handle.flush().await?;
                    drop(file_handle);

                    // verify file has some data
                    if bytes == 0 {
                        fs::remove_file(&path).await?;
                        return Err(Error::UploadMissingFileData);
                    }

                    // finalize the hash
                    let hash = hasher.finalize();

                    // build final path + upload data
                    let ext = file_type.to_ext();
                    let final_path = PathBuf::from(format!("/var/data/{file_name}.{ext}"));

                    let upload_data = UploadedFile {
                        bytes,
                        client_file_name: client_file_name.clone(),
                        file_type,
                        hash, 
                        path: final_path,
                        session_id,
                        storage_key
                    };

                    // remove temp file or move to finished directory
                    let id = match Extraction::into_db(&upload_data, database).await {
                        Ok(insert_id) => {
                            fs::rename(&path, &upload_data.path).await?;
                            insert_id
                        },
                        Err(e) => {
                            fs::remove_file(&upload_data.path).await?;
                            return Err(e);
                        }
                    };

                    // upload data
                    let ingested_doc = BatchIngestItem {
                        client_file_name,
                        id,
                        deduped: false,
                        bytes,
                        mime_type: file_type.to_mime().to_owned(),
                    };

                    files.push(ingested_doc);
                },
                _ => {
                        // drain + discard unexpected fields
                        // this will error silently and is bad design, but we can enforce once the SDK shape is set
                        while let Some(_chunk) = field.try_next().await.map_err(Error::Multipart)? {}
                    }
            }
        }

        // early return on empty file set
        if files.is_empty() {
            return Err(Error::UploadMissingFileData);
        }

        // try wake on the scanner
        if let Err(e) = scanner.wake() {
            println!("{e}");
        }

        // return upload data to caller
        Ok(files)
    }

    async fn new_session_logic(shared:&Data<AppState>) -> Result<NewScanSession> {
        let connection = shared.database();

        let scan_session = ScanSession::new(connection).await?;
        let return_data = NewScanSession {
            session_id: scan_session.id()
        };

        Ok(return_data)
    }

    async fn process_logic(session_id:i64, shared:&Data<AppState>) -> Result<()> {
        // extract connection
        let connection = shared.database();
        
        // validate session
        if !ScanSession::exists(session_id, connection).await? {
            return Err(Error::ScanSessionNotFound);
        }

        // update to queued
        let new_status = ScanStatus::Queued;
        ScanSession::update_status_by_id(session_id,new_status,connection).await?;

        shared.scanner().wake()?;

        Ok(())
    }

    pub async fn private_mulitpart_upload_response(_permissions: WereChecked, path: ActixPath<ReqPath>, mut payload:Multipart, shared: Data<AppState>) -> HttpResponse {
        let database = shared.database();
        let scanner = shared.scanner();
        let session_id = path.into_inner().session_id;

        let result = ScanSessionPost::batch_upload_logic(session_id, database, scanner, &mut payload).await;

        match result {
            Ok(d) => {
                let response = BatchIngestResponse {
                    session_id,
                    uploaded: d,
                };

                match ApiResult::ok(201, "resource created").with_data(response) {
                    Ok(s) => s.to_http(),
                    Err(_e) => ApiResult::server_error().to_http()
                }
            },
            Err(e) => {
                type E = Error;

                let error_opt = e.to_api_error_message();

                if let Some(d) = error_opt {
                    match e {
                        E::UploadMissingFieldName               => ApiResult::bad_request().with_reason(d).to_http(),
                        E::UploadMissingFileData                => ApiResult::bad_request().with_reason(d).to_http(),
                        E::UploadMissingFileName                => ApiResult::bad_request().with_reason(d).to_http(),
                        E::UploadTooLargeTotal                  => ApiResult::too_large().with_reason(d).to_http(),
                        E::UploadedFileTooLarge                 => ApiResult::too_large().with_reason(d).to_http(),
                        E::UploadMissingFileExt                 => ApiResult::bad_request().with_reason(d).to_http(),
                        E::UploadTooManyParts{ max: _ }         => ApiResult::bad_request().with_reason(d).to_http(),
                        E::ScanSessionNotFound                  => ApiResult::bad_request().with_reason(d).to_http(),
                        _                                       => ApiResult::server_error().to_http()
                    } 
                } else {
                    ApiResult::server_error().to_http()
                }
            }
        }
    }

    pub async fn private_process_response(_permissions: WereChecked, path: ActixPath<ReqPath>, shared: Data<AppState>) -> HttpResponse {
        let session_id = path.into_inner().session_id;

        match Self::process_logic(session_id,&shared).await {
            Ok(_) => ApiResult::accepted().to_http(),
            Err(e) => {
                type E = Error;

                let error_opt = e.to_api_error_message();

                if let Some(d) = error_opt {
                    match e {
                        E::ScanSessionNotFound  => ApiResult::not_found().with_reason(d).to_http(),
                        _                       => ApiResult::server_error().to_http()
                    }
                } else {
                    ApiResult::<()>::server_error().to_http()
                }
            }
        }
    }

    pub async fn private_sessions_response(_permissions: WereChecked, shared: Data<AppState>) -> HttpResponse {
        match Self::new_session_logic(&shared).await {
            Ok(d) => {
                ApiResult::resource_created_with_data(d).to_http()
            },
            Err(e) => {
                type E = Error;

                let error_opt = e.to_api_error_message();

                if let Some(d) = error_opt {
                    match e {
                        E::ScanSessionIdNotCreated  => ApiResult::server_error().with_reason(d).to_http(),
                        _                           => ApiResult::server_error().to_http()
                    }
                } else {
                    ApiResult::server_error().to_http()
                }
            }
        }
    }

}
