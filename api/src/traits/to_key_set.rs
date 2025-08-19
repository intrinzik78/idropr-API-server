use crate::enums::Error;

type Result<T> = std::result::Result<T,Error>;

const TOKEN_LENGTH:usize = 32;
const DIVIDER:usize = 16;
const BUFSIZE:usize = 16;

/// extracts the token segments from an authorization header
pub trait ToKeySet {
    fn to_key(&self) -> Result<[u8;BUFSIZE]>;
    fn to_secret(&self) -> Result<[u8;BUFSIZE]>;
}

impl ToKeySet for Vec<u8> {
    fn to_key(&self) -> Result<[u8;BUFSIZE]> {
        // length check
        let () = match self.len() {
            ..TOKEN_LENGTH => return Err(Error::SessionTokenLengthTooShort),
            TOKEN_LENGTH  => {},
            33.. => return Err(Error::SessionTokenLengthTooLong)
        };

        // extract key slice
        let key = &self[..DIVIDER];
        
        // reserve and copy key slice to buffer
        let mut key_buf= [0_u8;BUFSIZE];
        key_buf.copy_from_slice(key);

        if key[..BUFSIZE] == key_buf {
            Ok(key_buf)
        } else {
            Err(Error::MalformedAuthorizationToken)
        }
    }

    fn to_secret(&self) -> Result<[u8;BUFSIZE]> {
        // length check
        let () = match self.len() {
            ..TOKEN_LENGTH => return Err(Error::SessionTokenLengthTooShort),
            TOKEN_LENGTH  => {},
            33.. => return Err(Error::SessionTokenLengthTooLong)
        };

        // extract secret slice
        let secret = &self[DIVIDER..];

        // reserve and copy secret slice to buffer
        let mut secret_buf = [0_u8;BUFSIZE];
        secret_buf.copy_from_slice(secret);

        Ok(secret_buf)
    }
}