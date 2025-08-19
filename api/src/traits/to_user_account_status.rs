use crate::enums::{Error,UserAccountStatus};

type Result<T> = std::result::Result<T,Error>;
pub trait ToUserAccountStatus {
    fn to_user_account_status(self) -> Result<UserAccountStatus>;
}

impl ToUserAccountStatus for i8 {
 fn to_user_account_status(self) -> Result<UserAccountStatus> {
    let status = match self {
        ..0 => return Err(Error::UserAccountStatusOutOfBounds),
        0 => UserAccountStatus::Disabled,
        1 => UserAccountStatus::Enabled,
        2 => UserAccountStatus::Suspended,
        3 => UserAccountStatus::Banned,
        4.. => return Err(Error::UserAccountStatusOutOfBounds)
    };

    Ok(status)
 }
}