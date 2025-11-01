use crate::enums::Error;

#[derive(Clone,Debug,PartialEq,Eq)]
#[repr(u8)]
pub enum BusinessAccountStatus {
    Registered  = 1,
    Active      = 2,
    Banned      = 3,
    Cancelled   = 4,
    Suspended   = 5,
}

impl BusinessAccountStatus {
    pub fn from_u8(id:u8) -> Result<BusinessAccountStatus,Error> {
        let status = match id {
            1 => Self::Registered,
            2 => Self::Active,
            3 => Self::Banned,
            4 => Self::Cancelled,
            5 => Self::Suspended,
            _ => return Err(Error::AccountStatusOutOfBounds)
        };

        Ok(status)
    }
}


#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn business_account_ranges() {
        let registered = BusinessAccountStatus::Registered;
        let active = BusinessAccountStatus::Active;
        let banned = BusinessAccountStatus::Banned;
        let cancelled = BusinessAccountStatus::Cancelled;
        let suspended = BusinessAccountStatus::Suspended;

        let registered_test = BusinessAccountStatus::from_u8(1).unwrap();
        let active_test = BusinessAccountStatus::from_u8(2).unwrap();
        let banned_test = BusinessAccountStatus::from_u8(3).unwrap();
        let cancelled_test = BusinessAccountStatus::from_u8(4).unwrap();
        let suspended_test = BusinessAccountStatus::from_u8(5).unwrap();
        let out_of_bounds = BusinessAccountStatus::from_u8(6);

        assert_eq!(registered,registered_test);
        assert_eq!(active,active_test);
        assert_eq!(banned,banned_test);
        assert_eq!(cancelled,cancelled_test);
        assert_eq!(suspended,suspended_test);
        assert!(out_of_bounds.is_err());
    }
}