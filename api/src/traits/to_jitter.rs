use std::time::{Duration, Instant};

use rand::random_range;

pub trait ToJitter {
    fn to_jitter(&self) -> Instant;
}

impl ToJitter for u64 {
    fn to_jitter(&self) -> Instant {
        let now = Instant::now();
        let jitter = random_range(0.8..1.2);
        let duration_secs = (*self as f64 * jitter).trunc() as u64;
        
        now.checked_add(Duration::from_secs(duration_secs))
            .or(Some(now))
            .expect("unreachable after .or()")
    }
}

