use actix_web::{
    web::Data,
    HttpServer
};

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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
        RouteCollection,
        open_api_doc::ApiDoc
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
                .route("/api-docs/openapi.json", actix_web::web::get().to(|| async {
                    actix_web::HttpResponse::Ok()
                        .content_type("application/json")
                        .body(ApiDoc::openapi().to_pretty_json().unwrap())
                }))
                // YAML spec
                .route("/api-docs/openapi.yaml", actix_web::web::get().to(|| async {
                    actix_web::HttpResponse::Ok()
                        .content_type("application/yaml")
                        .body(ApiDoc::openapi().to_yaml().unwrap())
                }))
                // Swagger UI
                .service(
                    SwaggerUi::new("/swagger-ui/{_:.*}")
                        .url("/api-docs/openapi.json", ApiDoc::openapi())
                )
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