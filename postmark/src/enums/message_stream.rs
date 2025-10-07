use std::fmt::Display;
use serde::Serialize;

#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageStream {
    #[serde(rename = "outbound")] 
    Default,
    Transactional,
    System,
    Campaign
}

impl Display for MessageStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        type M = MessageStream;

        match self {
            M::Default => write!(f, "outbound"),
            M::Transactional => write!(f, "transactional"),
            M::System => write!(f, "system"),
            M::Campaign => write!(f, "campaign")
        }
    }
}