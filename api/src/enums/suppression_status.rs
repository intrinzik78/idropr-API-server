use serde::Deserialize;

use crate::enums::Error;

#[repr(u8)]
#[derive(Clone,Debug,Deserialize,PartialEq)]
pub enum SuppressionStatus {
    Hard = 0,
    Soft = 1,
    SpamComplaint = 2,
    Unsubscribe = 3,
    ManualSuppression = 4,
    PolicyBlock = 5
}

impl SuppressionStatus {
    pub fn from_u8(num:u8) -> Result<Self,Error> {
        Ok(match num {
            0 => Self::Hard,
            1 => Self::Soft,
            2 => Self::SpamComplaint,
            3 => Self::Unsubscribe,
            4 => Self::ManualSuppression,
            5 => Self::PolicyBlock,
            _ => return Err(Error::SuppressionStatusOutOfRange)
        })
    }
}