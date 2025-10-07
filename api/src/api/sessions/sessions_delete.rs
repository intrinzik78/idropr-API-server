use actix_web::{web,HttpRequest,Responder};

use crate::{
    enums::sessions::SessionControllerStatus,
    traits::ToHeaderAuthToken,
    types::{permissions::WereChecked, ApiResponse, AppState}
};

#[derive(Debug)]
pub struct SessionsDelete;

impl SessionsDelete {
    pub async fn logic(_permissions: WereChecked, req: HttpRequest, shared: web::Data<AppState>) -> impl Responder {

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
            Ok(()) => ApiResponse::no_content(),
            Err(_) => ApiResponse::server_error().error()
        }
    }
}