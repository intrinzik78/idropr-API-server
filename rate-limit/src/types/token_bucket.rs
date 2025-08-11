use chrono::{DateTime,Utc};

#[derive(Clone,Debug)]
pub struct TokenBucket {
    tokens: i32,
    last_connect: DateTime<Utc>
}

impl TokenBucket {
    pub fn new(tokens: i32) -> Self {
        TokenBucket {
            tokens,
            last_connect: Utc::now()
        }
    }

    pub fn drip(&mut self) -> i32 {
        self.tokens -= 1;
        self.last_connect = Utc::now();
        
        self.tokens
    }

    pub fn last_connect(&self) -> &DateTime<Utc> {
        &self.last_connect
    }
}