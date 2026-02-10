use serde::Serialize;

use crate::enums::Error;

#[repr(u8)]
#[derive(Copy,Clone,Debug,Serialize,PartialEq)]
pub enum CampaignStatus {
    Deleted   = 0,
    Created   = 1,
    Scheduled = 2,
    Active    = 3,
    Paused    = 4,
    Ended     = 5
}

impl CampaignStatus {
    pub fn from_u8(id: u8) -> Result<Self, Error> {
        let status = match id {
            0 => Self::Deleted,
            1 => Self::Created,
            2 => Self::Scheduled,
            3 => Self::Active,
            4 => Self::Paused,
            5 => Self::Ended,
            _ => return Err(Error::CampaignStatusOutOfBounds(id))
        };

        Ok(status)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn bounds_check() {
        type C = CampaignStatus;

        let deleted = C::Deleted;
        let created = C::Created;
        let scheduled = C::Scheduled;
        let active = C::Active;
        let paused = C::Paused;
        let ended = C::Ended;

        let deleted_test = C::from_u8(0).unwrap();
        let created_test = C::from_u8(1).unwrap();
        let scheduled_test = C::from_u8(2).unwrap();
        let active_test = C::from_u8(3).unwrap();
        let paused_test = C::from_u8(4).unwrap();
        let ended_test = C::from_u8(5).unwrap();
        let fail_test = C::from_u8(6);

        assert_eq!(deleted, deleted_test);
        assert_eq!(created, created_test);
        assert_eq!(scheduled, scheduled_test);
        assert_eq!(active, active_test);
        assert_eq!(paused, paused_test);
        assert_eq!(ended, ended_test);

        assert!(fail_test.is_err());
    }
}
