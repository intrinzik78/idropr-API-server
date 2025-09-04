use crate::enums::RowsAffected;

// Database update calls should normally result in an affected_row value > 0, however
// this is not always an error. The UpdateResult and ToUpdateResult trait allow the caller
// to make the decision about whether a NoRowsUpdated result is actually an error and preserves
// the Err state of a database query for real / technical connection errors
pub trait ToAffectedResult {
    fn to_affected_result(self) -> RowsAffected;
}

impl ToAffectedResult for i64 {
    fn to_affected_result(self) -> RowsAffected {
        match self {
            0 => RowsAffected::None,
            _ => RowsAffected::Some(self as u64)
        }
    }
}

impl ToAffectedResult for u64 {
    fn to_affected_result(self) -> RowsAffected {
        match self {
            0 => RowsAffected::None,
            _ => RowsAffected::Some(self)
        }
    }
}