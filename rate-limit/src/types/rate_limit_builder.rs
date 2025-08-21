use std::{collections::HashMap, net::IpAddr};

use crate::{enums::RefillRate, types::{RateLimiter,Timer,TokenBucket}};

#[derive(Clone,Debug)]
pub struct RateLimitBuilder {
    pub map: HashMap<IpAddr,TokenBucket>,            /// collection of ip addresses and associated connection data
    pub bucket_capacity: u32,
    pub initial_tokens_per_bucket: u32,
    pub refill_rate: RefillRate,
    pub blacklist: HashMap<IpAddr,Timer>,
    pub whitelist: HashMap<IpAddr,Timer>,
    pub threads: usize
}

impl RateLimitBuilder {
    /// constructor
    pub fn new(
        default_map_size: usize,
        bucket_capacity: u32,
        initial_tokens_per_bucket: u32,
        refill_rate: RefillRate,
        threads: usize
    ) -> Self {
        // default settings
        let whitelist: HashMap<IpAddr,Timer> = HashMap::new();
        let blacklist: HashMap<IpAddr,Timer> = HashMap::new();

        // allocate and move into mutex
        let map: HashMap<IpAddr, TokenBucket> = HashMap::with_capacity(default_map_size);

        RateLimitBuilder {
            map,
            bucket_capacity,
            initial_tokens_per_bucket,
            refill_rate,
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
    pub fn with_tokens_per_bucket(mut self, tokens: u32) -> Self {
        self.initial_tokens_per_bucket = tokens;
        self
    }

    /// set the default bucket capcity
    pub fn with_bucket_capacity(mut self, tokens: u32) -> Self {
        self.bucket_capacity = tokens;
        self
    }
    
    /// set the default monitoring window in seconds
    pub fn with_refill_rate(mut self, rate: RefillRate) -> Self {
        self.refill_rate = rate;
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
        // base settings
        let default_map_size:usize = 100;
        let bucket_capacity = 50;
        let initial_tokens_per_bucket = 10;
        let base_refill_rate = RefillRate::PerMinute(50.0);
        let threads = 2;

        Self::new(
            default_map_size,
            bucket_capacity,
            initial_tokens_per_bucket,
            base_refill_rate,
            threads
        )
    }
}

#[cfg(test)]
pub mod test {
    use crate::RateLimitBuilder;

    /// tests the default builder and associated builder functions
    #[test]
    fn builder() {
        let _default = RateLimitBuilder::default()
            .with_initial_capacity(1000)
            .with_tokens_per_bucket(100)
            .with_bucket_capacity(200)
            .with_refill_rate(crate::enums::RefillRate::PerMinute(60.0))
            .shard_into(4)
            .build();
    }
}