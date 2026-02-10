use std::fmt::{Display,Result};

#[derive(Debug)]
pub enum ExtractorError {
    ScanStatusOutOfBounds(u8),
    ScanModeOutOfBounds(u8),
    MimeTypeInvalid
}

impl std::error::Error for ExtractorError {}

impl ExtractorError {
    /// creates a front facing error message for public consumption
    pub fn to_api_error_reason(&self) -> Option<String> {
        type E = ExtractorError;

        let reason = match self {
            E::MimeTypeInvalid          => Some("inavlid mime/file type".to_string()),
            E::ScanStatusOutOfBounds(_) => Some("invalid scan status encountered".to_string()),
            E::ScanModeOutOfBounds(_)   => Some("invalid scan mode encountered".to_string())
        };
        
        reason
    }
}

impl Display for ExtractorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        type E = ExtractorError;
        
        match self {
            E::MimeTypeInvalid                   => write!(f,"invalid mime type, cannot parse into FileType"),
            E::ScanModeOutOfBounds(id)      => write!(f,"invalid scan mode status id provided. Status ({id}) cannot convert to ScanMode enum"),
            E::ScanStatusOutOfBounds(id)    => write!(f,"invalid scan status id provided. Status ({id}) cannot convert to ScanStatus enum")
        }   
    }
}