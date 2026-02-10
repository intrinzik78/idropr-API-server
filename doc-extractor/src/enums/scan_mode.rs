use utoipa::ToSchema;

use crate::enums::ExtractorError;

#[derive(Clone,Copy,Debug,ToSchema)]
#[repr(u8)]
pub enum ScanMode {
    Low     = 1,
    High    = 2
}

impl ScanMode {
    pub fn from_u8(num:u8) -> Result<Self,ExtractorError> {
        match num {
            1   => Ok(ScanMode::Low),
            2   => Ok(ScanMode::High),
            _   => Err(ExtractorError::ScanModeOutOfBounds(num))
        }
    }
}