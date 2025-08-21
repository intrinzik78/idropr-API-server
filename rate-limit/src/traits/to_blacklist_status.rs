use crate::enums::ListStatus;
pub trait ToBlackListStatus {
    fn to_blacklist_status(self) -> ListStatus;
}

impl ToBlackListStatus for bool {
    fn to_blacklist_status(self) -> ListStatus {
        match self {
            true => ListStatus::Blacklisted,
            false => ListStatus::None
        }
    }
}