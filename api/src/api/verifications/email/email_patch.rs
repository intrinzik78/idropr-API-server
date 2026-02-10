use actix_web::{Responder,web::{Data,Path}};
use database::types::DatabaseConnection;
use serde::Deserialize;

use crate::{
    enums::{ApiResult, Error, ExpiredStatus, RowsUpdated, VerificationStatus},
    traits::{ToHash,ToVerificationStatus},
    types::{AppState, email::EmailVerification}
};

type Result<T> = std::result::Result<T,Error>;

#[derive(Deserialize)]
pub struct PatchReqPath {
    pub id:i64,
    pub uuid:String
}

pub struct PatchEmailVerification;

impl PatchEmailVerification {

    pub async fn logic(id: i64, uuid: &str, database: &DatabaseConnection) -> Result<()> {
        // retrieve record by id
        let verification_opt = EmailVerification::by_id(id, database).await?;

        // if found, return a verification or error
        if let Some(email_verification) = verification_opt {

            // check if link already verified
            if email_verification.is_verified() == VerificationStatus::Verified {
                return Err(Error::EmailAlreadyVerified)
            }

            // check if link expired
            if email_verification.expired() == ExpiredStatus::Expired {
                println!("expired");
                return Err(Error::EmailVerificationExpired)
            }

            // convert post::uuid to blake3::Hash
            let uuid_hash = str::from_utf8(uuid.as_bytes())?.to_hash()?;

            // convert db::bytes to blake3::Hash
            let mut buf:[u8;32] = [0;32];
            buf[..email_verification.hash().len()].copy_from_slice(email_verification.hash());
            let db_hash = blake3::Hash::from_bytes(buf);

            // run verification
            if (uuid_hash == db_hash).to_verification_status() == VerificationStatus::Unverified{
                return Err(Error::VerificationHashCheckFailed)
            }

            match EmailVerification::verify(id, database).await? {
                RowsUpdated::None => Err(Error::TooFewRowsUpdated),
                RowsUpdated::Some(1) => Ok(()),
                RowsUpdated::Some(_) => Err(Error::TooManyRowsUpdated)
            }
            
        } else {
            Err(Error::VerificationEmailNotFound)
        }
    }

    pub async fn response(path: Path<PatchReqPath>, shared: Data<AppState>) -> impl Responder {
        type E = Error;

        let PatchReqPath { id, uuid } = path.into_inner();
        let database = shared.database();
       
        match Self::logic(id, &uuid, database).await {
            Ok(()) => ApiResult::no_content().to_http(),
            Err(e) => {
                let error_opt = e.to_api_error_message();

                if let Some(d) = error_opt {
                    match e {
                        E::EmailAlreadyVerified         => ApiResult::bad_request().with_reason(d).to_http(),
                        E::EmailVerificationExpired     => ApiResult::gone().with_reason(d).to_http(),
                        E::VerificationEmailNotFound    => ApiResult::gone().with_reason(d).to_http(),
                        E::VerificationHashCheckFailed  => ApiResult::unauthorized().with_reason(d).to_http(),
                        _                               => ApiResult::server_error().to_http()
                    }
                } else {
                    ApiResult::server_error().to_http()
                }
            }
        }
    }
}