// export type system
pub mod api;
pub mod enums;
pub mod services;
pub mod traits;
pub mod types;

// external
use actix_web::HttpServer;
use clap::Parser;

// internal
use {
    enums::{
        Error,
        PrimaryCommand
    },
    types::{
        Cli,
        HeaderSettings,
        RouteScope
    }
};

type Result<T> = std::result::Result<T,Error>;

#[actix_rt::main]
async fn main() -> Result<()> {
    
    // command line options for loading settings
    let cli = Cli::parse();
    let initial_state = match cli.command {
        PrimaryCommand::Dev => PrimaryCommand::dev_state().await?,   // local host settings
        PrimaryCommand::Prod => PrimaryCommand::prod_state().await?  // production host settings
    };

    // move app data behind Arc
    let app_state = actix_web::web::Data::new(initial_state);
    let ip_address = app_state.settings().ip_address.clone();

    let app = move || {
        // set cors policy
        let cors = match cli.command {
            PrimaryCommand::Dev => HeaderSettings::dev_cors(),
            PrimaryCommand::Prod => HeaderSettings::prod_cors()
        };

        // build route services
        let public_scope = RouteScope::public();

        // load services into app
        actix_web::App::new()
            .app_data(app_state.clone())
            .wrap(cors)
            .service(public_scope)
    };

    // start server
    HttpServer::new(app)
        .bind(&ip_address)
        .expect("Failed to generate a running server.")
        .run()
        .await
        .map_err(|_e| Error::ServerCrash)

}
