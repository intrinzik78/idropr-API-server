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

    #[from]
    Base64(base64::DecodeError),

    /// derived from `rand::rand_core::OsError`
    #[from]
    OsError(rand::rand_core::OsError),

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

    ApiPasswordOutOfBounds,             // api secret passwords must be: [0 < password < 32]
    ApiSecretsOutOfSyncWithDatabase,
    CannotDecryptEmptyDataSet,          // attempted decryption on an empty data set
    CouldNotVerifyEncryptionSuccess,
    DatabaseConnection(String),         // failed database connection with the message passed back by the database itself
    DatabaseConnectionTestFailed,       // generated during a test of a new database connection
    MalformedAuthorizationToken,        // authorization token did not 
    MasterPasswordNotProvided,          // secrets controller requires master password
    MissingAuthorizationBearerInHeader, // authorization bearer was not present during an authorization check
    PemCertFileReadSizeMismatch,        // generated when the buffer size does not match the size returned from the file read
    PoisonedApiSecretsList,             // api secrets rwlock could not be locked for reading / writing
    PoisonedSessionList,                // session shard could not be locked
    ZeroLengthUUIDFound,                // uuids cannot be zero length, zero length found
    ServerCrash(String),                // generated if the HttpServer itself were to crash
    ServerModeOutOfRange,               // generated when the ToServerMode cannot match a database server mode value
    SessionHashNotVerified,             // could not verify the bcrypt hash with the user's token
    SessionNotFoundDuringRefresh,       // generated when a token was marked stale, but then couldn't be retreived from the session map
    SessionNotFoundInDatabase,          // could not find a linked session in the database during a refresh
    SessionTokenLengthTooLong,          // client has provided a session token longer than required
    SessionTokenLengthTooShort,         // client has provided a session token shorter than required
    SessionTokenIncorrectType,          // UUID::Crypto is the correct type to pass to the session token hasher
    SliceNotCopied,                     // could not verify copy_from_slice was successful
    SystemSettingsNotSet,               // generated on startup when attempting to change a system while it's set to None
    SystemSettingsRecordNotReturned,    // a system settings record was not available in the database
    SystemFlagOutOfRange,               // generated when the ToSystemFlag trait cannot match a database system flag value 
    UserAccountStatusOutOfBounds,       // generated when ToUserAccountStatus cannot parse a value into a UserAccountStatus enum
    UserTypeOutOfBounds,                // generated when a user type id (database) cannot be parsed into a user type
    WrongUuidTypeForSessionHash,        // session hash requires a crypto uuid

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
            Error::PoisonedSessionList => write!(f,"[sessions] Session shard could not be locked."),
            Error::SessionTokenLengthTooLong => write!(f,"[sessions] Client provided session token out of bounds: too long."),
            Error::SessionTokenLengthTooShort => write!(f,"[sessions] Client provided session token out of bounds: too short"),
            Error::ServerCrash(server_error) => write!(f,"[http server error] {server_error}"),
            Error::ZeroLengthUUIDFound => write!(f, "[uuid] Invalid length provided to uuid generator"),
            Error::SystemSettingsRecordNotReturned => write!(f, "[database] System settings not found in database."),
            Error::UserTypeOutOfBounds => write!(f,"[api] invalid user type given"),
            Error::DevError(dev_message) => write!(f,"[dev message] {dev_message}"),
            Error::WrongUuidTypeForSessionHash => write!(f,"[sessions] Bad UUID type given for session hash"),
            _ => write!(f, "{self:?}")
        }
    }
}
