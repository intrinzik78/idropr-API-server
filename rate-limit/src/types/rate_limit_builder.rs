use std::{collections::HashMap, net::IpAddr};

use crate::types::{RateLimiter,Timer,TokenBucket};

#[derive(Clone,Debug)]
pub struct RateLimitBuilder {
    pub map: HashMap<IpAddr,TokenBucket>,            /// collection of ip addresses and associated connection data
    pub tokens_per_bucket: i32,
    pub monitoring_window_secs: u64,
    pub blacklist: HashMap<IpAddr,Timer>,
    pub whitelist: HashMap<IpAddr,Timer>,
    pub threads: usize
}

impl RateLimitBuilder {
    /// configures and returns a new RateLimiter
    pub fn new(
        default_capacity: usize,
        tokens_per_bucket: i32,
        monitoring_window_secs: u64,
        threads: usize
    ) -> Self {
        // default settings
        let whitelist: HashMap<IpAddr,Timer> = HashMap::new();
        let blacklist: HashMap<IpAddr,Timer> = HashMap::new();

        // allocate and move into mutex
        let map: HashMap<IpAddr, TokenBucket> = HashMap::with_capacity(default_capacity);

        RateLimitBuilder {
            map,
            tokens_per_bucket,
            monitoring_window_secs,
            blacklist,
            whitelist,
            threads
        }
    }

    /// set the default rate list memory allocation size
    pub fn with_initial_capacity(mut self, new_capacity: usize) -> Self {
        // size check: reserve method will panic on invalid size
        assert!(new_capacity > 0);
        assert!(self.map.len() < new_capacity);

        let additional_capacity = new_capacity - self.map.len();
        self.map.reserve(additional_capacity);

        self
    }

    /// set the default number of request tokens in each bucket
    pub fn with_tokens_per_client(mut self, tokens: i32) -> Self {
        self.tokens_per_bucket = tokens;
        self
    }
    
    /// set the default monitoring window in seconds
    pub fn with_monitoring_window_secs(mut self, seconds: u64) -> Self {
        self.monitoring_window_secs = seconds;
        self
    }

    /// the number of threads the server will use to determine shard quantity
    pub fn shard_into(mut self, threads: usize) -> Self {
        self.threads = threads;
        self
    }

    /// returns the RateLimiter
    pub fn build(self) -> RateLimiter {
        RateLimiter::new(self)
    }
}

impl Default for RateLimitBuilder {
    fn default() -> Self {
        let default_capacity:usize = 10;
        let tokens_per_bucket = 60;
        let monitoring_window_secs = 60;
        let threads = 1;

        Self::new(default_capacity,tokens_per_bucket,monitoring_window_secs,threads)
    }
}