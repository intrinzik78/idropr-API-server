// export type system
pub mod api;
pub mod enums;
pub mod services;
pub mod traits;
pub mod types;

// import packages
use clap::Parser;
use crate::types::{GarbageCollectorInterval, RouteCollection};

// internal types
use {
    enums::{
        Error,
        PrimaryCommand
    },
    types::{
        ApiServer,
        Cli,
        Env
    }
};

type Result<T> = std::result::Result<T,Error>;

#[actix_rt::main]
async fn main() -> Result<()> {
    // load env vars
    let env = Env::default();

    // command line parser
    let run_command = Cli::parse().command;

    // load base settings
    let initial_state = match run_command {
        PrimaryCommand::Dev => PrimaryCommand::dev_state(&env).await?,   // load local dev settings
        PrimaryCommand::Prod => PrimaryCommand::prod_state(&env).await?  // load production server settings
    };

    // move state into ARC ref
    let arc_state = actix_web::web::Data::new(initial_state);

    // add chron jobs / services here ↴
    let collection = RouteCollection;

    let () = GarbageCollectorInterval::run(&arc_state).await;

    // build and run server ↴
    let server = ApiServer::run(run_command, arc_state, collection);

    server.await // win
}