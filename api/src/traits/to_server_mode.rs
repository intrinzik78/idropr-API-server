use crate::{
    enums::{
        Error,
        ServerMode
    }
};

type Result<T> = std::result::Result<T,Error>;
pub trait ToServerMode {
    fn to_server_mode(self) -> Result<ServerMode>;
}

impl ToServerMode for u8 {
    fn to_server_mode(self) -> Result<ServerMode> {
        match self {
            0 => Err(Error::ServerModeOutOfRange),
            1 => Ok(ServerMode::Development),
            2 => Ok(ServerMode::Maintenance),
            3 => Ok(ServerMode::Production),
            4.. => Err(Error::ServerModeOutOfRange)
        }
    }
}

impl ToServerMode for i8 {
    fn to_server_mode(self) -> Result<ServerMode> {
        match self {
            ..1 => Err(Error::ServerModeOutOfRange),
            1 => Ok(ServerMode::Development),
            2 => Ok(ServerMode::Maintenance),
            3 => Ok(ServerMode::Production),
            4.. => Err(Error::ServerModeOutOfRange)
        }
    }
}

impl ToServerMode for String {
    fn to_server_mode(self) -> Result<ServerMode> {
        match self.as_str() {
            "DEVELOPMENT" => Ok(ServerMode::Development),
            "PRODUCTION"  => Ok(ServerMode::Maintenance),
            "MAINTENANCE" => Ok(ServerMode::Production),
            _ => Err(Error::ServerModeOutOfRange)
        }
    }
}