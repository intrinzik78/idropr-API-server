use crate::{
    enums::{
        ConnectionStatus,
        Error,
        RateLimiterStatus,
    },
    types::{
        DatabaseConnection,
        Env,
        Settings
    }
};

type Result<T> = std::result::Result<T,Error>;

const DATABASE_SETTINGS_ID:i64 = 1;

#[derive(Debug)]
pub struct AppState {
    database: DatabaseConnection,
    settings: Settings,
    limiter: RateLimiterStatus
}

impl AppState {

    /// constructor
    pub async fn new(env: &Env) -> Result<AppState> {
        // system settings
        let settings = Settings::default();

        // connect database
        let database = DatabaseConnection::new(env).await?;
        
        // test connection status
        match database.connection_status().await {
            ConnectionStatus::Connected => {},
            ConnectionStatus::Disconnected => return Err(Error::DatabaseConnectionTestFailed)
        }

        // construct app state
        let app_state = AppState {
            database,
            settings,
            limiter: RateLimiterStatus::Disabled
        };

        Ok(app_state)
    }

    // sync system settings with the database
    pub async fn with_database_settings(mut self) -> Result<Self> {
        let settings = Settings::by_id(DATABASE_SETTINGS_ID, &self.database).await?;

        // apply database setting overrides
        self.settings.load_email_queue_service = settings.load_email_queue_service;
        self.settings.postmark_email_service = settings.postmark_email_service;
        self.settings.load_rate_limiter_service = settings.load_rate_limiter_service;
        self.settings.load_text_queue_service = settings.load_text_queue_service;
        self.settings.ip_address = settings.ip_address;
        self.settings.server_mode = settings.server_mode;
        self.settings.timestamp = settings.timestamp;

        Ok(self)
    }

    /// database getter
    pub fn database(&self) -> &DatabaseConnection {
        &self.database
    }

    /// rate limiter getter
    pub fn rate_limiter(&self) -> &RateLimiterStatus {
        &self.limiter
    }

    /// accepts an instance of RateLimiter and moves it into the server
    pub fn with_rate_limit_status(mut self, status: RateLimiterStatus) -> Self {
        self.limiter = status;

        self
    }

    /// settings getter
    pub fn settings(&self) -> &Settings {
        &self.settings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{
        ServerMode,
        SystemFlag
    };

    /// constructor build test
    #[actix_rt::test]
    async fn app_state_builder() {
        // constructor build test
        let env = Env::default();
        let _constructor_test: AppState = AppState::new(&env).await.unwrap();

        let env_vars = Env::default();
        let server_port = env_vars.server_port();
        let database = DatabaseConnection::new(&env_vars).await.expect("failed to build database connection in app state test");
        let master_password = crate::enums::MasterPassword::None;

        let settings = Settings {
            load_email_queue_service: SystemFlag::Disabled,
            postmark_email_service: SystemFlag::Disabled,
            load_rate_limiter_service: SystemFlag::Disabled,
            load_text_queue_service: SystemFlag::Disabled,
            master_password,
            ip_address: String::from("ip_address"),
            server_mode: ServerMode::Maintenance,
            server_port,
            timestamp: chrono::Utc::now()
        };

        // manual build test
        let _manual_builder = AppState {
            database,
            settings,
            limiter: RateLimiterStatus::Disabled
        };

        // connection status is already checked in the AppState constructor()
        // assert_eq!(test.database.connection_status().await, ConnectionStatus::Connected);
    }
}