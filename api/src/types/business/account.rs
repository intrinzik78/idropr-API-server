use crate::{
    enums::{BusinessAccountStatus,Error},
    types::{business::Location,Person}};
use database::types::DatabaseConnection;
use sqlx::{MySql, Transaction, prelude::FromRow};

type Result<T> = std::result::Result<T,Error>;

pub struct Account {
    id: i64,
    business_name:String,
    business_owner_id:i64,
    status_id:BusinessAccountStatus
}

// sync
impl Account {
    pub fn id(&self) -> i64 { self.id }
    pub fn name(&self) -> &str { &self.business_name }
    pub fn status(&self) -> BusinessAccountStatus { self.status_id.clone() }
}

// async
impl Account {
    pub async fn into_db_as_transaction(busines_name:&str, owner_id:u64, status_id:u8, tx: &mut Transaction<'static,MySql>) -> Result<i64> {
        let sql = "INSERT INTO `business_account` (business_name,business_owner_id,status_id) VALUES (?,?,?)";
        let insert_id = sqlx::query(sql)
            .bind(busines_name)
            .bind(owner_id)
            .bind(status_id)
            .execute(&mut **tx)
            .await?
            .last_insert_id() as i64;

        Ok(insert_id)
    }

    pub async fn into_db(busines_name:&str, owner_id:u64, status_id:u8, connection: &DatabaseConnection) -> Result<i64> {
        let sql = "INSERT INTO `business_account` (business_name,business_owner_id,status_id) VALUES (?,?,?)";
        let insert_id = sqlx::query(sql)
            .bind(busines_name)
            .bind(owner_id)
            .bind(status_id)
            .execute(&connection.pool)
            .await?
            .last_insert_id() as i64;

        Ok(insert_id)
    }

    pub async fn by_id(id:i64, connection: &DatabaseConnection) -> Result<Option<Account>> {
        let sql = "SELECT id,business_name,business_owner_id,status_id FROM `business_account` WHERE business_account.id = ?";
        let helper_opt: Option<DatabaseHelper> = sqlx::query_as(sql)
            .bind(id)
            .fetch_optional(&connection.pool)
            .await?;

        if let Some(helper) = helper_opt {
            let account = helper.transform()?;
            Ok(Some(account))
        } else {
            Ok(None)
        }
    }

    pub async fn owner(&self, connection: &DatabaseConnection) -> Result<Person> {
        Person::by_id(self.business_owner_id, connection).await
    }

    pub async fn locations(&self, connection: &DatabaseConnection) -> Result<Vec<Location>> {
        Location::list_by_business_id(self.id, connection).await
    }

    pub async fn users(&self) { todo!() }
}

#[derive(FromRow)]
pub struct DatabaseHelper {
    id: i64,
    business_name:String,
    business_owner_id:i64,
    status_id:u8  
}

impl DatabaseHelper {
    pub fn transform(self) -> Result<Account> {
        let account = Account {
            id: self.id,
            business_name: self.business_name,
            business_owner_id: self.business_owner_id,
            status_id: BusinessAccountStatus::from_u8(self.status_id)?
        };

        Ok(account)
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn builder() {
        let builder = DatabaseHelper {
            id: 0,
            business_name: String::from("business_name"),
            business_owner_id: 1,
            status_id: 1
        };

        let business_account = builder.transform().unwrap();
        let test_status = BusinessAccountStatus::from_u8(1).unwrap();

        assert_eq!(business_account.id,0);
        assert_eq!(business_account.business_name,String::from("business_name"));
        assert_eq!(business_account.business_owner_id,1);
        assert_eq!(business_account.status(),test_status);
    }

    #[actix_rt::test]
    async fn get_owner() {
        let connection = DatabaseConnection::new().await.unwrap();

        let builder = DatabaseHelper {
            id: 0,
            business_name: String::from("business_name"),
            business_owner_id: 1,
            status_id: 1
        };

        let business_account = builder.transform().unwrap();
        let person = business_account.owner(&connection).await;

        // test person should exist
        assert!(person.is_ok());

        let builder = DatabaseHelper {
            id: 0,
            business_name: String::from("business_name"),
            business_owner_id: 0, // does not exist
            status_id: 1
        };

        let business_account = builder.transform().unwrap();
        let person = business_account.owner(&connection).await;

        // test person should not exist
        assert!(person.is_err());
    }

    #[actix_rt::test]
    async fn business_account_locations() {
        let connection = DatabaseConnection::new().await.unwrap();

        let account = Account::by_id(1, &connection).await.unwrap().ok_or(Error::AccountStatusOutOfBounds).unwrap();
        let locations = account.locations(&connection).await.unwrap();

        assert!(!locations.is_empty());
    }
}


// WITH input AS (
//   SELECT geom FROM main.zcta WHERE zipcode = :zip
// )
// SELECT
//   bl.id AS business_location_id,
//   bl.business_account_id,
//   ST_Distance_Sphere(i.geom, z.geom) AS meters
// FROM input i
// JOIN main.business_locations bl ON bl.address_id IS NOT NULL
// JOIN main.address a             ON a.id = bl.address_id
// JOIN main.zcta z                ON z.zipcode = a.zipcode
// ORDER BY meters
// LIMIT 1;
