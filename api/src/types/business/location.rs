use database::types::DatabaseConnection;
use sqlx::prelude::FromRow;

use crate::{
    enums::{Error,LocationPriority, Permission},
    traits::HasPermission,
    types::{address, business::Account, permissions::UserPermissions}
};

type Result<T> = std::result::Result<T,Error>;

#[derive(Debug)]
pub struct Location {
    id: i64,
    business_account_id: i64,
    address:address::Address,
    priority:LocationPriority
}

// sync
impl Location {
    pub fn id(&self) -> i64 { self.id }
    pub fn business_id(&self) -> i64 { self.business_account_id }
    pub fn address(&self) -> &address::Address { &self.address }
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

    /// retrieves single location from database to a private endpoint [read_self]
    pub async fn read_self_by_id(location_id:i64, user_id:i64, permissions: &UserPermissions, connection: &DatabaseConnection) -> Result<Location> {
        let resource = crate::enums::Resource::Locations;
        let required = UserPermissions::new().with_read_self(resource);

        if !matches!(permissions.has_permission(&required),Permission::Granted) {
            return Err(Error::InsufficientLocationPermissions)
        }

        let location = DatabaseHelper::private_by_id(location_id, user_id, connection)
            .await?
            .ok_or(Error::LocationRecordNotFoundById)?;

        Ok(location)
    }

    /// retrieves single location from database to a private endpoint [read_any]
    pub async fn read_any_by_id(location_id:i64, permissions: &UserPermissions, connection: &DatabaseConnection) -> Result<Location> {
        let resource = crate::enums::Resource::Locations;
        let required = UserPermissions::new().with_read_any(resource);

        if !matches!(permissions.has_permission(&required),Permission::Granted) {
            return Err(Error::InsufficientLocationPermissions)
        }

        let location = DatabaseHelper::pub_by_location_id(location_id, connection)
            .await?
            .ok_or(Error::LocationRecordNotFoundById)?;

        Ok(location)
    }

    /// retrieves all business locations from a business account from database
    pub async fn list_by_business_id(id:i64, connection: &DatabaseConnection) -> Result<Vec<Location>> {
        DatabaseHelper::pub_list_by_business_id(id, connection).await
    }

    /// retrieves a list of locations, sorted by distance from an input zipcode [read_any]
    pub async fn read_any_nearest_by_zipcode(zipcode:&str, permissions: &UserPermissions, connection: &DatabaseConnection) -> Result<Vec<Location>> {
        let resource = crate::enums::Resource::Locations;
        let required_permissions = UserPermissions::new().with_read_any(resource);

        if !matches!(permissions.has_permission(&required_permissions),Permission::Granted) {
            return Err(Error::InsufficientLocationPermissions)
        }

        DatabaseHelper::pub_list_nearest_by_zipcode(zipcode,connection).await
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
    async fn private_by_id(location_id:i64, user_id:i64, connection: &DatabaseConnection) -> Result<Option<Location>> {
        let sql = "SELECT business_location.id AS location_id, business_location.business_account_id,priority_id,address.id AS address_id,address_1,address_2,city,state,zipcode,country
                        FROM `business_location`
                        JOIN business_account_users ON business_account_users.business_account_id = business_location.business_account_id
                        JOIN address ON address.id = business_location.address_id
                        WHERE business_location.id = ? AND business_account_users.user_id = ?";
        let helper_opt:Option<DatabaseHelper> = sqlx::query_as(sql)
            .bind(location_id)
            .bind(user_id)
            .fetch_optional(&connection.pool)
            .await?;

        if let Some(helper) = helper_opt {
            let response = helper.transform()?;
            Ok(Some(response))
        } else {
            Ok(None)
        }
    }

    async fn pub_by_location_id(location_id:i64, connection: &DatabaseConnection) -> Result<Option<Location>> {
        let sql = "SELECT business_location.id as location_id, business_account_id,priority_id,address.id as address_id,address_1,address_2,city,state,zipcode,country
                        FROM `business_location`
                        JOIN address ON address.id = business_location.address_id
                        WHERE business_location.id = ?";
        let helper_opt:Option<DatabaseHelper> = sqlx::query_as(sql)
            .bind(location_id)
            .fetch_optional(&connection.pool)
            .await?;

        if let Some(helper) = helper_opt {
            let response = helper.transform()?;
            Ok(Some(response))
        } else {
            Ok(None)
        }
    }

    async fn pub_list_by_business_id(id:i64, connection: &DatabaseConnection) -> Result<Vec<Location>> {
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

    async fn pub_list_nearest_by_zipcode(zipcode:&str, connection: &DatabaseConnection) -> Result<Vec<Location>> {
        let sql = "WITH input AS (SELECT geom FROM main.zcta WHERE zipcode = ?)
                        SELECT bl.id AS location_id, bl.business_account_id,bl.priority_id,a.id AS address_id,a.address_1,a.address_2,a.city,a.state,a.zipcode,a.country, ST_Distance_Sphere(i.geom, z.geom) AS meters
                        FROM input i
                        JOIN main.business_location bl ON bl.address_id IS NOT NULL
                        JOIN main.address a ON a.id = bl.address_id
                        JOIN main.zcta z ON z.zipcode = a.zipcode

                        ORDER BY meters, bl.id

                        LIMIT 3";

        let helper_list:Vec<DatabaseHelper> = sqlx::query_as(sql)
            .bind(zipcode)
            .fetch_all(&connection.pool)
            .await?;

        if helper_list.is_empty() { return Ok(Vec::new()) }

        let mut location_list:Vec<Location> = Vec::with_capacity(3);

        for helper in helper_list {
            let location = helper.transform()?;
            location_list.push(location);
        }

        Ok(location_list)
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