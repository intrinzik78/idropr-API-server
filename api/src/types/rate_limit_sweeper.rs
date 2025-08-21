use actix_web::web::Data;

use crate::{
    enums::RateLimiterStatus,
    types::AppState
};

pub struct RateLimitSweeper;

impl RateLimitSweeper {
    pub async fn run(arc_state: &Data<AppState>) {
        let app_state = arc_state.clone();

        let _garbage_collector = actix_web::rt::spawn(async move {
            match app_state.rate_limiter() {
                RateLimiterStatus::Enabled(limiter) => limiter.watch().await,
                RateLimiterStatus::Disabled => {}
            }
        });
    }
}