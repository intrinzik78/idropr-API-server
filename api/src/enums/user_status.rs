use crate::enums::Error;

#[derive(Clone,Copy,Debug,PartialEq)]
#[repr(i8)]
pub enum UserAccountStatus {
    Disabled    = 0,  // 0
    Enabled     = 1,  // 1
    Suspended   = 2,  // 2
    Banned      = 3   // 3
}

impl UserAccountStatus {
    pub fn from_u8(value: u8) -> Result<Self, Error> {
        Ok(match value {
            0 => Self::Disabled,
            1 => Self::Enabled,
            2 => Self::Suspended,
            3 => Self::Banned,
            _ => return Err(Error::UserAccountStatusOutOfBounds)
        })
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::traits::ToUserAccountStatus;

    #[test]
    fn bounds_check() {
        type U = UserAccountStatus;

        let disabled = 0;
        let enabled = 1;
        let suspended = 2;
        let banned = 3;
        
        let disabled_test = disabled.to_user_account_status().unwrap();
        let enabled_test = enabled.to_user_account_status().unwrap();
        let suspended_test = suspended.to_user_account_status().unwrap();
        let banned_test = banned.to_user_account_status().unwrap();

        assert_eq!(U::Disabled,disabled_test);
        assert_eq!(U::Enabled,enabled_test);
        assert_eq!(U::Suspended,suspended_test);
        assert_eq!(U::Banned,banned_test);

        let fail = 4;
        let fail_test = fail.to_user_account_status();
        assert!(fail_test.is_err());
    }
}