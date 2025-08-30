mod secrets_delete;
mod secrets_get;
mod secrets_patch;
mod secrets_post;
mod secrets_put;

pub use secrets_delete::SecretsDelete;
pub use secrets_get::SecretsGet;
pub use secrets_patch::SecretsPatch;
pub use secrets_post::SecretsPost;
pub use secrets_put::SecretsPut;