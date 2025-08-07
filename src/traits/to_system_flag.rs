use crate::enums::{
    Error,
    SystemFlag
};

type Result<T> = std::result::Result<T,Error>;

// permissions are stored in the database as a TINYINT(1) with DEFAULT(0)
// the ToSystemFlag trait makes converting database values to enums states effortless

pub trait ToSystemFlag {
    fn to_system_flag(self) -> Result<SystemFlag>;
}

// sqlx::query_as() can return TINYINT(1) as a bool
impl ToSystemFlag for bool {
    fn to_system_flag(self) -> Result<SystemFlag> {
        let flag_value = match self {
            true  => SystemFlag::Enabled,
            false => SystemFlag::Disabled
        };

        Ok(flag_value)
    }
}

impl ToSystemFlag for i8 {
    fn to_system_flag(self) -> Result<SystemFlag> {
        let flag_value = match self {
            ..0 => return Err(Error::SystemFlagOutOfRange),
            0   => SystemFlag::Disabled,
            1   => SystemFlag::Enabled,
            2.. => return Err(Error::SystemFlagOutOfRange),
        };

        Ok(flag_value)
    }
}

impl ToSystemFlag for u8 {
    fn to_system_flag(self) -> Result<SystemFlag> {
        let flag_value = match self {
            0   => SystemFlag::Disabled,
            1   => SystemFlag::Enabled,
            2.. => return Err(Error::SystemFlagOutOfRange),
        };

        Ok(flag_value)
    }
}