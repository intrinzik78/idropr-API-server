use std::time::Instant;

use crate::enums::RefillRate;

const DAY_AS_SECS:f32 = 24.0 * 60.0 * 60.0;
const HOUR_AS_SECS:f32 = 60.0 * 60.0;
const MINUTE_AS_SECS:f32 = 60.0;
const SECOND:f32 = 1.0;

#[derive(Clone,Debug)]
pub struct TokenBucket {
    tokens: i32,
    last_connect: Instant,
    last_refill: Instant,
    capacity: u32,
    refill_rate: RefillRate,
}

impl TokenBucket {
    pub fn new() -> Self {
        let tokens = 1;
        let capacity = 50;

        TokenBucket {
            tokens,
            last_refill: Instant::now(),
            last_connect: Instant::now(),
            capacity,
            refill_rate: RefillRate::PerHour(tokens as f32)
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
        let time_since_last_connect = self.last_connect.elapsed().as_secs_f32();
        let time_since_last_refill = self.last_refill.elapsed().as_secs_f32();

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
    
    /// returns the number of tokens remaining after connection
    pub fn drip(&mut self) -> i32 {
        let tokens = self.refill();

        self.last_connect = Instant::now();
        self.tokens -= 1;

        tokens
    }
}

impl Default for TokenBucket {
    fn default() -> Self {
        Self::new()
    }
}