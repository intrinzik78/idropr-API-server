use crate::enums::RowsUpdated;

// Database update calls should normally result in an affected_row value > 0, however
// this is not always an error. The UpdateResult and ToUpdateResult trait allow the caller
// to make the decision about whether a NoRowsUpdated result is actually an error and preserves
// the Err state of a database query for real / technical connection errors
pub trait ToUpdatedResult {
    fn to_updated_result(self) -> RowsUpdated;
}

impl ToUpdatedResult for i64 {
    fn to_updated_result(self) -> RowsUpdated {
        match self {
            0 => RowsUpdated::NoRowsUpdated,
            _ => RowsUpdated::RowsUpdated(self as u64)
        }
    }
}

impl ToUpdatedResult for u64 {
    fn to_updated_result(self) -> RowsUpdated {
        match self {
            0 => RowsUpdated::NoRowsUpdated,
            _ => RowsUpdated::RowsUpdated(self)
        }
    }
}