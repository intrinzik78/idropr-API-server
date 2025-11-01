use crate::enums::Error;

#[repr(u8)]
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum LocationPriority {
    Reduced     = 0,
    Standard    = 1,
    Mid         = 2,
    High        = 3,
}

impl LocationPriority {
    pub fn from_u8(priority_id:u8) -> Result<Self,Error> {
        let priority = match priority_id {
            0 => Self::Reduced,
            1 => Self::Standard,
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
        type L = LocationPriority;

        let reduced     = L::Reduced;
        let standard    = L::Standard;
        let mid         = L::Mid;
        let high        = L::High;

        let reduced_test =  L::from_u8(0).unwrap();
        let standard_test = L::from_u8(1).unwrap();
        let mid_test =      L::from_u8(2).unwrap();
        let high_test =     L::from_u8(3).unwrap();

        // positive assertions
        assert_eq!(reduced,reduced_test);
        assert_eq!(standard,standard_test);
        assert_eq!(mid,mid_test);
        assert_eq!(high,high_test);

        // negative assertions
        let fail_test = L::from_u8(4);
        assert!(fail_test.is_err());
    }
}