use base64::prelude::*;

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