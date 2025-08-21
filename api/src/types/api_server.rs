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
    services::{RateLimitMiddleware},
    types::{
        AppState,
        HeaderSettings,
        RouteCollection
    }
};

pub struct ApiServer;

impl ApiServer {
    pub async fn run(command: PrimaryCommand, arc_state: Data<AppState>, collection: RouteCollection) -> Result<()> {
        let app_state = arc_state.clone();
        let ip_address = app_state.settings().ip_address.clone();
        let open_port = app_state.settings().server_port;

        // build app
        let app = move || {
            // load cross site scripting rules
            let cors = match command {
                PrimaryCommand::Dev => HeaderSettings::dev_cors(),  // all requests accepted
                PrimaryCommand::Prod => HeaderSettings::prod_cors() // production headers and limited origins accepted
            };

            // build route service collections
            let routes_v1 = collection.v1();

            // load services into app
            actix_web::App::new()
                .app_data(app_state.clone())
                .wrap(RateLimitMiddleware)
                .wrap(cors)
                .service(routes_v1)
        };

        // start server
        HttpServer::new(app)
            .bind((ip_address,open_port))
            .expect("Failed to generate a running server.")
            .workers(2)
            .run()
            .await
            .map_err(|e| Error::ServerCrash(e.to_string()))
    }
}