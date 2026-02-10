use crate::enums::Error;

#[repr(u8)]
#[derive(Clone,Debug,PartialEq)]
pub enum UserType {
    Business    = 1,
    Community   = 2,
    System      = 3
}

impl UserType {
    pub fn from_u8(value: u8) -> Result<Self, Error> {
        Ok(match value {
            1 => Self::Business,
            2 => Self::Community,
            3 => Self::System,
            _ => return Err(Error::UserTypeOutOfBounds)
        })
    }
}