use derive_more::derive::From;
use std::fmt::{Display,Result};

#[derive(Debug,From)]
pub enum ExtractorError {
    ScanStatusOutOfBounds(u8),
    Dev(String)
}

impl std::error::Error for ExtractorError {}

impl Display for ExtractorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        type E = ExtractorError;
        
        match self {
            E::ScanStatusOutOfBounds(id) => write!(f,"invalid scan status id provided. Status ({id}) cannot convert to ScanStatus enum"),
            E::Dev(s) => write!(f, "Dev error: {s}")
        }   
    }
}