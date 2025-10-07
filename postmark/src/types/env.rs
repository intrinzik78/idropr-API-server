use std::{collections::HashMap, fmt::Debug};

use dotenv;

// manages importing and testing of the .env file
#[derive(Debug)]
pub struct Env {
    // postmark server settings
    pub postmark_secret: String,       // path to local cert file
    pub postmark_senders: Vec<String>
}

impl Default for Env {
    fn default() -> Self {
        // load values
        let env:HashMap<String,String> = dotenv::vars().collect();

        // extracts env vars
        let postmark_secret = env.get("POSTMARK_SECRET")
            .expect("POSTMARK_SECRET not found in .env")
            .to_owned();

        #[allow(clippy::redundant_closure)]
        let postmark_senders:Vec<String> = env.get("POSTMARK_SENDERS")
            .expect("POSTMARK_SENDERS not found in .env")
            .split(',')
            .map(|s| String::from(s))
            .collect();

        Env {
            postmark_secret,
            postmark_senders
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_postmark_env_builder() {
        // manually construct Env, will fail on missing values
        let manual_env = Env {
            postmark_secret: String::from("postmark_secret"),
            postmark_senders: vec![String::from("test")]
        };

        // test function calls return correct data
        assert_eq!(manual_env.postmark_secret, String::from("postmark_secret"));

        // test constructor generated properties contain some values
        let builder = Env::default();
        assert!(!builder.postmark_secret.is_empty());    
    }
}