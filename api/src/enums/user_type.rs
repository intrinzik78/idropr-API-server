use crate::enums::Error;

#[repr(u8)]
#[derive(Clone,Debug,PartialEq)]
pub enum UserType {
    Standard = 1,
    System   = 2
}

impl UserType {
    pub fn from_u8(value: u8) -> Result<Self, Error> {
        Ok(match value {
            1 => Self::Standard,
            2 => Self::System,
            _ => return Err(Error::UserTypeOutOfBounds)
        })
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn bounds_check() {
        type U = UserType;

        let standard = U::Standard;
        let system = U::System;

        let standard_test = U::from_u8(1).unwrap();
        let system_test = U::from_u8(2).unwrap();
        let fail_low = U::from_u8(0);
        let fail_high = U::from_u8(3);

        assert_eq!(standard, standard_test);
        assert_eq!(system, system_test);

        assert!(fail_low.is_err());
        assert!(fail_high.is_err());
    }
}
