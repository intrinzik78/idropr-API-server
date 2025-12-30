use crate::enums::ExtractorError;
use serde::Serialize;

type Result<T> = std::result::Result<T,ExtractorError>;

#[derive(Copy,Clone,Debug,Serialize)]
#[repr(u8)]
pub enum ScanStatus {
    Created         = 1,
    Queued          = 2,
    Running         = 3,
    Completed       = 4,
    Failed          = 5,
    NeedsReview     = 6 
}

impl ScanStatus {
    pub fn from_u8(num:u8) -> Result<Self> {
        let status = match num {
            1 => Self::Created,
            2 => Self::Queued,
            3 => Self::Running,
            4 => Self::Completed,
            5 => Self::Failed,
            6 => Self::NeedsReview,
            _ => return Err(ExtractorError::ScanStatusOutOfBounds(num))
        };

        Ok(status)
    }
}