use crate::enums::VerificationStatus;

pub trait ToVerificationStatus {
    fn to_verification_status(&self) -> VerificationStatus;
}

impl ToVerificationStatus for bool {
    fn to_verification_status(&self) -> VerificationStatus {
        match self {
            true  => VerificationStatus::Verified,
            false => VerificationStatus::Unverified
        }
    }
}