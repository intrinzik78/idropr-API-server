/// the primary purpose of the rate limiter is to deter good actors from overusing resources, not necessarily to deter DDOS attacks.
use std::{
    hash::{Hash,Hasher},
    collections::hash_map::{
        DefaultHasher,
        HashMap
    },
    net::IpAddr,
    sync::{Mutex, RwLock}
};

use crate::{
    enums::{
        Decision,
        ListStatus,
        RateLimitError,
        RefillRate
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
    max_tokens_per_bucket: u32,
    initial_tokens_per_bucket: u32,
    base_refill_rate: RefillRate
}

impl RateLimiter {
    /// takes the builder and passes back the RateLimiter
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
            max_tokens_per_bucket: builder.bucket_capacity,
            initial_tokens_per_bucket: builder.initial_tokens_per_bucket,
            base_refill_rate: builder.refill_rate
        }
    }

    /// adds a connection to the blacklist
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

    /// adds a connection to the whitelist
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

    /// checks the blacklist for an ip
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

    /// checks the whitelist for an ip
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

    /// hashes an ip address for shard routing
    fn hash(&self, ip_address: &IpAddr) -> usize {
        let mut hasher = DefaultHasher::new();
        let shard_count = self.shards.len();
        
        ip_address.hash(&mut hasher);
        
        let hash = hasher.finish() as usize;
        
        hash % shard_count
    }

    /// entry point for a conenction
    pub fn try_connect(&self, ip_address: &str) -> Result<Decision> {
        let ip_address: IpAddr = ip_address.parse()?;
        let block_id = self.hash(&ip_address);

        // early return, no mutex locked
        if self.is_blacklisted(ip_address)? == ListStatus::Blacklisted {
            return Ok(Decision::Denied);
        }

        // early return, no mutex locked
        if self.is_whitelisted(ip_address)? == ListStatus::Whitelisted {
            return Ok(Decision::Approved);
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
                    let bucket = TokenBucket::new()
                        .with_capacity(self.max_tokens_per_bucket)
                        .with_initial_tokens(self.initial_tokens_per_bucket)
                        .with_refill_rate(self.base_refill_rate.clone());

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
    use std::{str::FromStr, time::Instant};
    use rand::Rng;

    use crate::enums::RefillRate;

    use super::*;

    /// tests blacklist and whitelist short-circuits
    #[test]
    fn test_add_to_black_and_whitelist() {
        let rate_limiter = RateLimitBuilder::default()
            .with_initial_capacity(100)
            .with_refill_rate(RefillRate::PerHour(60.0))
            .shard_into(2)
            .with_tokens_per_bucket(10)
            .build();

        let ip_address = IpAddr::from_str("127.0.0.1").expect("test failed parsing &str to ip address");

        rate_limiter.add_to_whitelist(ip_address, 60).expect("test failed adding ip address to whitelist");
        let decision = rate_limiter.try_connect(&"127.0.0.1").expect("test failed on try_connect()");
        assert_eq!(decision,Decision::Approved);

        rate_limiter.add_to_blacklist(ip_address, 60).expect("test failed adding ip address to blacklist");
        let decision = rate_limiter.try_connect(&"127.0.0.1").expect("test failed on try_connect()");
        assert_eq!(decision,Decision::Denied);
        
        let list_status = rate_limiter.is_whitelisted(ip_address).expect("test failed checking checking for white list status");
        assert_eq!(list_status, ListStatus::Whitelisted);

        let list_status = rate_limiter.is_blacklisted(ip_address).expect("test failed checking checking for white list status");
        assert_eq!(list_status, ListStatus::Blacklisted);
    }

    /// stress test successful connections
    #[test]
    fn test_try_connect() {
        let tokens_per_bucket = 10;
        let rate_limiter = RateLimitBuilder::default()
            .with_initial_capacity(1_000)
            .with_refill_rate(RefillRate::PerHour(60.0))
            .shard_into(4)
            .with_tokens_per_bucket(tokens_per_bucket)
            .with_bucket_capacity(tokens_per_bucket)
            .build();

        let mut rng = rand::rng();
        let items_to_test = 100_000;

        let mut ip_addresses: HashMap<String,()> = HashMap::with_capacity(items_to_test);
  
        for _ in 0..items_to_test {
            let a = rng.random_range(1..255);
            let b = rng.random_range(1..255);
            let c = rng.random_range(1..255);
            let d = rng.random_range(1..255);
            let ip = format!("{a}.{b}.{c}.{d}");
            ip_addresses.insert(ip,());
        }

        let a = Instant::now();
        ip_addresses.drain().for_each(|(ip,_)| {
            // approved connections
            for _x in 0..tokens_per_bucket+1 {
                let decision = rate_limiter.try_connect(&ip).expect("test failed on try_connect()");
                assert_eq!(decision,Decision::Approved);
            }

            // denied connection
            let decision = rate_limiter.try_connect(&ip).expect("test failed on try_connect()");
            assert_eq!(decision,Decision::Denied);
        });
        let b = Instant::now();

        let c = b - a;
        println!("\n{} miliseconds elapsed during token use test.\n", c.as_millis());
    }

    /// stress test black list
    #[test]
    fn test_blacklist_attempts() {
        let tokens_per_bucket = 10;
        let rate_limiter = RateLimitBuilder::default()
            .with_initial_capacity(1_000_000)
            .with_refill_rate(RefillRate::PerHour(60.0))
            .shard_into(4)
            .with_tokens_per_bucket(tokens_per_bucket)
            .with_bucket_capacity(tokens_per_bucket)
            .build();

        let mut rng = rand::rng();
        let items_to_test = 1_000_000;

        let mut ip_addresses: HashMap<String,IpAddr> = HashMap::with_capacity(items_to_test);

        for _ in 0..items_to_test {
            let a = rng.random_range(1..255);
            let b = rng.random_range(1..255);
            let c = rng.random_range(1..255);
            let d = rng.random_range(1..255);
            let ip = format!("{a}.{b}.{c}.{d}");
            let ip_addr = IpAddr::from_str(&ip).unwrap();
            ip_addresses.insert(ip,ip_addr);
        }

        let mut blacklist_connections = ip_addresses.clone();

        blacklist_connections.drain().for_each(|(_ip,addr)| {
            rate_limiter.add_to_blacklist(addr, 60).unwrap();
        });

        let mut denied_connections = ip_addresses.clone();

        let a = Instant::now();
        denied_connections.drain().for_each(|(ip,_)| {
            let decision = rate_limiter.try_connect(&ip).expect("test failed on try_connect()");
            assert_eq!(decision,Decision::Denied);
        });
        let b = Instant::now();

        let c = b - a;
        println!("\n{} miliseconds elapsed during blacklist test.\n", c.as_millis());
    }
}