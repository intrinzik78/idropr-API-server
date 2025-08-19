use actix_web::{web,HttpRequest,Responder};

use crate::{
    enums::SessionControllerStatus,
    traits::ToHeaderAuthToken,
    types::{ApiResponse, AppState}
};

#[derive(Debug)]
pub struct SessionsDelete;

impl SessionsDelete {
    pub async fn logic(req: HttpRequest, shared: web::Data<AppState>) -> impl Responder {

        // extract token
        let token = match req.to_auth() {
            Ok(t) => t,
            Err(_e) => return ApiResponse::unauthorized().ok()
        };

        // session controller reference
        let session_controller = match shared.sessions() {
            SessionControllerStatus::Enabled(controller) => controller,
            SessionControllerStatus::Disabled => return ApiResponse::server_error().error()
        };

        // delete session
        match session_controller.delete(&token) {
            Ok(_) => ApiResponse::success(),
            Err(_) => ApiResponse::server_error().error()
        }
    }
}