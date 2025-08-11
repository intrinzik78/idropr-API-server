use derive_more::derive::From;
use std::{
    fmt::Display,
    string::FromUtf8Error
};

#[derive(Debug,From)]
pub enum Error {
    
    /// derived from actix_rt for async join errors
    #[from]
    ActixJoinError(actix_rt::task::JoinError),
    
    /// derived from `actix_web::Error`
    #[from]
    Actix(actix_web::Error),

    /// derived from `sqlx::Error` for database errors
    #[from]
    Sqlx(sqlx::Error),
    
    /// derived from `bcrypt::BcryptError` for hashing errors
    #[from]
    BcryptError(bcrypt::BcryptError),
    
    /// derived from `aes_gcm::Error` for encryption errors
    #[from]
    EncryptionError(aes_gcm::Error),

    /// derived from `aes_gcm::Error` for encryption errors
    #[from]
    StdError(String),

    /// Utf8 errors are generated during decryption when Vec<u8> is converted to plain text
    #[from]
    FromUtf8Error(FromUtf8Error),

    DatabaseConnection(String),         // failed database connection with the message passed back by the database itself
    DatabaseConnectionTestFailed,       // generated during a test of a new database connection
    PemCertFileReadSizeMismatch,        // generated when the buffer size does not match the size returned from the file read
    ServerCrash(String),                // generated if the HttpServer itself were to crash
    ServerModeOutOfRange,               // generated when the ToServerMode cannot match a database server mode value
    SystemSettingsNotSet,               // generated on startup when attempting to change a system while it's set to None
    SystemSettingsRecordNotReturned,    // a system settings record was not available in the database
    SystemFlagOutOfRange,               // generated when the ToSystemFlag trait cannot match a database system flag value 

    // disabled by default ↴
    DevError(String),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // print only on non-production server modes, otherwise do not print detailed
        match self {
            Error::DatabaseConnection(e) => write!(f, "[database] Error connecting to database with message: {e}"),
            Error::DatabaseConnectionTestFailed => write!(f, "[database] Sqlx returned a valid connection, but a subsequent connection test failed."),
            Error::PemCertFileReadSizeMismatch => write!(f, "[file:io] Failed to read pem-certificate."),
            Error::ServerCrash(server_error) => write!(f,"[http server error] {server_error}"),
            Error::SystemSettingsRecordNotReturned => write!(f, "[database] System settings not found in database."),
            Error::DevError(dev_message) => write!(f,"[dev message] {dev_message}"),
            _ => write!(f, "{self:?}")
        }
    }
}
