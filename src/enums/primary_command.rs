/// simple command line arguments to load the correct environment vars
use clap::Subcommand;

use crate::{
    enums::Error,
    types::AppState
};

type Result<T> = std::result::Result<T,Error>;

#[derive(Clone,Debug,Subcommand)]
pub enum PrimaryCommand {
    Dev,      // loads settings for use on a localhost
    Prod      // loads settings for public host
}

impl PrimaryCommand {
    pub async fn dev_state() -> Result<AppState> {
        println!("\nwarning: running in dev mode\n");
        let app_state = AppState::new()
            .await?;

        Ok(app_state)
    }

    pub async fn prod_state() -> Result<AppState> {
        let app_state = AppState::new()
            .await?
            .with_database_settings()
            .await?;

        Ok(app_state)
    }
}