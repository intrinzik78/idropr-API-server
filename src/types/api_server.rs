use actix_web::{
    web::Data,
    HttpServer
};

type Result<T> = std::result::Result<T,Error>;

use crate::{
    enums::{
        Error,
        PrimaryCommand
    },
    types::{
        AppState,
        HeaderSettings,
        RouteScope
    }
};

pub struct ApiServer;

impl ApiServer {
    pub async fn new(command: PrimaryCommand, arc_state: Data<AppState>) -> Result<()> {
        let app_state = arc_state.clone();
        let ip_address = app_state.settings().ip_address.clone();

        // build app
        let app = move || {
            // load cross site scripting rules
            let cors = match command {
                PrimaryCommand::Dev => HeaderSettings::dev_cors(),  // all requests accepted
                PrimaryCommand::Prod => HeaderSettings::prod_cors() // production headers and limited origins accepted
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
            .bind(ip_address)
            .expect("Failed to generate a running server.")
            .run()
            .await
            .map_err(|e| Error::ServerCrash(e.to_string()))
    }
}