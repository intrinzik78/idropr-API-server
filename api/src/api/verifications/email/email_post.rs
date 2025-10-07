use std::num::NonZeroU8;

use actix_web::{web::{Data, Json}, Responder};
use database::types::DatabaseConnection;
use email_template::{
    enums::Locale,
    traits::EmailTemplate,
    types::{verification::InitialVerificationEmail, RenderedEmail}
};
use postmark::enums::SendStatus;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    enums::{EmailID, Error, RowsUpdated, Uuid, VerificationStatus},
    traits::ToHash,
    types::{email::{Email, EmailVerification, PostmarkLog, SuppressedEmail}, ApiErrorData, ApiResponse, AppState}
};

type Result<T> = std::result::Result<T,Error>;

#[derive(Debug,ToSchema,Deserialize)]
pub struct EmailVerificationPost {
    pub email: String
}

#[derive(ToSchema)]
pub struct CreateEmailVerification;

impl CreateEmailVerification {

    /// process the request
    pub async fn logic(post: Json<EmailVerificationPost>, shared: Data<AppState>) -> Result<()> {

        // create new uuid for email id
        let uuid = match Uuid::web_safe(NonZeroU8::new(16))? {
            Uuid::WebSafe(s) => s,
            _ => return Err(Error::WrongUuidTypeForEmailVerification),
        };

        // convert uuid to hash for storing in database as binary(32)
        let hash = uuid.to_hash()?.as_bytes().to_owned();
       
        // get database reference
        let database = shared.database();

        // check if email is on the suppressed list
        if SuppressedEmail::by_email(&post.email, database).await?.is_some() {
            return Err(Error::EmailIsSuppressed)
        };

        // check for existing verification email and process the correct branch
        let record_opt = EmailVerification::by_email(&post.email, database).await?;
        let record_id = {
            if let Some(record) = record_opt {
                let () = Self::update_existing(&record, &hash, database).await?;
                record.id()
            } else {
                EmailVerification::into_db(&post.email, &hash, database).await? as i64
            }
        };

        // dynamic content for email
        let expire_minutes= 10;
        let verify_url = format!("https://www.battlehouston.com/{uuid}/{}",&record_id.to_string());

        // render email
        let rendered_email = Self::build_email(&verify_url, expire_minutes)?;
        
        // build postmark email
        let email_id = EmailID::EmailVerification as u64;
        let email_body = Email::by_id(email_id, database).await?;
        let email = postmark::types::Email {
            to_address: &post.email,
            subject: email_body.subject(),
            html_body: &rendered_email.html,
            text_body: &rendered_email.text,
            from_address: email_body.from_address(),
            reply_to_address: email_body.reply_to_address(),
            message_stream: postmark::enums::MessageStream::Default
        };

        // send email
        let postmark_response = shared.postmark().send(email).await?;
        let status = match postmark_response.error_code {
            0 => SendStatus::Accepted,
            _ => SendStatus::Rejected
        };

        // insert postmark log
        let _insert_id = PostmarkLog::into_db(email_id, status.clone(), &postmark_response, database).await?;

        // return error response if present
        if status != SendStatus::Accepted {
            return Err(Error::VerificationEmailRejected);
        }

        Ok(())
    }
    
    /// update record branch
    pub async fn update_existing(record: &EmailVerification, hash:&[u8;32], database: &DatabaseConnection) -> Result<()> {
        // check if user is submitting too quickly
        if !record.may_refresh() {
            return Err(Error::RateLimitedEmailVerification)
        }

        // check if email is already verified
        if record.is_verified() == VerificationStatus::Verified {
            return Err(Error::EmailAlreadyVerified)
        }
        
        // update existing record
        match EmailVerification::refresh_by_id(record.id(), hash, database).await? {
            RowsUpdated::Some(1) => Ok(()),
            RowsUpdated::None => Err(Error::TooFewRowsUpdated),
            _ => Err(Error::TooManyRowsUpdated)
        }
    }

    /// build templated email
    pub fn build_email(verify_url: &'_ str, expire_minutes: u32) -> Result<RenderedEmail> {
        let locale = Locale::Default;
        let template = InitialVerificationEmail {
            verify_url,
            expire_minutes
        };

        Ok(template.render(&locale)?)
    }

    /// main entry point
    pub async fn response(post: Json<EmailVerificationPost>, shared: Data<AppState>) -> impl Responder {
        type E = Error;

        match Self::logic(post,shared).await {
            Ok(()) => ApiResponse::resource_created().ok(),
            Err(e) =>{
                let error_opt = e.to_api_error_message();

                if let Some(d) = error_opt {
                    match e {
                        E::EmailAlreadyVerified         => ApiResponse::<ApiErrorData>::default().with_code(400).with_data(d).error(),
                        E::RateLimitedEmailVerification => ApiResponse::<ApiErrorData>::default().with_code(429).with_data(d).error(),
                        E::EmailIsSuppressed            => ApiResponse::<ApiErrorData>::default().with_code(403).with_data(d).error(),
                        E::VerificationEmailRejected    => ApiResponse::<ApiErrorData>::default().with_code(400).with_data(d).error(),
                        _                               => ApiResponse::server_error().error()
                    }
                } else {
                    ApiResponse::server_error().error()
                }
            }
        }
    }
}