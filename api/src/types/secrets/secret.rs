use chrono::{DateTime,Utc};
use sqlx::{FromRow, MySql, Transaction};

use crate::{
    enums::{Error, MasterPassword, RowsAffected, RowsUpdated},
    traits::{ToAffectedResult, ToDecryptedString, ToEncryptedBuffer, ToUpdatedResult},
    types::DatabaseConnection};

type Result<T> = std::result::Result<T,Error>;

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
    /// api_key getter
    pub fn api_key(&self) -> Option<&Vec<u8>> {
        self.api_key.as_ref()
    }

    /// api_secret getter
    pub fn api_secret(&self) -> Option<&Vec<u8>> {
        self.api_secret.as_ref()
    }

    /// query db for an api secret by api name
    fn by_name_sql() -> String {
        String::from("SELECT id,name,description,api_key,api_secret,created_at,updated_at FROM `api_secrets` WHERE name = ?")
    }

    /// query db for an encrypted secret
    pub async fn by_name(secret_name: &str, database: &DatabaseConnection) -> Result<Option<EncryptedSecret>> {
        let sql = &Self::by_name_sql();
        let secret_opt: Option<EncryptedSecret> = sqlx::query_as(sql)
            .bind(secret_name)
            .fetch_optional(&database.pool)
            .await?;

        Ok(secret_opt)
    }

    /// query db for an encrypted secret during a transaction
    pub async fn by_name_as_transaction(name: &str, tx: &mut Transaction<'_,MySql>) -> Result<Option<EncryptedSecret>> {
        let sql = &Self::by_name_sql();

        let secret_opt: Option<EncryptedSecret> = sqlx::query_as(sql)
            .bind(name)
            .fetch_optional(&mut **tx)
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
    pub async fn decrypt(&self, master_password: &MasterPassword) -> Result<DecryptedSecret> {

        // extract key or create empty vec
        let api_key = match &self.api_key {
            Some(key) => key.decrypt_to_string(master_password).await?,
            None => None
        };

        // extract key or create empty vec
        let api_secret = match &self.api_secret {
            Some(secret) => secret.decrypt_to_string(master_password).await?,
            None => None
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

    /// insert record into db as a transaction
    pub async fn into_db(&self, database: &DatabaseConnection) -> Result<u64> {
        let sql = "INSERT INTO `api_secrets` (name,description,api_key,api_secret) VALUES(?,?,?,?)";
        let insert_id = sqlx::query(sql)
            .bind(&self.name)
            .bind(&self.description)
            .bind(&self.api_key)
            .bind(&self.api_secret)
            .execute(&database.pool)
            .await?
            .last_insert_id();

        Ok(insert_id)
    }

    /// update the api name in the database
    pub async fn update_db_name(cur_name: &str, new_name:&str, tx: &mut Transaction<'_,MySql>) -> Result<RowsUpdated> {
        let sql = "UPDATE `api_secrets` SET name = ? WHERE name = ?";
        let rows_updated = sqlx::query(sql)
            .bind(new_name)
            .bind(cur_name)
            .execute(&mut **tx)
            .await?
            .rows_affected()
            .to_updated_result();

        Ok(rows_updated)
    }

    /// update the api key in the database
    pub async fn update_db_api_key(name: &str, api_key: &[u8], tx: &mut Transaction<'_,MySql>) -> Result<RowsUpdated> {
        let sql = "UPDATE `api_secrets` SET api_key = ? WHERE name = ?";
        let rows_updated = sqlx::query(sql)
            .bind(api_key)
            .bind(name)
            .execute(&mut **tx)
            .await?
            .rows_affected()
            .to_updated_result();

        Ok(rows_updated)
    }

    /// update the api secret in the database
    pub async fn update_db_api_secret(name: &str, api_secret: &[u8], tx: &mut Transaction<'_,MySql>) -> Result<RowsUpdated> {
        let sql = "UPDATE `api_secrets` SET api_secret = ? WHERE name = ?";
        let rows_updated = sqlx::query(sql)
            .bind(api_secret)
            .bind(name)
            .execute(&mut **tx)
            .await?
            .rows_affected()
            .to_updated_result();

        Ok(rows_updated)
    }

    /// delete a record from the database by name
    pub async fn delete_from_db(name: &str, database: &DatabaseConnection) -> Result<RowsAffected> {
        let sql = "DELETE FROM `api_secrets` WHERE name = ? LIMIT 1";
        let rows_affected = sqlx::query(sql)
            .bind(name)
            .execute(&database.pool)
            .await?
            .rows_affected()
            .to_affected_result();

        Ok(rows_affected)
    }

    /// delete a record from the database by name
    pub async fn delete_from_db_as_transaction(name: &str, tx: &mut Transaction<'_,MySql>) -> Result<RowsAffected> {
        let sql = "DELETE FROM `api_secrets` WHERE name = ? LIMIT 1";
        let rows_affected = sqlx::query(sql)
            .bind(name)
            .execute(&mut **tx)
            .await?
            .rows_affected()
            .to_affected_result();

        Ok(rows_affected)
    }

    /// updates the api name on the database
    pub async fn update_db_description(name: &str, description: &str, database: &DatabaseConnection) -> Result<RowsUpdated> {
        let sql = "UPDATE `api_secrets` SET description = ? WHERE name = ?";
        let rows_updated = sqlx::query(sql)
            .bind(description)
            .bind(name)
            .execute(&database.pool)
            .await?
            .rows_affected()
            .to_updated_result();

        Ok(rows_updated)
    }


    pub async fn rotate_encryption() {

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

    /// name setter
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_owned();
    }

    /// name setter
    pub fn set_description(&mut self, description: &str) {
        self.description = description.to_owned();
    }

    /// api_key setter
    pub fn set_api_key(&mut self, api_key: &str) {
        self.api_key = Some(api_key.to_owned());
    }

    /// api_secret setter
    pub fn set_api_secret(&mut self, api_secret: &str) {
        self.api_secret = Some(api_secret.to_owned());
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
    pub fn api_key(&self) -> Option<&String> {
        self.api_key.as_ref()
    }

    /// public api secret getter
    pub fn api_secret(&self) -> Option<&String> {
        self.api_secret.as_ref()
    }  

    /// private encryption algo for storing keys/secrets in the database
    pub async fn encrypt(&self, master_password: &MasterPassword) -> Result<EncryptedSecret> {
        // encrypt api key
        let api_key = match &self.api_key {
            Some(api_key) => Some(api_key.as_str().encrypt(master_password).await?),
            None => None
        };

        // encrypt api secret
        let api_secret = match &self.api_secret {
            Some(api_secret) => Some(api_secret.as_str().encrypt(master_password).await?),
            None => None
        };

        Ok(EncryptedSecret {
            id: self.id,
            name: self.name.clone(),
            description: self.description.clone(),
            api_key,
            api_secret,
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

        let encrypted = decrypted_secret.encrypt(&master_password).await.unwrap();
        let decrypted = encrypted.decrypt(&master_password).await.unwrap();

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