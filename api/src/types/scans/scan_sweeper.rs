use actix_web::web::Data;
use database::types::DatabaseConnection;

use crate::types::AppState;

pub struct ScanSweeper;

impl ScanSweeper {
    pub async fn run(arc_state: &Data<AppState>) {
        let app_state = arc_state.clone();
        let connection = DatabaseConnection::new()
            .await
            .expect("could not construct database connection for UserEpochController");

        let _controller = actix_web::rt::spawn(async move {
            let scanner = app_state.scanner();
            scanner.watch(connection).await
        });
    }
}