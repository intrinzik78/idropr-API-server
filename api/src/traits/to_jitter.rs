use std::time::{Duration, Instant};

use rand::random_range;

pub trait ToJitter {
    fn to_jitter_micros(&self) -> Instant;
    fn to_jitter_millis(&self) -> Instant;
    fn to_jitter_nanos(&self) -> Instant;
    fn to_jitter_secs(&self) -> Instant;
}

impl ToJitter for u64 {
    fn to_jitter_micros(&self) -> Instant {
        let now = Instant::now();
        let jitter = random_range(0.8..1.2);
        let duration_secs = (*self as f64 * jitter).trunc() as u64;
        
        now.checked_add(Duration::from_micros(duration_secs))
            .or(Some(now))
            .expect("unreachable after .or()")
    }

    fn to_jitter_millis(&self) -> Instant {
        let now = Instant::now();
        let jitter = random_range(0.8..1.2);
        let duration_secs = (*self as f64 * jitter).trunc() as u64;
        
        now.checked_add(Duration::from_millis(duration_secs))
            .or(Some(now))
            .expect("unreachable after .or()")
    }

    fn to_jitter_nanos(&self) -> Instant {
        let now = Instant::now();
        let jitter = random_range(0.8..1.2);
        let duration_secs = (*self as f64 * jitter).trunc() as u64;
        
        now.checked_add(Duration::from_nanos(duration_secs))
            .or(Some(now))
            .expect("unreachable after .or()")
    }

    fn to_jitter_secs(&self) -> Instant {
        let now = Instant::now();
        let jitter = random_range(0.8..1.2);
        let duration_secs = (*self as f64 * jitter).trunc() as u64;
        
        now.checked_add(Duration::from_secs(duration_secs))
            .or(Some(now))
            .expect("unreachable after .or()")
    }

}

