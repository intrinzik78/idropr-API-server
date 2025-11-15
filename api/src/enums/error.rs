use actix_web::http::header::ToStrError;
use derive_more::derive::From;
use postmark;
use std::{
    fmt::Display,
    str::Utf8Error,
    string::FromUtf8Error
};

use crate::types::ApiErrorData;

#[derive(Debug,From)]
pub enum Error {

    // dev error should be disabled by default ↴
    // DevError(String),

    // interal crate defined errors ↴

    /// derived from actix_rt for async join errors
    #[from]
    ActixJoinError(actix_rt::task::JoinError),
    
    /// derived from `actix_web::Error`
    #[from]
    Actix(actix_web::Error),

    /// derived from `base64::DecodeError`
    #[from]
    Base64(base64::DecodeError),

    /// derived from `bcrypt::BcryptError` for hashing errors
    #[from]
    BcryptError(bcrypt::BcryptError),

    /// derived from `chrono`
    #[from]
    Chrono(chrono::ParseError),
    
    /// derived from the database crate
    #[from]
    DatabaseError(database::enums::DatabaseError),

    /// derived from `aes_gcm::Error` for encryption errors
    #[from]
    EmailTemplate(email_template::enums::TemplateError),

    /// derived from `aes_gcm::Error` for encryption errors
    #[from]
    EncryptionError(aes_gcm::Error),

    /// Utf8 errors are generated during decryption when Vec<u8> is converted to plain text
    #[from]
    FromUtf8Error(FromUtf8Error),

    /// Utf8 errors are generated during decryption when Vec<u8> is converted to plain text
    #[from]
    StrFromUtf8Error(Utf8Error),

    /// derived from `rand::rand_core::OsError`
    #[from]
    OsError(rand::rand_core::OsError),

    /// derived from `rand::rand_core::OsError`
    #[from]
    Postmark(postmark::enums::PostmarkError),

    /// derived from `aes_gcm::Error` for encryption errors
    #[from]
    StdError(String),

    /// derived from `sqlx::Error` for database errors
    #[from]
    Sqlx(sqlx::Error),

    /// derived from `aes_gcm::Error` for encryption errors
    #[from]
    ToStrError(ToStrError),

    // interal server errors ↴
    AccountStatusOutOfBounds,
    ApiPasswordOutOfBounds,             // api secret passwords must be: [0 < password < 32]
    ApiSecretsOutOfSyncWithDatabase,
    AddressBuilderMissingData(String),
    BusinessAccountNotFound(i64),
    CannotDecryptEmptyDataSet,          // attempted decryption on an empty data set
    CouldNotVerifyEncryptionSuccess,
    DatabaseConnection(String),         // failed database connection with the message passed back by the database itself
    DatabaseConnectionTestFailed,       // generated during a test of a new database connection
    DatabaseTransactionVerification,
    EmailVerificationExpired,
    EmptyStringWhereDataExpected,
    InsufficientLocationPermissions,
    LocationPriorityOutOfBounds,
    LocationRecordNotFoundById,
    MalformedAuthorizationToken,        // authorization token did not 
    MasterPasswordNotProvided,          // secrets controller requires master password
    MissingAuthorizationBearerInHeader, // authorization bearer was not present during an authorization check
    MissingUserInAuthContext,
    MissingLocationQueryParam(String),  // generated when a query on a Location resource is missing a required get parameter
    NoApiRecordByThatName,
    PemCertFileReadSizeMismatch,        // generated when the buffer size does not match the size returned from the file read
    PoisonedApiSecretsList,             // api secrets rwlock could not be locked for reading / writing
    PoisonedSessionList,                // session shard could not be locked
    PoisonedUserEpoch,
    RequiredUserBuildDataMissing,
    ServerCrash(String),                // generated if the HttpServer itself were to crash
    ServerModeOutOfRange,               // generated when the ToServerMode cannot match a database server mode value
    SessionHashNotVerified,             // could not verify the bcrypt hash with the user's token
    SessionGarbageInstantFailed,        // garbage collector couldn't create a new Instant during sweep startup
    SessionExpired,
    SessionLockNotAquired,
    SessionNotFound,
    SessionNotFoundDuringRefresh,       // generated when a token was marked stale, but then couldn't be retreived from the session map
    SessionNotFoundDuringUpdate,
    SessionNotFoundInDatabase,          // could not find a linked session in the database during a refresh
    SessionTokenLengthTooLong,          // client has provided a session token longer than required
    SessionTokenLengthTooShort,         // client has provided a session token shorter than required
    SessionTokenIncorrectType,          // UUID::Crypto is the correct type to pass to the session token hasher
    SliceNotCopied,                     // could not verify copy_from_slice was successful
    SuppressionStatusOutOfRange,
    SystemSettingsNotSet,               // generated on startup when attempting to change a system while it's set to None
    SystemSettingsRecordNotReturned,    // a system settings record was not available in the database
    SystemFlagOutOfRange,               // generated when the ToSystemFlag trait cannot match a database system flag value
    TooFewRowsUpdated,                  //
    TooManyRowsUpdated,                 // 
    UnexpectedEmptyUserList,
    UserEpochLockNotAquired,
    UserEpochPoisoned,
    UserAccountStatusNotEnabled,
    UserAccountStatusOutOfBounds,       // generated when ToUserAccountStatus cannot parse a value into a UserAccountStatus enum
    UserIdNotInDatabase,
    UserTypeOutOfBounds,                // generated when a user type id (database) cannot be parsed into a user type
    VerificationEmailRejected,          // email was rejected by email sending service
    VerificationHashCheckFailed,
    VerificationEmailNotFound,
    WrongUuidTypeForSessionHash,        // session hash requires a crypto uuid
    WrongUuidTypeForEmailVerification,  // email verification uuid must be either web-safe string or web-safe-nums string
    ZeroLengthUUIDFound,                // uuids cannot be zero length, zero length found
    

    // external facing api errors ↴
    DuplicateSecretNameExists,          // cannot create a new secret, name already in use
    EmailAlreadyVerified,               // email already verified
    EmailIsSuppressed,                  // email is on the suppression list
    RateLimitedEmailVerification,       // generic rate limited status
}

impl Error {
    /// creates a front facing error message for public consumption
    pub fn to_api_error_message(&self) -> Option<ApiErrorData> {
        type E = Error;

        let data = match self {
            E::DuplicateSecretNameExists       => ApiErrorData { code: 1000, reason: String::from("name already in use") },
            E::EmailAlreadyVerified            => ApiErrorData { code: 1001, reason: String::from("email verified, no further action necessary") },
            E::EmailIsSuppressed               => ApiErrorData { code: 1002, reason: String::from("email address is suppressed") },
            E::RateLimitedEmailVerification    => ApiErrorData { code: 1003, reason: String::from("new verification requested too quickly") },
            E::VerificationEmailRejected       => ApiErrorData { code: 1004, reason: String::from("email service rejected request") },
            E::EmailVerificationExpired        => ApiErrorData { code: 1005, reason: String::from("verification link has expired") },
            E::VerificationEmailNotFound       => ApiErrorData { code: 1006, reason: String::from("record does not exist") },
            E::VerificationHashCheckFailed     => ApiErrorData { code: 1007, reason: String::from("verification failed") },
            E::LocationRecordNotFoundById      => ApiErrorData { code: 1008, reason: String::from("record does not exist") },
            E::InsufficientLocationPermissions => ApiErrorData { code: 1009, reason: String::from("insufficient permissions on location resource") },
            E::MissingLocationQueryParam(_)    => ApiErrorData { code: 1010, reason: String::from("missing location query parameter") },
            E::DatabaseTransactionVerification => ApiErrorData { code: 1011, reason: String::from("server error, data was not saved, try again") },
           _ => return None
        };
        
        Some(data)
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        type E = Error;

        // print only on non-production server modes, otherwise do not print detailed
        match self {
            // Error::DevError(dev_message) => write!(f,"[dev message] {dev_message}"),
            E::DatabaseConnection(e) => write!(f, "[database] Error connecting to database with message: {e}"),
            E::DatabaseConnectionTestFailed => write!(f, "[database] Sqlx returned a valid connection, but a subsequent connection test failed."),
            E::PemCertFileReadSizeMismatch => write!(f, "[file:io] Failed to read pem-certificate."),
            E::PoisonedSessionList => write!(f,"[sessions] Session shard could not be locked."),
            E::SessionTokenLengthTooLong => write!(f,"[sessions] Client provided session token out of bounds: too long."),
            E::SessionTokenLengthTooShort => write!(f,"[sessions] Client provided session token out of bounds: too short"),
            E::ServerCrash(server_error) => write!(f,"[http server error] {server_error}"),
            E::ZeroLengthUUIDFound => write!(f, "[uuid] Invalid length provided to uuid generator"),
            E::SystemSettingsRecordNotReturned => write!(f, "[database] System settings not found in database."),
            E::UserTypeOutOfBounds => write!(f,"[api] invalid user type given"),
            E::WrongUuidTypeForSessionHash => write!(f,"[sessions] Bad UUID type given for session hash"),
            _ => write!(f, "{self:?}")
        }
    }
}
