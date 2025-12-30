use actix_multipart::Multipart;
use actix_web::{web::{Data,Path},HttpResponse};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    enums::{Error},
    types::{ApiResponse, AppState, permissions::WereChecked, scans::ScanSession}
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
    session_id:String
}

pub struct UploadedFile {
    pub client_file_name: Option<String>,
    pub bytes: Vec<u8>
}

pub struct ScanSessionPost;

impl ScanSessionPost {

    async fn batch_upload_logic(payload: &mut Multipart) -> Result<Vec<UploadedFile>> {
        let mut files: Vec<UploadedFile> = Vec::new();
        let mut total_bytes: u64 = 0;

        while let Some(mut field) = payload.try_next().await.map_err(Error::Multipart)? {
            // extract the field name
            let field_name = field.name().ok_or(Error::UploadMissingFieldName)?;

            match field_name {
                "file" => {
                    if files.len() >= MAX_PARTS_PER_REQUEST {
                        return Err(Error::UploadTooManyParts {
                            max: MAX_PARTS_PER_REQUEST,
                        });
                    }

                    // extract the client file name for errors
                    let client_file_name = field
                        .content_disposition()
                        .and_then(|d| d.get_filename().map(|s| s.to_string()));

                    let mut bytes: Vec<u8> = Vec::new();

                    while let Some(chunk) = field.try_next().await.map_err(Error::Multipart)? {
                        bytes.extend_from_slice(&chunk);

                        if bytes.len() as u64 > MAX_BYTES_PER_FILE {
                            return Err(Error::UploadedFileTooLarge);
                        }

                        total_bytes += chunk.len() as u64;
                        if total_bytes > MAX_TOTAL_BYTES {
                            return Err(Error::UploadTooLargeTotal);
                        }
                    }

                    if bytes.is_empty() {
                        return Err(Error::UploadMissingFileData);
                    }

                    files.push(UploadedFile {
                        client_file_name,
                        bytes,
                    });
                }

                _ => {
                        // drain and discard unexpected fields
                        // this will error silently and is bad design, but we can enforce once the SDK shape is set
                        while let Some(_chunk) = field.try_next().await.map_err(Error::Multipart)? {}
                    }
            }
        }

        if files.is_empty() {
            return Err(Error::UploadMissingFileData);
        }

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

    pub async fn private_sessions_response(_permissions: WereChecked, shared: Data<AppState>) -> HttpResponse {
        match Self::new_session_logic(&shared).await {
            Ok(d) => ApiResponse::default().with_code(201).with_message("resource created".to_string()).with_data(d).ok(),
            Err(e) => {
                type E = Error;

                let error_opt = e.to_api_error_message();

                if let Some(d) = error_opt {
                    match e {
                        E::ScanSessionIdNotCreated  => ApiResponse::default().with_code(500).with_data(d).error(),
                        _                           => ApiResponse::server_error().error()
                    }
                } else {
                    ApiResponse::server_error().error()
                }
            }
        }
    }

    pub async fn private_single_upload_response(_permissions: WereChecked, path: Path<ReqPath>, mut payload:Multipart, shared: Data<AppState>) -> HttpResponse {
        let _database = shared.database();
        let _session_id = &path.session_id;

        let _result = ScanSessionPost::batch_upload_logic(&mut payload).await;

        ApiResponse::success()
    }

}
