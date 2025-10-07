use base64::prelude::*;

use crate::enums::Uuid;

/// converts a token buffer to base64 for http transport
pub trait ToBase64 {
    fn to_base64_url(&self) -> String;
}

impl ToBase64 for [u8;32] {
    fn to_base64_url(&self) -> String {
        BASE64_URL_SAFE_NO_PAD.encode(self)
    }
}

impl ToBase64 for &[u8;32] {
    fn to_base64_url(&self) -> String {
        BASE64_URL_SAFE_NO_PAD.encode(self)
    }
}

impl ToBase64 for String {
    fn to_base64_url(&self) -> String {
        BASE64_URL_SAFE_NO_PAD.encode(self)
    }
}

impl ToBase64 for Uuid {
    fn to_base64_url(&self) -> String {
        match self {
            Uuid::Crypto(bytes) => BASE64_URL_SAFE_NO_PAD.encode(bytes),
            Uuid::WebSafe(s) => BASE64_URL_SAFE_NO_PAD.encode(s),
            Uuid::WebSafeNums(s) => BASE64_URL_SAFE_NO_PAD.encode(s)
        }
    }
}