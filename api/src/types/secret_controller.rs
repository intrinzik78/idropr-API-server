use std::{collections::HashMap, sync::RwLock};

use crate::{enums::{Error, MasterPassword},types::{DatabaseConnection, DecryptedSecret, EncryptedSecret}};

type Result<T> = std::result::Result<T,Error>;

#[derive(Debug,Default)]
pub struct SecretController {
    map: RwLock<HashMap<String,DecryptedSecret>>,
    master_password: MasterPassword
}

impl SecretController {
    pub async fn new(master_password: MasterPassword, database: &DatabaseConnection) -> Result<Self> {
        // decrypt secrets and store in list
        let encrypted_list:Vec<EncryptedSecret> = EncryptedSecret::get_all(database).await?;
        let mut decrypted_list: Vec<DecryptedSecret> = Vec::with_capacity(encrypted_list.len());

        for secret in encrypted_list {
            let s = secret.decrypt(master_password.clone()).await?;
            decrypted_list.push(s);
        }

        // map decrypted list
        let mut map: HashMap<String, DecryptedSecret> = HashMap::with_capacity(decrypted_list.len());

        while let Some(secret) = decrypted_list.pop() {
            let name = secret.name().to_owned();
            map.insert(name, secret);
        }

        Ok(Self {
            map: RwLock::new(map),
            master_password
        })
    }

    pub fn list(&self) -> Result<Vec<String>> {
        // begin locked read scope
        let list = {
            let locked_list = self.map.read().map_err(|_e| Error::PoisonedApiSecretsList)?;
            let mut list:Vec<String> = Vec::with_capacity(locked_list.len());

            for (key,_) in locked_list.iter() {
                list.push(key.clone());
            }

            list
        };
        // end locked read scope

        Ok(list)
    }

    pub fn get(&self, name: &str) -> Result<Option<DecryptedSecret>> {
        // begin locked read scope
        {
            let locked_list = self.map.read().map_err(|_e| Error::PoisonedApiSecretsList)?;

            if let Some(secret)  = locked_list.get(name) {
                let s = secret.to_owned();
                Ok(Some(s))
            } else {
                Ok(None)
            }
        }
        // end locked read scope
    }

    pub fn master_password(&self) -> &MasterPassword {
        &self.master_password
    }
}