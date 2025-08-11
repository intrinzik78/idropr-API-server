use crate::enums::ListStatus;

pub trait ToWhiteListStatus {
    fn to_whitelist_status(self) -> ListStatus;
}

impl ToWhiteListStatus for bool {
    fn to_whitelist_status(self) -> ListStatus {
        match self {
            true => ListStatus::Whitelisted,
            false => ListStatus::None
        }
    }
}