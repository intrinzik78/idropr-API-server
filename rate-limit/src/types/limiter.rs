use std::{
    hash::{Hash,Hasher},
    collections::hash_map::{
        DefaultHasher,
        HashMap
    },
    net::IpAddr,
    sync::{Mutex, RwLock}, time::Duration
};

use crate::{
    enums::{
        Decision,
        ListStatus,
        RateLimitError
    },
    traits::{
        ToBlackListStatus,
        ToWhiteListStatus
    },
    types::{
        RateLimitBuilder,
        Timer,
        TokenBucket
    }
};

type Result<T> = std::result::Result<T,RateLimitError>;

const BLACK_LIST_TIME:u64 = 60;
const BLACK_LIST_LIMIT:i32 = -25;
const SHARD_FACTOR:usize = 2;       // 2 shards per worker thread

#[derive(Debug)]
#[allow(dead_code)]
pub struct RateLimiter {
    shards: Vec<Mutex<HashMap<IpAddr,TokenBucket>>>,
    blacklist:  RwLock<HashMap<IpAddr,Timer>>,
    whitelist:  RwLock<HashMap<IpAddr,Timer>>,
    tokens_per_bucket: i32,
    window: Duration
}

impl RateLimiter {
    pub fn new(builder: RateLimitBuilder) -> Self {
        let shard_number = SHARD_FACTOR * builder.threads;
        let mut shards: Vec<Mutex<HashMap<IpAddr,TokenBucket>>> = Vec::with_capacity(shard_number);
        
        // build shard list
        for _ in 0..shard_number {
            // clone the pre-configured hashmap
            let map = builder.map.clone();

            // apply the mutex
            let locked_map = Mutex::new(map);

            // push as a shard
            shards.push(locked_map);
        }

        // assemble RateLImiter
        RateLimiter {
            shards,
            blacklist: RwLock::new(builder.blacklist),
            whitelist: RwLock::new(builder.whitelist),
            tokens_per_bucket: builder.tokens_per_bucket,
            window: Duration::from_secs(builder.monitoring_window_secs)
        }
    }

    fn add_to_blacklist(&self, ip_address: IpAddr, secs: u64) -> Result<()> {
        // begin locked scope
        let locked_list = &mut self.blacklist
            .try_write()
            .map_err(|_e| RateLimitError::PoisonedBlacklist)?;

        
        if locked_list.contains_key(&ip_address) {
            return Err(RateLimitError::DuplicateBlacklistEntry(ip_address))
        }

        let timer = Timer::new(secs);
        let _ = locked_list.insert(ip_address, timer);

        Ok(())
    }

    pub fn add_to_whitelist(&self, ip_address: IpAddr, secs: u64) -> Result<()> {
        // begin locked scope
        let mut locked_list = self.whitelist
            .write()
            .map_err(|_e| RateLimitError::PoisonedWhitelistlist)?;

        if locked_list.contains_key(&ip_address) {
            return Err(RateLimitError::DuplicateBlacklistEntry(ip_address))
        }

        let timer = Timer::new(secs);
        let _ = locked_list.insert(ip_address, timer);

        Ok(())
    }

    fn is_blacklisted(&self, ip_address: IpAddr) -> Result<ListStatus> {
        // read-lock
        let result = {
            let locked_list = &self.blacklist
                .read()
                .map_err(|_e| RateLimitError::PoisonedBlacklist)?;
            
            locked_list
                .contains_key(&ip_address)
                .to_blacklist_status()
        };

        Ok(result)
    }

    fn is_whitelisted(&self, ip_address: IpAddr) -> Result<ListStatus> {
        // read-lock
        let result = {
            let locked_list = &self.whitelist
                .read()
                .map_err(|_e| RateLimitError::PoisonedWhitelistlist)?;
            
            locked_list
                .contains_key(&ip_address)
                .to_whitelist_status()
        };

        Ok(result)
    }

    fn hash(&self, ip_address: &IpAddr) -> usize {
        let mut hasher = DefaultHasher::new();
        let shard_count = self.shards.len();
        
        ip_address.hash(&mut hasher);
        
        let hash = hasher.finish() as usize;
        
        hash % shard_count
    }

    pub fn try_connect(&self, ip_address: &str) -> Result<Decision> {
        let ip_address: IpAddr = ip_address.parse()?;
        let block_id = self.hash(&ip_address);

        // early return, no mutex locked
        if self.is_whitelisted(ip_address)? == ListStatus::Whitelisted {
            return Ok(Decision::Approved);
        }

        // early return, no mutex locked
        if self.is_blacklisted(ip_address)? == ListStatus::Blacklisted {
            return Ok(Decision::Denied);
        }

        let mut blacklist_flag = ListStatus::None;

        // begin locked scope
        let result = {
            let locked_list = &mut self.shards[block_id]
                .lock()
                .map_err(|_e| RateLimitError::PoisonedRateLimiterMap)?;

            match locked_list.get_mut(&ip_address) {
                Some(bucket) => {
                    let tokens_remaining = bucket.drip();

                    if tokens_remaining < BLACK_LIST_LIMIT {
                        blacklist_flag = ListStatus::Blacklisted
                    }

                    match tokens_remaining {
                        ..1   => Decision::Denied,
                        1..   => Decision::Approved
                   }
                },
                None => {
                    let bucket = TokenBucket::new(self.tokens_per_bucket);
                    locked_list.insert(ip_address, bucket);

                    Decision::Approved
                }
            }
        };
        // end locked scope

        if blacklist_flag == ListStatus::Blacklisted {
            self.add_to_blacklist(ip_address,BLACK_LIST_TIME)?;
        }

        Ok(result)
    }
}

#[cfg(test)]
pub mod test {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_add_to_black_and_whitelist() {
        let rate_limiter = RateLimitBuilder::default()
            .with_initial_capacity(100)
            .with_monitoring_window_secs(60)
            .shard_into(2)
            .with_tokens_per_bucket(10)
            .build();

        let ip_address = IpAddr::from_str("127.0.0.1").expect("test failed parsing &str to ip address");

        rate_limiter.add_to_whitelist(ip_address, 60).expect("test failed adding ip address to whitelist");
        rate_limiter.add_to_blacklist(ip_address, 60).expect("test failed adding ip address to blacklist");
        
        let list_status = rate_limiter.is_whitelisted(ip_address).expect("test failed checking checking for white list status");
        assert_eq!(list_status, ListStatus::Whitelisted);

        let list_status = rate_limiter.is_blacklisted(ip_address).expect("test failed checking checking for white list status");
        assert_eq!(list_status, ListStatus::Blacklisted);
    }

    #[test]
    fn test_try_connect() {
        let rate_limiter = RateLimitBuilder::default()
            .with_initial_capacity(100)
            .with_monitoring_window_secs(60)
            .shard_into(2)
            .with_tokens_per_bucket(10)
            .build();

        // let ip_address = IpAddr::from_str("127.0.0.1").expect("test failed parsing &str to ip address");
        let ip_address = "127.0.0.1";

        let decision = rate_limiter.try_connect(ip_address).expect("test failed on try_connect()");

        assert_eq!(decision, Decision::Approved);

        for _ in 0..50 {
            rate_limiter.try_connect(ip_address).expect("test failed on try_connect()");
        }

        let decision = rate_limiter.try_connect(ip_address).expect("test failed on try_connect()");

        assert_eq!(decision, Decision::Denied);
    }
}