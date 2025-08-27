use actix_web::{dev::Payload, error::{ErrorForbidden}, Error, FromRequest, HttpMessage, HttpRequest};
use futures::future::ready;

use std::rc::Rc;
use crate::types::UserPermissions;

#[derive(Clone,Debug)]
pub struct NeedCheck(pub Rc<UserPermissions>);

pub struct WereChecked(pub Rc<UserPermissions>);

impl FromRequest for WereChecked {
    type Error = Error;
    type Future = futures::future::Ready<std::result::Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let checked = req.extensions().get::<NeedCheck>().cloned();

        match checked {
            Some(NeedCheck(p)) => ready(Ok(WereChecked(p))),
            None => ready(Err(ErrorForbidden("insufficient permissions")))
        }
    }
}