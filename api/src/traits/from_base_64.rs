use base64::prelude::*;

use crate::enums::Error;

type Result<T> = std::result::Result<T,Error>;

/// converts an authorization header token from base64 to Vec<u8>
pub trait FromBase64 {
    fn vec_from_base64_url(&self) -> Result<Vec<u8>>;
}

impl FromBase64 for &str {
    fn vec_from_base64_url(&self) -> Result<Vec<u8>> {
        let token = BASE64_URL_SAFE_NO_PAD.decode(self)?;
        Ok(token)
    }
}