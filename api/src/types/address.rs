use crate::enums::Error;
use database::types::DatabaseConnection;
use sqlx::{MySql, Transaction, prelude::FromRow};

type Result<T> = std::result::Result<T,Error>;

#[derive(Debug,FromRow)]
pub struct Address {
    id: i64,
    address_1:String,
    address_2:Option<String>,
    city:String,
    state:String,
    zipcode:String,
    country:String
}

// sync
impl Address {
    pub fn id(&self) -> i64 { self.id }
    pub fn address_1(&self) -> &str { &self.address_1 }
    pub fn address_2(&self) -> Option<&String> { self.address_2.as_ref() }
    pub fn city(&self) -> &str { &self.city }
    pub fn state(&self) -> &str { &self.state }
    pub fn zipcode(&self) -> &str { &self.zipcode }
    pub fn country(&self) -> &str { &self.country }
}

// async
impl Address {
    async fn into_db_as_transaction(builder: &Builder, tx: &mut Transaction<'static,MySql>) -> Result<i64> {
        let sql = "INSERT INTO `address` (address_1,address_2,city,state,zipcode,country) VALUES (?,?,?,?,?,?)";
        let insert_id = sqlx::query(sql)
            .bind(&builder.address_1)
            .bind(&builder.address_2)
            .bind(&builder.city)
            .bind(&builder.state)
            .bind(&builder.zipcode)
            .bind(&builder.country)
            .execute(&mut **tx)
            .await?
            .last_insert_id() as i64;

        Ok(insert_id)
    }

    async fn into_db(builder: &Builder, connection: &DatabaseConnection) -> Result<i64> {
        let sql = "INSERT INTO `address` (address_1,address_2,city,state,zipcode,country VALUES (?,?,?,?,?,?)";
        let insert_id = sqlx::query(sql)
            .bind(&builder.address_1)
            .bind(&builder.address_2)
            .bind(&builder.city)
            .bind(&builder.state)
            .bind(&builder.zipcode)
            .bind(&builder.country)
            .execute(&connection.pool)
            .await?
            .last_insert_id() as i64;

        Ok(insert_id)
    }

    pub async fn by_id(id:i64, connection: &DatabaseConnection) -> Result<Option<Address>> {
        let sql = "SELECT id,address_1,address_2,city,state,zipcode,country FROM `address` WHERE address.id = ?";
        let address_opt: Option<Address> = sqlx::query_as(sql)
            .bind(id)
            .fetch_optional(&connection.pool)
            .await?;

        Ok(address_opt)
    }
}


#[derive(FromRow,Default)]
pub struct Builder {
    pub id:Option<i64>,
    pub address_1:Option<String>,
    pub address_2:Option<String>,
    pub city:Option<String>,
    pub state:Option<String>,
    pub zipcode:Option<String>,
    pub country:Option<String>
}

/// exposes builder pattern to create Addresses ergonomically
impl Builder {
    pub fn id(mut self, id:i64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn address_1(mut self, s:String) -> Self {
        self.address_1 = Some(s);
        self
    }

    pub fn address_2(mut self, s_opt:Option<String>) -> Self {
        self.address_2 = s_opt;
        self
    }

    pub fn city(mut self, s:String) -> Self {
        self.city = Some(s);
        self
    }

    pub fn state(mut self, s:String) -> Self {
        self.state = Some(s);
        self
    }

    pub fn zipcode(mut self, s:String) -> Self {
        self.zipcode = Some(s);
        self
    }

    pub fn country(mut self, s:String) -> Self {
        self.country = Some(s);
        self
    }

    pub fn build(self) -> Result<Address> {
        type E = Error;
        let id =            self.id.ok_or(E::AddressBuilderMissingData(String::from("id")))?;
        let address_1 =  self.address_1.ok_or(E::AddressBuilderMissingData(String::from("address_1")))?;
        let address_2 =  self.address_2;
        let city =       self.city.ok_or(E::AddressBuilderMissingData(String::from("city")))?;
        let state =      self.state.ok_or(E::AddressBuilderMissingData(String::from("state")))?;
        let zipcode =    self.zipcode.ok_or(E::AddressBuilderMissingData(String::from("zipcode")))?;
        let country =    self.country.ok_or(E::AddressBuilderMissingData(String::from("country")))?;

        let address = Address {
            id,
            address_1,
            address_2,
            city,
            state,
            zipcode,
            country
        };

        Ok(address)
    }

    pub async fn into_db(mut self, connection: &DatabaseConnection) -> Result<Address> {
        let id = Address::into_db(&self, connection).await?;
        self.id = Some(id);

        Self::build(self)
    }

    pub async fn into_db_as_transaction(mut self, tx: &mut Transaction<'static,MySql>) -> Result<Address> {
        let id = Address::into_db_as_transaction(&self, tx).await?;
        self.id = Some(id);

        Self::build(self)
    }
}



#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn address_builder() {

        // positive assertion tests
        let builder = Builder::default()
            .id(0)
            .address_1(String::from("address_1"))
            .address_2(Some(String::from("address_2")))
            .city(String::from("city"))
            .state(String::from("state"))
            .zipcode(String::from("12345"))
            .country(String::from("country"));

        let address = builder.build().unwrap();

        assert_eq!(address.id,0);
        assert_eq!(address.address_1,String::from("address_1"));
        assert_eq!(address.address_2,Some(String::from("address_2")));
        assert_eq!(address.city,String::from("city"));
        assert_eq!(address.state,String::from("state"));
        assert_eq!(address.zipcode,String::from("12345"));
        assert_eq!(address.country,String::from("country"));
    }

    #[test]
    fn address_missing_id() {
        let address = Builder::default().build();

        if let Some(err) = address.err() {
            match err {
                Error::AddressBuilderMissingData(s) => assert_eq!(s,String::from("id")),
                _ => panic!("incorrect error message in address builder")
            }
        } else {
            panic!("failed to return an error on missing address.id")
        }
    }

    #[test]
    fn missing_address_1() {
        let address = Builder::default()
            .id(0)
            .build();

        if let Some(err) = address.err() {
            match err {
                Error::AddressBuilderMissingData(s) => assert_eq!(s,String::from("address_1")),
                _ => panic!("incorrect error message in address builder")
            }
        } else {
            panic!("failed to return an error on missing address.address_1")
        }
    }

    #[test]
    fn missing_city() {
        let address = Builder::default()
            .id(0)
            .address_1(String::from("address_1"))
            .address_2(Some(String::from("address_2")))
            .build();

        if let Some(err) = address.err() {
            match err {
                Error::AddressBuilderMissingData(s) => assert_eq!(s,String::from("city")),
                _ => panic!("incorrect error message in address builder")
            }
        } else {
            panic!("failed to return an error on missing address.city")
        }
    }

    #[test]
    fn missing_state() {
        let address = Builder::default()
            .id(0)
            .address_1(String::from("address_1"))
            .address_2(Some(String::from("address_2")))
            .city(String::from("city"))
            .build();

        if let Some(err) = address.err() {
            match err {
                Error::AddressBuilderMissingData(s) => assert_eq!(s,String::from("state")),
                _ => panic!("incorrect error message in address builder")
            }
        } else {
            panic!("failed to return an error on missing address.state")
        }
    }

    #[test]
    fn missing_zipcode() {
        let address = Builder::default()
            .id(0)
            .address_1(String::from("address_1"))
            .address_2(Some(String::from("address_2")))
            .city(String::from("city"))
            .state(String::from("state"))
            .build();

        if let Some(err) = address.err() {
            match err {
                Error::AddressBuilderMissingData(s) => assert_eq!(s,String::from("zipcode")),
                _ => panic!("incorrect error message in address builder")
            }
        } else {
            panic!("failed to return an error on missing address.zipcode")
        }
    }

    #[test]
    fn missing_country() {
        let address = Builder::default()
            .id(0)
            .address_1(String::from("address_1"))
            .address_2(Some(String::from("address_2")))
            .city(String::from("city"))
            .state(String::from("state"))
            .zipcode(String::from("12345"))
            .build();

        if let Some(err) = address.err() {
            match err {
                Error::AddressBuilderMissingData(s) => assert_eq!(s,String::from("country")),
                _ => panic!("incorrect error message in address builder")
            }
        } else {
            panic!("failed to return an error on missing address.country")
        }
    }

    #[actix_rt::test]
    async fn address_insert() {
        use database::types::DatabaseConnection;

        let mut tx = DatabaseConnection::new()
            .await
            .unwrap()
            .pool
            .begin()
            .await
            .unwrap();

        let builder = Builder::default()
            .address_1(String::from("address_1"))
            .address_2(Some(String::from("address_2")))
            .city(String::from("city"))
            .state(String::from("state"))
            .zipcode(String::from("77023"))         // must be a valid zipcode contained in the zcta
            .country(String::from("country"));

        let address_id = Address::into_db_as_transaction(&builder, &mut tx).await.unwrap();
        let address = builder
            .id(address_id)
            .build();

        assert!(address.is_ok());

        tx.rollback().await.unwrap();
    }
}






