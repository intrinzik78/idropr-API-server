use actix_web::web::Data;
use database::types::DatabaseConnection;

use crate::{
    enums::sessions::SessionControllerStatus,
    types::AppState
};

pub struct UserEpochSync;

impl UserEpochSync {
    pub async fn run(arc_state: &Data<AppState>) {
        let app_state = arc_state.clone();
        let connection = DatabaseConnection::new()
            .await
            .expect("could not construct database connection for UserEpochController");

        let _epoch_controller = actix_web::rt::spawn(async move {
            match app_state.sessions() {
                SessionControllerStatus::Enabled(controller) => controller.watch_epoch(&connection).await,
                SessionControllerStatus::Disabled => {}
            }
        });
    }
}