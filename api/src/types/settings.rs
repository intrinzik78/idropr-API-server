use chrono::{DateTime,Utc};
use sqlx::FromRow;

use crate::{
    enums::{
        Error,
        MasterPassword,
        ServerMode,
        SystemFlag
    }, 
    traits::{
        ToServerMode,
        ToSystemFlag
    },
    types::{
        DatabaseConnection, Env
    }
};

type Result<T> = std::result::Result<T,Error>;

/// system settings container
#[derive(Clone,Debug)]
pub struct Settings {
    pub load_email_queue_service: SystemFlag,
    pub postmark_email_service: SystemFlag,
    pub load_rate_limiter_service: SystemFlag,
    pub load_text_queue_service: SystemFlag,
    pub master_password: MasterPassword,
    pub ip_address: String,
    pub server_mode: ServerMode,
    pub server_port: u16,
    pub timestamp: DateTime<Utc>
}

/// database record transformer
#[derive(Debug,FromRow)]
struct SettingsHelper {
    pub load_email_queue_service: i8,
    pub postmark_email_service: i8,
    pub load_rate_limiter_service: i8,
    pub load_text_queue_service: i8,
    pub ip_address: String,
    pub server_mode: i8,
    pub server_port: u16,
    pub timestamp: DateTime<Utc>
}

impl SettingsHelper {
    /// transforms the raw database record into a Settings struct
    fn transform(&self) -> Result<Settings> {
        let settings = Settings {
            load_email_queue_service: self.load_email_queue_service.to_system_flag()?,
            postmark_email_service: self.postmark_email_service.to_system_flag()?,
            load_rate_limiter_service: self.load_rate_limiter_service.to_system_flag()?,
            load_text_queue_service: self.load_text_queue_service.to_system_flag()?,
            master_password: MasterPassword::None,
            ip_address: self.ip_address.clone(),
            server_mode: self.server_mode.to_server_mode()?,
            server_port: self.server_port,
            timestamp: self.timestamp,
        };

        Ok(settings)
    }
}

impl Settings {
    /// settings record by id
    pub async fn by_id(id: i64, database: &DatabaseConnection) -> Result<Settings> {
        let sql = "SELECT load_email_queue_service, postmark_email_service, load_rate_limiter_service, load_text_queue_service, ip_address, server_mode, server_port, timestamp FROM `system_settings` WHERE system_settings.id = ?";
        let helper:Option<SettingsHelper> = sqlx::query_as(sql)
            .bind(id)
            .fetch_optional(&database.pool)
            .await?;

        if let Some(settings) = helper {
            let result = settings.transform()?;
            Ok(result)
        } else {
            Err(Error::SystemSettingsRecordNotReturned)
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        let now = Utc::now();
        let env = Env::default();
        let ip_address = env.ip_address;
        let password = env.master_password;
        let server_port = env.server_port;
        let master_password = MasterPassword::Some(password);

        Settings {
            load_email_queue_service: SystemFlag::Disabled,
            postmark_email_service: SystemFlag::Disabled,
            load_rate_limiter_service: SystemFlag::Disabled,
            load_text_queue_service: SystemFlag::Disabled,
            master_password,
            ip_address,
            server_mode: ServerMode::Maintenance,
            server_port,
            timestamp: now
        }
    }        
}

#[cfg(test)]
mod tests {
    use super::*;

    // tests transforming of database helper struct SettingsHelper into Settings
    #[actix_rt::test]
    async fn database_helper_transform() {
        let now = chrono::Utc::now();

        let test_data = SettingsHelper {
            load_email_queue_service: 1,
            postmark_email_service: 1,
            load_rate_limiter_service: 1,
            load_text_queue_service: 1,
            ip_address: String::from("ip_address"),
            server_mode: 1,
            server_port: 1,
            timestamp: now
        };

        let transformed_data = test_data.transform().unwrap();

        assert_eq!(transformed_data.load_email_queue_service, SystemFlag::Enabled);
        assert_eq!(transformed_data.postmark_email_service, SystemFlag::Enabled);
        assert_eq!(transformed_data.load_rate_limiter_service, SystemFlag::Enabled);
        assert_eq!(transformed_data.load_text_queue_service, SystemFlag::Enabled);
        assert_eq!(transformed_data.ip_address, String::from("ip_address"));
        assert_eq!(transformed_data.server_port, 1);
        assert_eq!(transformed_data.timestamp, now);
        
    }
}