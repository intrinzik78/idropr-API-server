use chrono::{ DateTime, Utc };
use sqlx::FromRow;

use database::types::DatabaseConnection;
use crate::enums::Error;

type Result<T> = std::result::Result<T,Error>;

#[derive(Debug,Clone,FromRow)]
pub struct CampaignEmail {
    id: i64,
    campaign_id: i64,
    name: String,                       // admin facing
    description: String,                // admin facing
    subject: String,                    // short and catchy, 60 characters or less
    html_body: String,                  // styled body
    text_body: String,                  // unstyled body
    day_to_send: i64,                   // from signup / to event date based on DripDriction
    from_address: String,               // sending email address
    reply_to_address: String,           // reply-to email address
    created_at: DateTime<Utc>,          // date created
    updated_at: Option<DateTime<Utc>>   // last update
}

//sync
impl CampaignEmail {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn campaign_id(&self) -> i64 {
        self.campaign_id
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

    pub fn html_body(&self) -> &str {
        &self.html_body
    }

    pub fn text_body(&self) -> &str {
        &self.text_body
    }

    pub fn day_to_send(&self) -> i64 {
        self.day_to_send
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
impl CampaignEmail {

    /// Retrieves the entire email sequence assigned to a campaign
    pub async fn list_by_campaign_id(id: i64, database: &DatabaseConnection) -> Result<Vec<CampaignEmail>> {
        let sql = "SELECT campaign_email.id, campaign_email.campaign_id, campaign_email.name, campaign_email.description, campaign_email.subject, campaign_email.html_body, campaign_email.text_body, campaign_email.day_to_send, campaign_email.from_address, campaign_email.reply_to_address, campaign_email.created_at, campaign_email.updated_at FROM `campaign_email` WHERE campaign_email.campaign_id = ? LIMIT 1000";
        let results:Vec<CampaignEmail> = sqlx::query_as(sql)
            .bind(id)
            .fetch_all(&database.pool)
            .await?;

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn list_by_campaign_id() {
        let database_connection: DatabaseConnection = DatabaseConnection::new()
            .await
            .expect("Failed to create database connection pool.");

        let list = CampaignEmail::list_by_campaign_id(1, &database_connection).await;

        assert!(list.is_ok());
    }

    #[test]
    fn  build_campaign_email() {
        let now = Utc::now();
        let test_obj = CampaignEmail {
            id: 0,
            campaign_id: 1,
            name: String::from("name"),
            description: String::from("description"),
            subject: String::from("subject"),
            html_body: String::from("html_body"),
            text_body: String::from("text_body"),
            day_to_send: i64::max_value(),           
            from_address: String::from("from_address"),
            reply_to_address: String::from("reply_to_address"),
            created_at: now.clone(),
            updated_at: Some(now)
        };

        assert_eq!(test_obj.id(),0);
        assert_eq!(test_obj.campaign_id(),1);
        assert_eq!(test_obj.name(),String::from("name"));
        assert_eq!(test_obj.description(),String::from("description"));
        assert_eq!(test_obj.subject(),String::from("subject"));
        assert_eq!(test_obj.html_body(),String::from("html_body"));
        assert_eq!(test_obj.text_body(),String::from("text_body"));
        assert_eq!(test_obj.day_to_send(),i64::max_value());
        assert_eq!(test_obj.from_address(),String::from("from_address"));
        assert_eq!(test_obj.reply_to_address(),String::from("reply_to_address"));
        assert_eq!(test_obj.created_at(),&now);
        assert_eq!(test_obj.updated_at().unwrap(),&now);
    }

}