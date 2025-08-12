/// simple command line arguments to load the correct environment vars
use clap::Subcommand;
use rate_limit::{
    enums::{TimeWindow,RefillRate},
    types::RateLimitBuilder
};

use crate::{
    enums::{Error, RateLimiterStatus, SystemFlag},
    types::{AppState,Env}
};

type Result<T> = std::result::Result<T,Error>;

#[derive(Clone,Debug,Subcommand)]
pub enum PrimaryCommand {
    Dev,      // loads settings for use on a localhost
    Prod      // loads settings for public host
}

impl PrimaryCommand {

    fn build_refill_rate(rate: f32, window: TimeWindow) -> RefillRate {
        match window {
            TimeWindow::Day => RefillRate::PerDay(rate),
            TimeWindow::Hour => RefillRate::PerHour(rate),
            TimeWindow::Minute => RefillRate::PerMinute(rate),
            TimeWindow::Second => RefillRate::PerSecond(rate)
        }
    }

    /// loads settings for local developement
    pub async fn dev_state(env: &Env) -> Result<AppState> {

        println!("\nwarning: server running in dev mode\n");
        
        let threads = env.server_threads();
        // initialize rate limiter
        let rate = env.limiter_refill_rate();
        let window = env.limiter_refill_window();
        let refill_rate = PrimaryCommand::build_refill_rate(rate, window);
        let limiter = RateLimitBuilder::default()
            .with_initial_capacity(env.limiter_initial_capacity())
            .with_tokens_per_bucket(env.limiter_tokens_per_bucket())
            .with_refill_rate(refill_rate)
            .shard_into(threads)
            .build();

        let rate_limit_status = RateLimiterStatus::Enabled(Box::new(limiter));
        
        // initialize app state
        let app_state = AppState::new(env)
            .await?
            .with_rate_limit_status(rate_limit_status);

        Ok(app_state)
    }

    pub async fn prod_state(env: &Env) -> Result<AppState> {

        println!("\nserver running in production mode\n");
        
        let app_state = AppState::new(env)
            .await?
            .with_database_settings()
            .await?;
        
        // check flag and load limiter if enabled
        let app_state = match app_state.settings().load_rate_limiter_service {
            SystemFlag::Enabled => {
                let threads = env.server_threads();

                // initialize rate limiter
                let rate = env.limiter_refill_rate();
                let window = env.limiter_refill_window();
                let refill_rate = PrimaryCommand::build_refill_rate(rate, window);
                let limiter = RateLimitBuilder::default()
                    .with_initial_capacity(env.limiter_initial_capacity())
                    .with_tokens_per_bucket(env.limiter_tokens_per_bucket())
                    .with_refill_rate(refill_rate)
                    .shard_into(threads)
                    .build();
                
                let new_status = RateLimiterStatus::Enabled(Box::new(limiter));
                
                app_state.with_rate_limit_status(new_status)
            },
            SystemFlag::Disabled => app_state
        };

        Ok(app_state)
    }
}