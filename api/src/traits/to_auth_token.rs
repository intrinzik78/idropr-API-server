use actix_web::HttpRequest;
use crate::enums::Error;
pub trait  ToHeaderAuthToken {
    fn to_auth(&self) -> Result<String,Error>;
}

impl ToHeaderAuthToken for HttpRequest {
    fn to_auth(&self) -> Result<String,Error> {
        let (_,token) = self.headers()
            .get("Authorization")
            .ok_or(Error::MissingAuthorizationBearerInHeader)?
            .to_str()
            .map_err(|_e| Error::MalformedAuthorizationToken)?
            .rsplit_once(" ")
            .ok_or(Error::MalformedAuthorizationToken)?;

        Ok(token.to_owned())
    }
}