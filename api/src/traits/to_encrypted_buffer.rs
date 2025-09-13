use aes_gcm::{aead::{Aead, OsRng}, AeadCore, Aes256Gcm, Key, KeyInit};

use crate::enums::{Error,MasterPassword};

type Result<T> = std::result::Result<T,Error>;

const NONCE_LEN:usize = 12;

pub trait ToEncryptedBuffer {
    fn encrypt(&self, password: &MasterPassword) -> impl std::future::Future<Output = Result<Vec<u8>>> + Send;
}

impl ToEncryptedBuffer for &str {
    async fn encrypt(&self, master_password: &MasterPassword) -> Result<Vec<u8>> {
       // extract password from wrapper
        let password = match master_password {
            MasterPassword::Some(password) => password.clone(),
            MasterPassword::None => return Err(Error::MasterPasswordNotProvided)
        };

        // max token length is 32 characters
        if password.len() > 32 || password.is_empty() {
            return Err(Error::ApiPasswordOutOfBounds);
        }

        // confirm data present
        if self.is_empty() {
            return Err(Error::EmptyStringWhereDataExpected);
        }

        // create buffer
        let slice = password.as_bytes();
        let master_key: &mut [u8;32] = &mut [0u8;32];
        
        master_key[..slice.len()].copy_from_slice(slice);
          
        // verify data was copied
        if &master_key[..slice.len()] != slice {
            return Err(Error::SliceNotCopied);
        }

        // generate cipher
        let key = Key::<Aes256Gcm>::from_slice(master_key);
        let cipher = Aes256Gcm::new(key);

        let encrypted_data = {
            //build cipher
            let nonce = Aes256Gcm::generate_nonce(&mut OsRng).to_owned();
            let plain_text = self.as_bytes().to_owned();
            let cipher_text = actix_rt::task::spawn_blocking(move || cipher.encrypt(&nonce, plain_text.as_slice())).await??;
            
            //encrypt
            let mut buf:Vec<u8> = Vec::with_capacity(NONCE_LEN + cipher_text.len());

            //append nonce
            buf.extend_from_slice(&nonce);
            buf.extend_from_slice(&cipher_text);

            buf
        };

        Ok(encrypted_data)
    }
}