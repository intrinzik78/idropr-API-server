use aes_gcm::{aead::{Aead, OsRng}, AeadCore, Aes256Gcm, Key, KeyInit, Nonce};
use chrono::{DateTime,Utc};
use sqlx::{FromRow, MySql, Transaction};

use crate::{enums::{Error, MasterPassword},types::DatabaseConnection};

type Result<T> = std::result::Result<T,Error>;

const NONCE_LEN:usize = 12;
const KEY_LEN:usize = 32;

#[derive(Clone,Debug,FromRow)]
pub struct EncryptedSecret {
    id: Option<i64>,
    name: String,
    description: String,
    api_key: Option<Vec<u8>>,
    api_secret: Option<Vec<u8>>,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>
}

#[derive(Clone,Debug)]
pub struct DecryptedSecret {
    id: Option<i64>,
    name: String,
    description: String,
    api_key: Option<String>,
    api_secret: Option<String>,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>
}

//async
impl EncryptedSecret {

    /// query db for an api secret by api name
    pub async fn by_name(secret_name: &str, database: &DatabaseConnection) -> Result<Option<EncryptedSecret>> {
        let sql = "SELECT id,name,description,api_key,api_secret,created_at,updated_at FROM `api_secrets` WHERE name = ?";
        let secret_opt: Option<EncryptedSecret> = sqlx::query_as(sql)
            .bind(secret_name)
            .fetch_optional(&database.pool)
            .await?;

        Ok(secret_opt)
    }

    /// query db for an api secret by record id
    pub async fn by_id(secret_id: i64, database: &DatabaseConnection) -> Result<EncryptedSecret> {
        let sql = "SELECT id,name,description,api_key,api_secret,created_at,updated_at FROM `api_secrets` WHERE id = ?";
        let secret: EncryptedSecret = sqlx::query_as(sql)
            .bind(secret_id)
            .fetch_one(&database.pool)
            .await?;
        
        Ok(secret)
    }

    /// retrieve all db records from the database
    pub async fn get_all(database: &DatabaseConnection) -> Result<Vec<EncryptedSecret>> {
        let sql = "SELECT id,name,description,api_key,api_secret,created_at,updated_at FROM `api_secrets`";
        let list: Vec<EncryptedSecret> = sqlx::query_as(sql)
            .fetch_all(&database.pool)
            .await?;

        Ok(list)
    }

    /// private decryption algo which decrypts on a spawned blocking thread
    pub async fn decrypt(&self, master_password: MasterPassword) -> Result<DecryptedSecret> {
        let password = match &master_password {
            MasterPassword::Some(password) => password.clone(),
            MasterPassword::None => return Err(Error::MasterPasswordNotProvided)
        };
        
        let encrypted_key = {
            if let Some(key) = &self.api_key {
                key.to_owned()
            } else {
                Vec::with_capacity(0)
            }
        };

        let encrypted_secret = {
            if let Some(secret) = &self.api_secret {
                secret.to_owned()
            } else {
                Vec::with_capacity(0)
            }
        };

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
        let api_key = {
            if !encrypted_key.is_empty() {
                let decrypted_data= actix_rt::task::spawn_blocking(move || {
                    let nonce_bytes = &encrypted_key[..NONCE_LEN];
                    let nonce = Nonce::from_slice(nonce_bytes);
                    let ciphertext = &encrypted_key[NONCE_LEN..];

                    cipher.decrypt(nonce, ciphertext)
                })
                .await??;

                // try convert to utf8 string
                let plain_text = String::from_utf8(decrypted_data)?;
                
                Some(plain_text)
            } else {
                None
            }
        };

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
        let api_secret = {
            if !encrypted_secret.is_empty() {
                let decrypted_data= actix_rt::task::spawn_blocking(move || {
                    let nonce_bytes = &encrypted_secret[..NONCE_LEN];
                    let nonce = Nonce::from_slice(nonce_bytes);
                    let ciphertext = &encrypted_secret[NONCE_LEN..];

                    cipher.decrypt(nonce, ciphertext)
                })
                .await??;

                // try convert to utf8 string
                let plain_text = String::from_utf8(decrypted_data)?;
                
                Some(plain_text)
            } else {
                None
            }
        }; 

        Ok(DecryptedSecret {
            id: self.id,
            name: self.name.clone(),
            description: self.description.clone(),
            api_key,
            api_secret,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }

    /// insert record into db as a transaction
    pub async fn into_db_as_transaction(&self, tx: &mut Transaction<'_, MySql>) -> Result<u64> {
        let sql = "INSERT INTO `api_secrets` (name,description,api_key,api_secret) VALUES(?,?,?,?)";
        let insert_id = sqlx::query(sql)
            .bind(&self.name)
            .bind(&self.description)
            .bind(&self.api_key)
            .bind(&self.api_secret)
            .execute(&mut **tx)
            .await?
            .last_insert_id();

        Ok(insert_id)
    }
}

// sync
impl DecryptedSecret {
    pub fn new(name: &str, description: &str, api_key: &Option<String>, api_secret: &Option<String>) -> Self {
        Self {
            id: None,
            name: name.to_owned(),
            description: description.to_owned(),
            api_key: api_key.to_owned(),
            api_secret: api_secret.to_owned(),
            created_at: None,
            updated_at: None
        }
    }

    /// id getter
    pub fn id(&self) -> Option<i64> {
        self.id
    }

    /// secret name getter
    pub fn name(&self) -> &str {
        &self.name
    }

    /// description getter
    pub fn description(&self) -> &str {
        &self.description
    }

    /// creation timestamp getter
    pub fn created_at(&self) -> Option<&DateTime<Utc>> {
        self.created_at.as_ref()
    }

    /// last updated timestamp getter
    pub fn updated_at(&self) -> Option<&DateTime<Utc>> {
        self.updated_at.as_ref()
    }

    /// public api key getter
    pub async fn api_key(&self) -> Option<&String> {
        self.api_key.as_ref()
    }

    /// public api secret getter
    pub async fn api_secret(&self) -> Option<&String> {
        self.api_secret.as_ref()
    }  

    /// private encryption algo for storing keys/secrets in the database
    pub async fn encrypt(&self, master_password: MasterPassword) -> Result<EncryptedSecret> {
        let password = match &master_password {
            MasterPassword::Some(password) => password.clone(),
            MasterPassword::None => return Err(Error::MasterPasswordNotProvided)
        };

        // max token length is 32 characters
        if password.len() > 32 || password.is_empty() {
            return Err(Error::ApiPasswordOutOfBounds);
        }

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

        let encrypted_key = {
            if let Some(api_key) = &self.api_key {
                //build cipher
                let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
                let plain_text = api_key.as_bytes();
                let cipher_text = cipher.encrypt(&nonce, plain_text)?;
                
                //encrypt
                let mut encrypted_api_secret:Vec<u8> = Vec::with_capacity(NONCE_LEN + cipher_text.len());

                //append nonce
                encrypted_api_secret.extend_from_slice(&nonce);
                encrypted_api_secret.extend_from_slice(&cipher_text);

                Some(encrypted_api_secret)
            } else {
                None
            }
        };

        let encrypted_secret = {
            if let Some(api_secret) = &self.api_secret {
                // build cipher
                let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
                let plain_text = api_secret.as_bytes();
                let cipher_text = cipher.encrypt(&nonce, plain_text)?;
                
                // encrypt
                let mut encrypted_api_secret:Vec<u8> = Vec::with_capacity(NONCE_LEN + cipher_text.len());

                //append nonce
                encrypted_api_secret.extend_from_slice(&nonce);
                encrypted_api_secret.extend_from_slice(&cipher_text);

                Some(encrypted_api_secret)
            } else {
                None
            }
        };

        Ok(EncryptedSecret {
            id: self.id,
            name: self.name.clone(),
            description: self.description.clone(),
            api_key: encrypted_key,
            api_secret: encrypted_secret,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }

}

#[cfg(test)]
mod tests {
    use std::num::NonZero;

    use super::*;
    use crate::enums::Uuid;

    #[actix_rt::test]
    async fn secret_builder() {
        let uuid = Uuid::web_safe_with_nums(NonZero::new(16)).unwrap();
        let master_password = match uuid {
            Uuid::WebSafeNums(s) => MasterPassword::Some(s),
            _ => MasterPassword::None
        };
        let now = Utc::now();
        let key = String::from("key");
        let secret = String::from("secret");
        let decrypted_secret = DecryptedSecret {
            id: Some(0),
            name: String::from("name"),
            description: String::from("description"),
            api_key: Some(key),
            api_secret: Some(secret),
            created_at: Some(now.clone()),
            updated_at: Some(now.clone())
        };

        let encrypted = decrypted_secret.encrypt(master_password.clone()).await.unwrap();
        let decrypted = encrypted.decrypt(master_password.clone()).await.unwrap();

        assert_eq!(decrypted.api_key,decrypted_secret.api_key);
        assert_eq!(decrypted.api_secret,decrypted_secret.api_secret);
        assert_eq!(decrypted.id, decrypted_secret.id);
        assert_eq!(decrypted.name, decrypted_secret.name);
        assert_eq!(decrypted.description, decrypted_secret.description);
        assert_eq!(decrypted.api_key, decrypted_secret.api_key);
        assert_eq!(decrypted.api_secret, decrypted_secret.api_secret);
        assert_eq!(decrypted.created_at, decrypted_secret.created_at);
        assert_eq!(decrypted.updated_at, decrypted_secret.updated_at);
    }
}