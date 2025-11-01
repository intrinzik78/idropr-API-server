use database::types::DatabaseConnection;
use sqlx::prelude::FromRow;

use crate::{
    enums::{Error,LocationPriority},
    types::{address::{self, Address}, business::Account}
};

type Result<T> = std::result::Result<T,Error>;

#[derive(Debug)]
pub struct Location {
    id: i64,
    business_account_id: i64,
    address:Address,
    priority:LocationPriority
}

// sync
impl Location {
    pub fn id(&self) -> i64 { self.id }
    pub fn address(&self) -> &Address { &self.address }
    pub fn priority(&self) -> LocationPriority { self.priority.clone() }

}

// async
impl Location {
    /// retrieve the parent of a business location
    pub async fn business_account(&self, connection: &DatabaseConnection) -> Result<Account> { 
        Account::by_id(self.business_account_id, connection)
            .await?
            .ok_or(Error::BusinessAccountNotFound(self.business_account_id))
    }

    /// retrieves single location from database
    pub async fn by_id(id:i64,connection: &DatabaseConnection) -> Result<Location> {
        DatabaseHelper::location_by_id(id, connection).await
    }

    /// retrieves all business locations from a business account from database
    pub async fn list_by_business_id(id:i64, connection: &DatabaseConnection) -> Result<Vec<Location>> {
        DatabaseHelper::list_by_business_id(id, connection).await
    }
}

#[derive(FromRow)]
struct DatabaseHelper {
    location_id:i64,
    business_account_id:i64,
    address_id:i64,
    address_1:String,
    address_2:Option<String>,
    city:String,
    state:String,
    zipcode:String,
    country:String,
    priority_id:u8
}

impl DatabaseHelper {
    pub async fn location_by_id(id:i64, connection: &DatabaseConnection) -> Result<Location> {
        let sql = "SELECT business_location.id as location_id, business_account_id,priority_id,address.id as address_id,address_1,address_2,city,state,zipcode,country
                         FROM `business_location`
                         JOIN address ON address.id = business_location.address_id
                         WHERE business_location.id = ?";
        let helper:DatabaseHelper = sqlx::query_as(sql)
            .bind(id)
            .fetch_one(&connection.pool)
            .await?;

        Ok(helper.transform()?)
    }

    pub async fn list_by_business_id(id:i64, connection: &DatabaseConnection) -> Result<Vec<Location>> {
        let sql = "SELECT business_location.id as location_id, business_account_id,priority_id,address.id as address_id,address_1,address_2,city,state,zipcode,country
                         FROM `business_location`
                         JOIN address ON address.id = business_location.address_id
                         WHERE business_location.business_account_id = ?";
        let mut rows:Vec<DatabaseHelper> = sqlx::query_as(sql)
            .bind(id)
            .fetch_all(&connection.pool)
            .await?;

        let mut locations:Vec<Location> = Vec::new();

        while let Some(record) = rows.pop() {
            locations.push(record.transform()?);
        }

        Ok(locations)
    }

    fn transform(self) -> Result<Location> {
        let address = address::Builder::default()
            .id(self.address_id)
            .address_1(self.address_1)
            .address_2(self.address_2)
            .city(self.city)
            .state(self.state)
            .zipcode(self.zipcode)
            .country(self.country)
            .build()?;

        let priority = LocationPriority::from_u8(self.priority_id)?;

        let location = Location {
            id: self.location_id,
            business_account_id: self.business_account_id,
            address,
            priority
        };

        Ok(location)
    }
}