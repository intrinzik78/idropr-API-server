use chrono::{ DateTime, Utc };
use sqlx::FromRow;

use database::types::DatabaseConnection;
use crate::enums::Error;

type Result<T> = std::result::Result<T,Error>;

#[derive(Debug,Clone,FromRow)]
pub struct Email {
    id: i64,
    name: String,                       // admin facing
    description: String,                // admin facing
    subject: String,                    // short and catchy, 60 characters or less
    from_address: String,               // sending email address
    reply_to_address: String,           // reply-to email address
    created_at: DateTime<Utc>,          // date created
    updated_at: Option<DateTime<Utc>>   // last update
}

//sync
impl Email {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str{
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn from_address(&self) -> &str {
        &self.from_address
    }

    pub fn reply_to_address(&self) -> &str {
        &self.reply_to_address
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_at(&self) -> Option<&DateTime<Utc>> {
        if let Some(_opt) = self.updated_at {
            self.updated_at.as_ref()
        } else {
            None
        }
    }
}

//async
impl Email {

    /// Retrieves the entire email sequence assigned to a campaign
    pub async fn by_id(id: u64, database: &DatabaseConnection) -> Result<Email> {
        let sql = "SELECT id, name, description, subject, from_address, reply_to_address, created_at, updated_at FROM `email` WHERE id = ?";
        let result:Email = sqlx::query_as(sql)
            .bind(id)
            .fetch_one(&database.pool)
            .await?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn get_by_id() {
        let database_connection: DatabaseConnection = DatabaseConnection::new()
            .await
            .expect("Failed to create database connection pool.");

        let email = Email::by_id(1, &database_connection).await;

        assert!(email.is_ok());
    }
}