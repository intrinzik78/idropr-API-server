use derive_more::derive::From;
use std::{fmt::{Debug, Display}, net::IpAddr};

/// customer error types for RateLimter
#[derive(Debug,From)]
pub enum RateLimitError {
    // derived errors ↴
    #[from]
    IpAddr(std::net::AddrParseError),

    // custom error types
    DuplicateBlacklistEntry(IpAddr),
    DuplicateWhitelistEntry(IpAddr),
    PoisonedBlacklist,
    PoisonedRateLimiterMap,
    PoisonedWhitelistlist,

    // disabled by default ↴
    // DevError(String),
}

impl std::error::Error for RateLimitError {}

impl Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // print only on non-production server modes, otherwise do not print detailed
        match self {
            RateLimitError::DuplicateBlacklistEntry(ip) => write!(f, "[rate limit] ip \"{ip:?}\" already blacklisted"),
            RateLimitError::DuplicateWhitelistEntry(ip) => write!(f, "[rate limit] ip \"{ip:?}\" already whitelisted"),
            RateLimitError::PoisonedBlacklist => write!(f, "[rate limit] rate limiter black list poisoned"),
            RateLimitError::PoisonedRateLimiterMap => write!(f, "[rate limit] rate limiter map poisoned"),
            RateLimitError::PoisonedWhitelistlist => write!(f, "[rate limit] rate limiter white list poisoned"),
            // RateLimitError::DevError(dev_message) => write!(f,"[dev message] {dev_message}"),
            _ => write!(f, "[rate limit error]")
        }
    }
}
