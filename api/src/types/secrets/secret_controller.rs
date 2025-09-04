use std::{collections::HashMap, sync::RwLock, time::{Duration, Instant}};

use sqlx::{MySql, Transaction};

use crate::{
    enums::{Error, MasterPassword, RowsAffected, RowsUpdated},
    traits::ToEncryptedBuffer,
    types::{secrets::{DecryptedSecret, EncryptedSecret}, DatabaseConnection}
};

type Result<T> = std::result::Result<T,Error>;

const REFRESH_TIME:u64 = 60 * 60 * 24; // 24 hours

#[derive(Clone,Debug)]
struct Inner {
    map: HashMap<String,DecryptedSecret>,
    next_refresh: Instant
}

impl Inner {
    pub fn with_map(mut self, map: HashMap<String,DecryptedSecret>) -> Self {
        self.map = map;
        self
    }
}

impl Default for Inner {
    fn default() -> Self {
        let next_refresh = Instant::now()
            .checked_add(Duration::from_secs(REFRESH_TIME))
            .expect("could not establish next update time during startup");

        Inner {
            map: HashMap::with_capacity(0),
            next_refresh
        }
    }
}

#[derive(Debug,Default)]
pub struct SecretController {
    inner: RwLock<Inner>,
    master_password: RwLock<MasterPassword>
}

impl SecretController {

    /// removes a secret from memory
    pub async fn delete(&mut self, secret_name:&str) -> Result<()> {
        // begin locked read scope
        {
            let mut locked_inner = self.inner.write().map_err(|_e| Error::PoisonedApiSecretsList)?;
            let _ = locked_inner.map.remove(secret_name);
        }
        // end locked read scope

        Ok(())   
    }

    /// builder: inserts all database secrets into memory and returns the controller
    pub async fn new(master_password: MasterPassword, database: &DatabaseConnection) -> Result<Self> {
        // decrypt secrets and store in list
        let encrypted_list:Vec<EncryptedSecret> = EncryptedSecret::get_all(database).await?;
        let mut decrypted_list: Vec<DecryptedSecret> = Vec::with_capacity(encrypted_list.len());

        for secret in encrypted_list {
            let s = secret.decrypt(&master_password).await?;
            decrypted_list.push(s);
        }

        // map decrypted list
        let mut map: HashMap<String, DecryptedSecret> = HashMap::with_capacity(decrypted_list.len());

        while let Some(secret) = decrypted_list.pop() {
            let name = secret.name().to_owned();
            map.insert(name, secret);
        }

        // build inner
        let unlocked_inner = Inner::default().with_map(map);

        let inner = RwLock::new(unlocked_inner);

        Ok(Self {
            inner,
            master_password: RwLock::new(master_password)
        })
    }

    /// replaces the inner map with an updated secrets list from the database
    pub async fn refresh_secrets_list(&self, database: &DatabaseConnection) -> Result<()> {
        // decrypt secrets and store in list
        let encrypted_list:Vec<EncryptedSecret> = EncryptedSecret::get_all(database).await?;
        
        // decrypt secrets and store in map
        let mut map: HashMap<String, DecryptedSecret> = HashMap::with_capacity(encrypted_list.len());
        
        for secret in encrypted_list {
            let decrypted = secret.decrypt(&self.master_password()).await?;
            map.insert(decrypted.name().to_owned(), decrypted);
        }

        // set inner last update time and next refresh
        let now = Instant::now();
        let next_refresh = match now.checked_add(Duration::from_secs(REFRESH_TIME)) {
            Some(t) => t,
            None => now
        };

        // begin locked write scope
        {
            // lock and replace the map
            let mut locked_inner = self.inner.write().map_err(|_e| Error::PoisonedApiSecretsList)?;
            locked_inner.map = map;
            locked_inner.next_refresh = next_refresh;
        }
        // end locked write scope

        Ok(())
    }

    /// returns a list of secret names
    pub fn list_secret_names(&self) -> Result<Vec<String>> {

        // begin locked read scope
        let list = {
            let locked_inner = self.inner.read().map_err(|_e| Error::PoisonedApiSecretsList)?;
            let map = &locked_inner.map;

            let mut list:Vec<String> = Vec::with_capacity(map.len());

            for (secret_name,_) in map.iter() {
                list.push(secret_name.to_owned());
            }

            list
        };
        // end locked read scope

        Ok(list)
    }

    /// secret getter, returns an owned copy of the secret
    pub fn get(&self, secret_name: &str) -> Result<Option<DecryptedSecret>> {
        let now = Instant::now();

        // begin locked read scope
        {
            let locked_inner = self.inner.read().map_err(|_e| Error::PoisonedApiSecretsList)?;
            let map = &locked_inner.map;

            if now > locked_inner.next_refresh  {
                return Err(Error::ApiSecretsOutOfSyncWithDatabase);
            }

            if let Some(secret)  = map.get(secret_name) {
                let s = secret.to_owned();
                Ok(Some(s))
            } else {
                Ok(None)
            }
        }
        // end locked read scope
    }

    /// takes an unencrypted secret, encrypts it and stores it in memory and on the database
    pub async fn new_secret(&self, secret: DecryptedSecret, database: &DatabaseConnection) -> Result<()> {
        
        // short circuit if secret name is already in memory
        if self.get(secret.name())?.is_some() {
            return Err(Error::DuplicateSecretNameExists);
        }

        // encrypt
        let encrypted = secret.encrypt(&self.master_password()).await?;
        
        // insert into db
        let mut tx = database.pool.begin().await?;
        encrypted.into_db_as_transaction(&mut tx).await?;
        
        // verify insertion & refresh secrets list
        match EncryptedSecret::by_name_as_transaction(secret.name(),&mut tx).await? {
            Some(_) => {
                tx.commit().await?;
                self.refresh_secrets_list(database).await?;
                Ok(())
            },
            None => {
                tx.rollback().await?;
                Err(Error::DatabaseTransactionVerification)
            }
        }
    }

    /// update an existing secret name, by name
    pub async fn update_secret_name(&self, cur_name: &str, new_name: &str, database: &DatabaseConnection) -> Result<()> {
        // validate target is in memory
        let _mem_secret = self
            .get(cur_name)?
            .ok_or(Error::NoApiRecordByThatName)?;
        
        // update the database record with the new api name
        let mut tx = database.pool.begin().await?;
        let () = match EncryptedSecret::update_db_name(cur_name,new_name,&mut tx).await? {
            RowsUpdated::Some(1) => {
                tx.commit().await?;
            },
            RowsUpdated::Some(_) => {
                tx.rollback().await?;
                return Err(Error::TooManyRowsUpdated);
            },
            RowsUpdated::None => {
                tx.rollback().await?;
                return Err(Error::TooManyRowsUpdated);
            }
        };

        // begin locked write scope
        {
            let mut locked_inner = self.inner.write().map_err(|_e| Error::PoisonedApiSecretsList)?;
            let map = &mut locked_inner.map;

            if let Some(secret)  = map.get_mut(cur_name) {
                secret.set_name(new_name);
            } else {
                return Err(Error::ApiSecretsOutOfSyncWithDatabase);
            }
        }
        // end locked write scope

        Ok(())
    }

    /// update an existing secret key, by name
    pub async fn update_secret_key(&self, name: &str, new_key: &str, database: &DatabaseConnection) -> Result<()> {
        // validate target is in memory
        let _mem_secret = self
            .get(name)?
            .ok_or(Error::NoApiRecordByThatName)?;

        // encrypt key
        let api_key = new_key.encrypt(&self.master_password()).await?;

        // update the database record with the encrypted secret
        let mut tx = database.pool.begin().await?;
        let () = match EncryptedSecret::update_db_api_key(name,&api_key, &mut tx).await? {
            RowsUpdated::Some(1) => {
                tx.commit().await?;
            },
            RowsUpdated::Some(_) => {
                tx.rollback().await?;
                return Err(Error::TooManyRowsUpdated);
            },
            RowsUpdated::None => {
                tx.rollback().await?;
                return Err(Error::TooManyRowsUpdated);
            }
        };

        // begin locked write scope
        {
            let mut locked_inner = self.inner.write().map_err(|_e| Error::PoisonedApiSecretsList)?;
            let map = &mut locked_inner.map;

            if let Some(secret)  = map.get_mut(name) {
                secret.set_api_key(new_key);
            } else {
                return Err(Error::ApiSecretsOutOfSyncWithDatabase);
            }
        }
        // end locked write scope

        Ok(())
    }

    /// update an existing secret secret, by name
    pub async fn update_secret_secret(&self, name: &str, new_secret: &str, database: &DatabaseConnection) -> Result<()> {  
        // validate target is in memory
        let _mem_secret = self
            .get(name)?
            .ok_or(Error::NoApiRecordByThatName)?;

        // encrypt secret
        let api_secret = new_secret.encrypt(&self.master_password()).await?;

        // update the database record with the encrypted secret
        let mut tx = database.pool.begin().await?;
        
        match EncryptedSecret::update_db_api_secret(name, &api_secret, &mut tx).await? {
            RowsUpdated::Some(1) => {
                tx.commit().await?;
            },
            RowsUpdated::Some(_) => {
                tx.rollback().await?;
                return Err(Error::TooManyRowsUpdated);
            },
            RowsUpdated::None => {
                tx.rollback().await?;
                return Err(Error::TooManyRowsUpdated);
            }
        };

        // begin locked write scope
        {
            let mut locked_inner = self.inner.write().map_err(|_e| Error::PoisonedApiSecretsList)?;
            let map = &mut locked_inner.map;

            if let Some(secret)  = map.get_mut(name) {
                secret.set_api_secret(new_secret);
            } else {
                return Err(Error::ApiSecretsOutOfSyncWithDatabase);
            }
        }
        // end locked write scope

        Ok(())
    }

    /// update an existing secret description, by name
    pub async fn update_secret_description(&self, name: &str, description: &str, database: &DatabaseConnection) -> Result<()> {
        // validate target is in memory
        let _mem_secret = self
            .get(name)?
            .ok_or(Error::NoApiRecordByThatName)?;

        
        match EncryptedSecret::update_db_description(name, description, database).await? {
            RowsUpdated::Some(1) => {},
            RowsUpdated::Some(_) => return Err(Error::TooManyRowsUpdated),
            RowsUpdated::None => return Err(Error::TooManyRowsUpdated)
        };
        
        // begin locked write scope
        {
            let mut locked_list = self.inner.write().map_err(|_e| Error::PoisonedApiSecretsList)?;
            
            if let Some(secret) = locked_list.map.get_mut(name) {
                secret.set_description(description);
            } else {
                return Err(Error::ApiSecretsOutOfSyncWithDatabase);
            }
        }
        // end locked write scope

        Ok(())
    }
    
    /// rotates the api secrets password
    /// - retreive each secret in memory from the database
    /// - decrypt it with the current password 
    /// - encrypt it with the new password
    /// - delete the old record
    /// - insert the new record
    pub async fn change_db_master_password(&self, new_password: MasterPassword, tx: &mut Transaction<'_, MySql>) -> Result<RowsUpdated> {

        // get the list of secret names in memory
        let names = self.list_secret_names()?;
        let mut count = 0_u64;

        // fetch, decrypt, delete, encrypt, insert
        for name in names.iter() {
            let encrypted_secret = match EncryptedSecret::by_name_as_transaction(name,tx).await? {
                Some(r) => r,
                None => return Err(Error::NoApiRecordByThatName)
            };

            let decrypted_secret = encrypted_secret.decrypt(&self.master_password()).await?;
            let rotated_secret = decrypted_secret.encrypt(&new_password).await?;

            match EncryptedSecret::delete_from_db_as_transaction(name, tx).await? {
                RowsAffected::Some(1) => count += 1,
                RowsAffected::Some(_) => return Err(Error::TooManyRowsUpdated),
                RowsAffected::None => return Err(Error::TooFewRowsUpdated),
            };

            let _a = rotated_secret.into_db_as_transaction(tx).await?;
        }

        match count {
            0 => Ok(RowsUpdated::None),
            _ => Ok(RowsUpdated::Some(count))
        }
    }

    #[inline]
    pub fn master_password(&self) -> MasterPassword {
        match self.master_password.read() {
            Ok(master_password) => master_password.clone(),
            Err(_) => MasterPassword::None
        }
    }

    pub fn set_master_password(&mut self, new_password: MasterPassword) {
        self.master_password = RwLock::new(new_password);
    }

}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU8;

    use super::*;
    use crate::{
        enums::{RowsAffected, Uuid},
        types::{DatabaseConnection, Env}
    };

    #[actix_rt::test]
    async fn new_rotate_delete() {
        let env = Env::default();

        // build a random uuid name
        let uuid = Uuid::web_safe(NonZeroU8::new(32)).unwrap();
        let secret_name = match uuid {
            Uuid::WebSafe(s) => s,
            _ => panic!("no api name found in uuid")
        };

        // instantiate
        let master_password = MasterPassword::Some(env.master_password.clone());
        let database = DatabaseConnection::new(&env).await.unwrap();
        let mut secret_controller = SecretController::new(master_password.clone(), &database).await.unwrap();
        

        // build another random uuid name
        let uuid = Uuid::web_safe(NonZeroU8::new(32)).unwrap();
        
        let name = match uuid {
            Uuid::WebSafe(n) => n,
            _ => panic!("no api name found in uuid")
        };
        
        // define secret properties
        let description = String::from("description");
        let api_key = String::from("api_key");
        let api_secret = String::from("api_secret");
        let secret = DecryptedSecret::new(&name, &description, &Some(api_key.clone()), &Some(api_secret.clone()));

        // encrypt
        let encrypted_secret = secret.encrypt(&master_password)
            .await
            .unwrap();

        // begin transaction
        let mut tx  = database.pool
            .begin()
            .await
            .unwrap();
        
        // insert
        let _insert_id = encrypted_secret
            .into_db_as_transaction(&mut tx)
            .await
            .unwrap();

        // commit
        tx.commit()
            .await
            .unwrap();

        // refresh the controller's decrypted secrets list
        secret_controller.refresh_secrets_list(&database).await.unwrap();

        // copy the new secret
        let mem = secret_controller
            .get(&secret_name)
            .unwrap()
            .unwrap();

        // run tests to verify decrypted data is as expected
        assert_eq!(mem.name(), name, "the decrypted api name did not match the original input name");
        assert_eq!(mem.api_key().unwrap().to_owned(), api_key, "the decrypted api key did not match the original input key");
        assert_eq!(mem.api_secret().unwrap().to_owned(), api_secret, "the decrypted api_secret did not match the original input secret");
        assert_eq!(mem.description(), description);

        // begin a new transaction
        let mut tx = database.pool
            .begin()
            .await
            .unwrap();

        // build a new password for rotating keys
        let new_password = String::from("new password");
        let new_master_password = MasterPassword::Some(new_password);
        let count = secret_controller
            .change_db_master_password(new_master_password.clone(), &mut tx)
            .await
            .unwrap();
        
        // store the new password
        secret_controller.set_master_password(new_master_password.clone());

        // commit and test
        tx.commit()
            .await
            .unwrap();

        assert_eq!(count,RowsUpdated::Some(1));
        
        let encrypted = EncryptedSecret::by_name(&name.clone(), &database)
            .await
            .unwrap()
            .unwrap();
        
        let _decrypted = encrypted.decrypt(&new_master_password.clone())
            .await
            .unwrap();

        // delete test from database
        let delete_result = EncryptedSecret::delete_from_db(&name, &database)
            .await
            .unwrap();

        assert_eq!(delete_result, RowsAffected::Some(1), "failure occured cleaning test data from the database");

        // verify record deleted from database
        let refetch = EncryptedSecret::by_name(&secret_name, &database)
            .await
            .unwrap();

        assert!(refetch.is_none());

        // delete from memory
        let delete_result = secret_controller
            .delete(&name)
            .await;

        assert!(delete_result.is_ok());

        // verify record deleted from memory
        let refetch = secret_controller
            .get(&secret_name)
            .unwrap();

        assert!(refetch.is_none());
    }
}