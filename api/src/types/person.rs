use chrono::{NaiveDate};
use database::types::DatabaseConnection;
use sqlx::{MySql, Transaction, prelude::FromRow};

use crate::enums::Error;

type Result<T> =  std::result::Result<T,Error>;

#[derive(Debug,FromRow)]
pub struct Person {
    id: i64,                    // required
    f_name: Option<String>,
    l_name: Option<String>,
    email: String,              // required
    phone: Option<String>,
    birthday: Option<NaiveDate>
}

// sync
impl Person {
    pub fn id(&self) -> i64 { self.id }
    pub fn f_name(&self) -> Option<&String> { self.f_name.as_ref() }
    pub fn l_name(&self) -> Option<&String> { self.l_name.as_ref() }
    pub fn email(&self) -> &str { &self.email }
    pub fn phone(&self) -> Option<&String> { self.phone.as_ref() }
    pub fn birthday(&self) -> Option<&NaiveDate> { self.birthday.as_ref() }
}

// async
impl Person {
    /// returns a person record from the database. assumes the record exists (searching by id), returning an error on NULL
    pub async fn by_id(id:i64, connection: &DatabaseConnection) -> Result<Person> {
        let sql = "SELECT id,f_name,l_name,email,phone,birthday FROM `person` WHERE person.id = ?";
        let person = sqlx::query_as(sql)
            .bind(id)
            .fetch_one(&connection.pool)
            .await?;

        Ok(person)
    }

    /// returns an optional person record from the database, searching on email
    pub async fn by_email(email:&str, connection: &DatabaseConnection) -> Result<Option<Person>> {
        let sql = "SELECT id,f_name,l_name,email,phone,birthday FROM `person` WHERE person.email = ?";
        let person_opt: Option<Person> = sqlx::query_as(sql)
            .bind(email)
            .fetch_optional(&connection.pool)
            .await?;

        if let Some(person) = person_opt {
            Ok(Some(person))
        } else {
            Ok(None)
        }
    }

    /// inserts a person record into the database
    pub async fn into_db(new_person: &NewPerson, connection: &DatabaseConnection) -> Result<i64> {
        let sql = "INSERT INTO `person` (f_name,l_name,email,phone,birthday) VALUES(?,?,?,?,?)";
        let insert_id = sqlx::query(sql)
            .bind(&new_person.f_name)
            .bind(&new_person.l_name)
            .bind(&new_person.email)
            .bind(&new_person.phone)
            .bind(new_person.birthday)
            .execute(&connection.pool)
            .await?
            .last_insert_id() as i64;

        Ok(insert_id)
    }

    /// inserts a person record into the database as part of a transaction
    pub async fn into_db_as_transaction(new_person: &NewPerson, tx: &mut Transaction<'_,MySql>) -> Result<i64> {
        let sql = "INSERT INTO `person` (f_name,l_name,email,phone,birthday) VALUES(?,?,?,?,?)";
        let insert_id = sqlx::query(sql)
            .bind(&new_person.f_name)
            .bind(&new_person.l_name)
            .bind(&new_person.email)
            .bind(&new_person.phone)
            .bind(new_person.birthday)
            .execute(&mut **tx)
            .await?
            .last_insert_id() as i64;

        Ok(insert_id)
    }
}

#[derive(Clone,Debug)]
/// helper for building a Person object prior to inserting into the database and aquiring a person.id
pub struct NewPerson {
    f_name: Option<String>,
    l_name: Option<String>,
    email: String,              // required
    phone: Option<String>,
    birthday: Option<NaiveDate>
}

impl NewPerson {
    /// inserts a person into the database and returns a Person object
    pub async fn into_db(self,connection:&DatabaseConnection) -> Result<Person> {
        let id = Person::into_db(&self, connection).await?;
        
        Ok(Self::transform(self,id))
    }

    /// inserts a person into the database as a transaction and returns a Person object
    pub async fn into_db_as_transaction(self,tx:&mut Transaction<'_,MySql>) -> Result<Person> {
        let id = Person::into_db_as_transaction(&self, tx).await?;
        
        Ok(Self::transform(self,id))
    }

    // transforms the NewPerson into a Person after being assigned a person.id by the database
    fn transform(self, id:i64) -> Person {
        Person {
            id,
            f_name:self.f_name,
            l_name:self.l_name,
            email:self.email,
            phone:self.phone,
            birthday:self.birthday
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use database::types::DatabaseConnection;

    /// transaction constructor build test
    #[actix_rt::test]
    async fn create_new_person() {
        let year:i32 = 1980;
        let month:u32 = 6;
        let day:u32 = 5;

        let connection = DatabaseConnection::new().await.unwrap();
        let mut tx = connection.pool.begin().await.unwrap();

        let manual_person = NewPerson {
            f_name: Some(String::from("f_name")),
            l_name: Some(String::from("l_name")),
            email:  String::from("email"),
            phone:  Some(String::from("phone")),
            birthday: Some(NaiveDate::from_ymd_opt(year,month,day).unwrap()),
        };
        let cloned = manual_person.clone();
        
        let person = manual_person
            .into_db_as_transaction(&mut tx)
            .await
            .unwrap();

        assert!(person.id > 0);
        assert_eq!(person.f_name,cloned.f_name);
        assert_eq!(person.l_name,cloned.l_name);
        assert_eq!(person.email,cloned.email);
        assert_eq!(person.phone,cloned.phone);

        tx.rollback().await.unwrap();
    }

    /// get person by email
    #[actix_rt::test]
    async fn get_person_by_email() {
        let year:i32 = 1980;
        let month:u32 = 6;
        let day:u32 = 5;

        let connection = DatabaseConnection::new().await.unwrap();

        let manual_person = NewPerson {
            f_name: Some(String::from("test")),
            l_name: Some(String::from("test")),
            email:  String::from("test@test.com"),
            phone:  Some(String::from("test")),
            birthday: Some(NaiveDate::from_ymd_opt(year,month,day).unwrap()),
        };
        
        let person = Person::by_email(&manual_person.email, &connection).await.unwrap().unwrap();

        assert_eq!(person.id, 2);
        assert_eq!(person.f_name,manual_person.f_name);
        assert_eq!(person.l_name,manual_person.l_name);
        assert_eq!(person.email,manual_person.email);
        assert_eq!(person.phone,manual_person.phone);
    }

    /// get person by id
    #[actix_rt::test]
    async fn get_person_by_id() {
        let year:i32 = 1980;
        let month:u32 = 6;
        let day:u32 = 5;

        let connection = DatabaseConnection::new().await.unwrap();

        let manual_person = NewPerson {
            f_name: Some(String::from("test")),
            l_name: Some(String::from("test")),
            email:  String::from("test@test.com"),
            phone:  Some(String::from("test")),
            birthday: Some(NaiveDate::from_ymd_opt(year,month,day).unwrap()),
        };
        
        let person = Person::by_id(2, &connection).await.unwrap();

        assert_eq!(person.id, 2);
        assert_eq!(person.f_name,manual_person.f_name);
        assert_eq!(person.l_name,manual_person.l_name);
        assert_eq!(person.email,manual_person.email);
        assert_eq!(person.phone,manual_person.phone);
    }
}