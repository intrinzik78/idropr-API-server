pub mod email_verifications_openapi_spec;

mod email_delete;
mod email_get;
mod email_patch;
mod email_post;
mod email_put;
mod open_api_email_tests;

pub use email_post::CreateEmailVerification;
pub use email_patch::PatchEmailVerification;