use std::time::{Duration, Instant};

use crate::enums::{BucketStatus, Decision, RefillRate};

const DAY_AS_SECS:f32 = 24.0 * 60.0 * 60.0;
const HOUR_AS_SECS:f32 = 60.0 * 60.0;
const MINUTE_AS_SECS:f32 = 60.0;
const SECOND:f32 = 1.0;
const MIN_BUCKET_TTL:u64 = 60 * 2; // 2 minutes

#[derive(Clone,Debug)]
pub struct TokenBucket {
    capacity: u32,
    last_connect: Instant,
    last_refill: Instant,
    refill_rate: RefillRate,
    tokens: i32,
    ver: u64
}

impl TokenBucket {

    /// builder
    pub fn new() -> Self {
        let tokens = 1;
        let capacity = 50;

        TokenBucket {
            capacity,
            last_refill: Instant::now(),
            last_connect: Instant::now(),
            refill_rate: RefillRate::PerHour(tokens as f32),
            tokens,
            ver: 0
        }
    }

    /// last connect getter
    pub fn last_connect(&self) -> Instant {
        self.last_refill
    }

    /// ver getter
    pub fn ver(&self) -> u64 {
        self.ver
    }

    /// bucket ttl getter
    pub fn expires_at(&self) -> Instant {
        // bucket ttl should exceed the check window or a bucket could respawn after dropping with refilled tokens
        let time = match self.refill_rate {
            RefillRate::PerDay(_)    => Duration::from_secs(MIN_BUCKET_TTL.max(DAY_AS_SECS as u64)),
            RefillRate::PerHour(_)   => Duration::from_secs(MIN_BUCKET_TTL.max(HOUR_AS_SECS as u64)),
            RefillRate::PerMinute(_) => Duration::from_secs(MIN_BUCKET_TTL.max(MINUTE_AS_SECS as u64)),
            RefillRate::PerSecond(_) => Duration::from_secs(MIN_BUCKET_TTL.max(MIN_BUCKET_TTL as u64)),
        };
        
        self.last_connect
            .checked_add(time)
            .or(Some(self.last_connect))
            .expect("unreachable after .or()")
    }
   
    /// checks ttl vs last connect and returns a BucketStatus
    pub fn is_expired(&self) -> BucketStatus {
        let now = Instant::now();

        // bucket ttl should exceed the check window or a bucket could respawn after dropping with refilled tokens
        let time = match self.refill_rate {
            RefillRate::PerDay(_)    => Duration::from_secs(MIN_BUCKET_TTL.max(DAY_AS_SECS as u64)),
            RefillRate::PerHour(_)   => Duration::from_secs(MIN_BUCKET_TTL.max(HOUR_AS_SECS as u64)),
            RefillRate::PerMinute(_) => Duration::from_secs(MIN_BUCKET_TTL.max(MINUTE_AS_SECS as u64)),
            RefillRate::PerSecond(_) => Duration::from_secs(MIN_BUCKET_TTL.max(MIN_BUCKET_TTL as u64)),
        };
        let expire_time = self.last_connect
            .checked_add(time)
            .or(Some(now))
            .expect("can't fail after the .or()");

        if now >= expire_time {
            BucketStatus::Expired
        } else {
            BucketStatus::NotExpired
        }
    }

    /// sets the bucket capacity
    pub fn with_capacity(mut self, capacity: u32) -> Self {
        self.capacity = capacity;
        self
    }

    /// sets the bucket refill rate
    pub fn with_refill_rate(mut self, refill_rate: RefillRate) -> Self {
        self.refill_rate = refill_rate;
        self
    }

    /// sets the starting number of tokens in the bucket this will prevent new
    /// connections from flooding system with DDOS style attacks
    pub fn with_initial_tokens(mut self, initial_amount: u32) -> Self {
        self.tokens = initial_amount as i32;
        self
    }

    /// caculates the number of new tokens to be added since last connection and adds them to the bucket
    fn refill(&mut self) -> i32 {
        
        // return early on an expired bucket
        if self.is_expired() == BucketStatus::Expired {
            return 0;
        }

        let time_since_last_connect = self.last_connect.elapsed().as_secs_f32();
        let time_since_last_refill = self.last_refill.elapsed().as_secs_f32();

        // determine refill rate
        let window_expired = match self.refill_rate {
            RefillRate::PerDay(_) => time_since_last_connect.ge(&DAY_AS_SECS),
            RefillRate::PerHour(_) => time_since_last_connect.ge(&HOUR_AS_SECS),
            RefillRate::PerMinute(_) => time_since_last_connect.ge(&MINUTE_AS_SECS),
            RefillRate::PerSecond(_) => time_since_last_connect.ge(&SECOND)
        };

        // early return with a full bucket if the window has expired
        if window_expired {
            self.tokens = self.capacity as i32;
            return self.tokens;
        }

        // calculate the refill rate per second
        let refill_rate = match self.refill_rate {
            RefillRate::PerDay(n) => n / DAY_AS_SECS,
            RefillRate::PerHour(n) => n / HOUR_AS_SECS,
            RefillRate::PerMinute(n) => n / MINUTE_AS_SECS,
            RefillRate::PerSecond(n) => n
        };

        let refill_amount = (time_since_last_refill * refill_rate).trunc() as i32;

        if refill_amount > 0 {
            self.tokens += refill_amount;
            self.last_refill = Instant::now();
            self.last_connect = Instant::now();
        } else {
            self.last_connect = Instant::now();
        }

        // ensure the bucket does not exceed max capacity
        self.tokens = self.tokens.min(self.capacity as i32);

        self.tokens
    }

    /// tokens remaining getter  
    pub fn tokens(&self) -> i32 {
        self.tokens
    }

    /// returns the number of tokens remaining after connection
    pub fn drip(&mut self) -> Decision {
        let tokens = self.refill();

        self.tokens -= 1;

        if tokens > 0 {
            self.last_connect = Instant::now();

            // wrap on overflow
            if self.ver + 1 == u64::MAX {
                self.ver = 0;
            } else {
                self.ver += 1;
            }

            Decision::Approved
            
        } else {
            Decision::Denied
        }
    }
}

impl Default for TokenBucket {
    fn default() -> Self {
        Self::new()
    }
}