use crate::enums::{Decision, RateLimitError};

type Result<T> = std::result::Result<T,RateLimitError>;

pub trait ToDecision {
    fn to_decision(self) -> Result<Decision>;
}

impl ToDecision for i32 {
    fn to_decision(self) -> Result<Decision> {
        if self > 0 {
            Ok(Decision::Approved)
        } else {
            Ok(Decision::Denied)
        }
    }
}