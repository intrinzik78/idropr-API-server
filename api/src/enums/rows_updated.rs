// used by the ToUpdatedResult trait

use crate::enums::Error;

type Result<T> = std::result::Result<T,Error>;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum RowsUpdated {
    None,
    Some(u64)
}

impl RowsUpdated {
    pub fn is_none(&self) -> bool {
        matches!(self, RowsUpdated::None)
    }

    pub fn require_one(self, err: Error) -> Result<Self> {
        match self {
            RowsUpdated::Some(n) if n >= 1 => Ok(Self::Some(1)),
            _ => Err(err),
        }
    }
}