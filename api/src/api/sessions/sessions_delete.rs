use actix_web::{web,HttpRequest,Responder};

use crate::{
    enums::{ApiResult,sessions::SessionControllerStatus},
    traits::ToHeaderAuthToken,
    types::{permissions::WereChecked, AppState}
};

#[derive(Debug)]
pub struct SessionsDelete;

impl SessionsDelete {
    pub async fn logic(_permissions: WereChecked, req: HttpRequest, shared: web::Data<AppState>) -> impl Responder {

        // extract token
        let token = match req.to_auth() {
            Ok(t) => t,
            Err(_e) => return ApiResult::unauthorized().to_http()
        };

        // session controller reference
        let session_controller = match shared.sessions() {
            SessionControllerStatus::Enabled(controller) => controller,
            SessionControllerStatus::Disabled => return ApiResult::server_error().to_http()
        };

        // delete session
        match session_controller.delete(&token) {
            Ok(()) => ApiResult::no_content().to_http(),
            Err(_) => ApiResult::server_error().to_http()
        }
    }
}