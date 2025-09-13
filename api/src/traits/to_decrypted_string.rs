use aes_gcm::{aead::Aead, Aes256Gcm, Key, KeyInit, Nonce};

use crate::enums::{Error,MasterPassword};

type Result<T> = std::result::Result<T,Error>;

const NONCE_LEN:usize = 12;
const KEY_LEN:usize = 32;

pub trait ToDecryptedString {
    fn decrypt_to_string(&self, master_password: &MasterPassword) -> impl std::future::Future<Output = Result<Option<String>>> + Send;
}

impl ToDecryptedString for Vec<u8> {
    async fn decrypt_to_string(&self, master_password: &MasterPassword) -> Result<Option<String>> {
        
        // short circuit on empty data set
        if self.is_empty() {
            return Ok(None);
        }

        // extract password from wrapper
        let password = match master_password {
            MasterPassword::Some(password) => password,
            MasterPassword::None => return Err(Error::MasterPasswordNotProvided)
        };

        // max token length is 32 characters
        if password.len() > 32 || password.is_empty() {
            return Err(Error::ApiPasswordOutOfBounds);
        }

        // extract key or create empty vec
        let encrypted_data: Vec<u8> = self.clone();

        // build cipher
        let cipher = {
            // create 32 byte slice and copy password into it
            let slice = password.as_bytes();
            let mut master_password: [u8;KEY_LEN] = [0_u8;32];
            master_password[..password.len()].copy_from_slice(slice);

            // test for copy success
            if &master_password[..password.len()] != slice {
                return Err(Error::SliceNotCopied)
            }

            // create cipher key
            let key = Key::<Aes256Gcm>::from_slice(&master_password);
            Aes256Gcm::new(key)
        };

        // try decrypt key in a blocking thread
        let decrypted_buf = {
            actix_rt::task::spawn_blocking(move || {
                let nonce_bytes = &encrypted_data[..NONCE_LEN];
                let nonce = Nonce::from_slice(nonce_bytes);
                let ciphertext = &encrypted_data[NONCE_LEN..];

                cipher.decrypt(nonce, ciphertext)
            })
            .await??
        };

        let decrypted_string = String::from_utf8(decrypted_buf)?;
        
        Ok(Some(decrypted_string))
    }
}