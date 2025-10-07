use derive_more::derive::From;
use std::fmt::Display;

use crate::types::PostmarkResponse;

#[repr(u32)]
#[derive(Debug,From)]
pub enum PostmarkError {
    #[from]
    Reqwest(reqwest::Error),

    // define custom errors default ↴
    Api(PostmarkResponse),
    SendingEmailAddressNotPermitted,

    // disabled by default ↴
    // DevError(String),
}

impl std::error::Error for PostmarkError {}

impl Display for PostmarkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        type P = PostmarkError;
        match self {
            P::SendingEmailAddressNotPermitted => write!(f, "email not on the permitted list of senders found in the .env file"),
            _ => write!(f, "{self:?}")
        }
    }
}
