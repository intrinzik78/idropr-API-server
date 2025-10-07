use reqwest::Client;
use crate::{
    enums::PostmarkError,
    types::{Email,Env,PostmarkResponse}
};

type Result<T> = std::result::Result<T,PostmarkError>;

#[derive(Debug)]
pub struct Postmark {
    api_key: String,
    allowed_senders: Vec<String>
}

impl Postmark {

    pub async fn send(&self, email: Email<'_>) -> Result<PostmarkResponse> {
        // verify the sending email address is in the permitted sender list
        if !self.allowed_senders.contains(&email.from_address.to_string()) {
            return Err(PostmarkError::SendingEmailAddressNotPermitted);
        }

        let url = "https://api.postmarkapp.com/email";

        let client = Client::new();

        // build and send request
        let api_response:PostmarkResponse = client
            .post(url)
            .json(&email)
            .header("Accept", "application/json")
            .header("X-Postmark-Server-Token", &self.api_key)
            .header("Content-Type","application/json")
            .header("X-PM-Message-Stream", email.message_stream.to_string())
            .send() 
            .await?                             // return early on http response error
            .json()   // return early on api response parse failure
            .await?;

        // check for api error message and return response
        if api_response.error_code == 0 {
            Ok(api_response)
        } else {
            Err(PostmarkError::Api(api_response))
        }
    }
}

impl Default for Postmark {
    fn default() -> Self {
        let env = Env::default();
        let allowed_senders:Vec<String> = env.postmark_senders;

        Self {
            api_key: env.postmark_secret,
            allowed_senders
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::MessageStream;

    /// constructor build test
    #[actix_rt::test]
    async fn send_postmark_test_email() {
        // build test values
        let mut postmark = Postmark::default();
        let subject =  String::from("subject");
        let html_body =  String::from("html_body");
        let text_body =  String::from("text_body");
        let to_address =  String::from("testing@battletexas.com");
        let from_address =  String::from("testing@battletexas.com");
        let reply_to_address =  String::from("testing@battletexas.com");

        postmark.allowed_senders.push(String::from("testing@battletexas.com"));
        
        // set test stream
        let message_stream = MessageStream::System;

        // build test object
        let email = Email {
            to_address: &to_address,
            subject: &subject,
            html_body: &html_body,
            text_body: &text_body,
            from_address: &from_address,
            reply_to_address: &reply_to_address,
            message_stream
        };

        let _result = postmark.send(email).await.unwrap();
    }
}