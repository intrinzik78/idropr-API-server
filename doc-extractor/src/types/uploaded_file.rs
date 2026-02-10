use std::path::PathBuf;

use blake3::Hash;
use crate::enums::FileType;

pub struct UploadedFile {
    pub bytes: u64,
    pub client_file_name: String,
    pub file_type: FileType,
    pub hash: Hash,
    pub path: PathBuf,
    pub session_id: i64,
    pub storage_key: [u8;16]
}