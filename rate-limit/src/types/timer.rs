use std::time::{Duration,Instant};

#[derive(Clone,Debug,PartialEq)]
pub struct Timer {
    expires: Box<Option<Instant>>
}

impl Timer {
    pub fn new(secs: u64) -> Self {
        let expires = Instant::now().checked_add(Duration::from_secs(secs));
        let boxed = Box::new(expires);

        Timer {
            expires: boxed
        }
    }

    pub fn expires(self) -> Box<Option<Instant>> {
        self.expires
    }
}