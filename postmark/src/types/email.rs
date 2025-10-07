use serde::Serialize;
use crate::enums::MessageStream;


#[derive(Debug,Serialize)]
pub struct Email<'a> {
    #[serde(rename = "To")] 
    pub to_address: &'a str,               // short and catchy, 60 characters or less

    #[serde(rename = "Subject")] 
    pub subject: &'a str,               // short and catchy, 60 characters or less
    
    #[serde(rename = "HtmlBody")] 
    pub html_body: &'a str,             // styled body

    #[serde(rename = "TextBody")] 
    pub text_body: &'a str,             // unstyled body

    #[serde(rename = "From")] 
    pub from_address: &'a str,          // sending email address

    #[serde(rename = "ReplyTo")] 
    pub reply_to_address: &'a str,      // reply-to email address
    
    #[serde(rename = "MessageStream")] 
    pub message_stream: MessageStream   // server message stream
}