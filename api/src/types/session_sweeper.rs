use actix_web::web::Data;

use crate::{
    enums::SessionControllerStatus,
    types::AppState
};

pub struct SessionSweeper;

impl SessionSweeper {
    pub async fn run(arc_state: &Data<AppState>) {
        let app_state = arc_state.clone();

        let _garbage_collector = actix_web::rt::spawn(async move {
            match app_state.sessions() {
                SessionControllerStatus::Enabled(controller) => controller.watch().await,
                SessionControllerStatus::Disabled => {}
            }
        });
    }
}