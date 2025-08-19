use blake3::Hash;
use std::time::{Duration, Instant};
use rand::random_range;

use crate::{
    enums::{ExpiredStatus, RefreshStatus, User},
    types::KeySet
};

const BASE_REFRESH_TIME:u64 = 60 * 10;      // 10 minutes
const MAX_SESSION_AGE:u64 = 60 * 60 * 3;    // 3 days

#[derive(Clone,Debug)]
pub struct Session {
    pub hash: Hash,
    pub next_refresh: Instant,
    pub user: User
}

impl Session {

    /// creates a new session container
    pub fn new(key_set: &KeySet, user: User) -> Self {
        let now = Instant::now();
        let jitter = random_range(0.8..1.2);
        let duration_secs = (BASE_REFRESH_TIME as f32 * jitter).trunc() as u64;
        let next_refresh = now
            .checked_add(Duration::from_secs(duration_secs))
            .or(Some(now))
            .expect("unreachable after .or()");

        Session {
            hash: key_set.hash,
            next_refresh,
            user
        }
    }

    /// returns a refresh status
    pub fn is_stale(&self) -> RefreshStatus {
        let now = Instant::now();

        if now > self.next_refresh {
            RefreshStatus::Refresh
        } else {
            RefreshStatus::None
        }
    }

    /// returns expired status
    pub fn is_expired(&self) -> ExpiredStatus {
        let now = Instant::now();
        let time_to_expiration = Duration::from_secs(MAX_SESSION_AGE);
        let expiration = now
            .checked_add(time_to_expiration)
            .or(Some(now))
            .expect("unreachable after .or()");

        if expiration > self.next_refresh {
            ExpiredStatus::Expired
        } else {
            ExpiredStatus::NotExpired
        }
    }
}