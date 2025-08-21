use actix_web::dev::ServiceRequest;

use crate::enums::Error;

type Result<T> = std::result::Result<T,Error>;

pub struct AuthorizationToken;

impl AuthorizationToken {
    
    pub fn extract(req: &ServiceRequest) -> Result<String> {
        let str = req
            .headers()
            .get("Authorization")
            .ok_or(Error::MissingAuthorizationBearerInHeader)?
            .to_str()
            .map_err(|_e| Error::MalformedAuthorizationToken)?;
    
        let token = {
            let str = AuthorizationToken::parse_token(str)
                .map_err(|_e| Error::MalformedAuthorizationToken)?;
            
            str.to_string()
        };
    
        Ok(token)
    }
    
    fn parse_token(token: &str) -> Result<&str> {
        let token = token
            .rsplit_once(" ")
            .ok_or(Error::MalformedAuthorizationToken)?
            .1; // takes the 2nd index of the tuple from rsplit_once
    
        Ok(token)
    }
}