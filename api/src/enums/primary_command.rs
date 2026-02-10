/// simple command line arguments to load the correct environment vars
use clap::Subcommand;
use database::types::DatabaseConnection;
use rate_limit::{
    enums::{TimeWindow,RefillRate},
    types::RateLimitBuilder
};
use tokio::fs;

use crate::{
    enums::{Error, RateLimiterStatus, SystemFlag, sessions::SessionControllerStatus},
    types::{AppState,Env, sessions::{SessionController, UserEpochController}}
};

type Result<T> = std::result::Result<T,Error>;

#[derive(Clone,Debug,Subcommand)]
pub enum PrimaryCommand {
    Dev,      // loads settings for use on a localhost
    Prod      // loads settings for public host
}

impl PrimaryCommand {

    /// builds the rate limiter
    fn build_rate_limiter(env: &Env) -> RateLimiterStatus {
        // settings
        let threads = env.server_threads;
        let rate = env.limiter_refill_rate;
        let refill_rate = match env.limiter_refill_window {
            TimeWindow::Day => RefillRate::PerDay(rate),
            TimeWindow::Hour => RefillRate::PerHour(rate),
            TimeWindow::Minute => RefillRate::PerMinute(rate),
            TimeWindow::Second => RefillRate::PerSecond(rate)
        };

        // initialize
        let limiter = RateLimitBuilder::default()
            .with_initial_capacity(env.limiter_initial_capacity)
            .with_tokens_per_bucket(env.limiter_tokens_per_bucket)
            .with_refill_rate(refill_rate)
            .shard_into(threads)
            .build();

        RateLimiterStatus::Enabled(Box::new(limiter))
    }

    /// builds the session controller
    async fn build_session_controller(env: &Env, connection: &DatabaseConnection) -> Result<SessionControllerStatus> {
        
        // pull and set the current user epoch from the database
        let capacity = env.sessions_initial_capacity;
        let threads = env.server_threads;
        let epoch_controller = UserEpochController::new(connection).await?;
        let session_controller = SessionController::new(capacity,threads,epoch_controller);

        Ok(SessionControllerStatus::Enabled(Box::new(session_controller)))
    }

    /// loads settings for local developement
    pub async fn dev_state(env: &Env) -> Result<AppState> {

        println!("\nwarning: server running in dev mode\n");

        // create required directories
        fs::create_dir_all(&env.extractor_final_dir).await?;
        fs::create_dir_all(&env.extractor_temp_dir).await?;

        let connection = DatabaseConnection::new().await?;

        let limiter = PrimaryCommand::build_rate_limiter(env);
        let sessions = PrimaryCommand::build_session_controller(env,&connection).await?;

        // initialize app state
        let app_state = AppState::new()
            .await?
            .with_rate_limit_status(limiter)
            .with_session_status(sessions);

        Ok(app_state)
    }

    /// loads settings for production environment
    pub async fn prod_state(env: &Env) -> Result<AppState> {

        println!("\nserver running in production mode\n");

        // create required directories
        fs::create_dir_all(&env.extractor_final_dir).await?;
        fs::create_dir_all(&env.extractor_temp_dir).await?;

        let connection = DatabaseConnection::new().await?;
        
        let app_state = AppState::new()
            .await?
            .with_database_settings()
            .await?;
        
        // check flag and load limiter if enabled
        let app_state = match app_state.settings().load_rate_limiter_service {
            SystemFlag::Enabled => {
                let limiter = PrimaryCommand::build_rate_limiter(env);
                let sessions = PrimaryCommand::build_session_controller(env,&connection).await?;
                app_state
                    .with_rate_limit_status(limiter)
                    .with_session_status(sessions)
            },
            SystemFlag::Disabled => app_state
        };

        Ok(app_state)
    }
}