use crate::enums::Error;

#[repr(u8)]
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum LocationPriority {
    None  = 0,
    Low   = 1,
    Mid   = 2,
    High  = 3,
}

impl LocationPriority {
    pub fn from_u8(priority_id:u8) -> Result<Self,Error> {
        let priority = match priority_id {
            0 => Self::None,
            1 => Self::Low,
            2 => Self::Mid,
            3 => Self::High,
            _ => return Err(Error::LocationPriorityOutOfBounds)
        };

        Ok(priority)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn location_priority_range() {
        let none = LocationPriority::None;
        let low = LocationPriority::Low;
        let mid = LocationPriority::Mid;
        let high = LocationPriority::High;

        let none_test = LocationPriority::from_u8(0).unwrap();
        let low_test = LocationPriority::from_u8(1).unwrap();
        let mid_test = LocationPriority::from_u8(2).unwrap();
        let high_test = LocationPriority::from_u8(3).unwrap();
        let fail_test = LocationPriority::from_u8(4);

        assert_eq!(none,none_test);
        assert_eq!(low,low_test);
        assert_eq!(mid,mid_test);
        assert_eq!(high,high_test);
        assert!(fail_test.is_err());
    }
}