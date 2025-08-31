use blake3::Hash;
use blake3;
use chrono::{DateTime,Utc};
use sqlx::{prelude::FromRow};
use std::{time::{Duration, Instant}};
use rand::random_range;

use crate::{
    enums::{Error, ExpiredStatus, sessions::RefreshStatus, RowsUpdated, User, VerificationStatus},
    traits::{ToUpdatedResult, ToVerificationStatus},
    types::{DatabaseConnection, sessions::KeySet}
};

type Result<T> = std::result::Result<T,Error>;

const BASE_REFRESH_TIME:u64 = 60 * 60 * 8;       // 8 hours
const MAX_SESSION_AGE:u64 = 60 * 60 * 24 * 10;   // 10 day

#[derive(Clone,Debug)]
pub struct Session {
    pub hash: Hash,
    pub next_refresh: Instant,
    pub user: User
}

#[derive(Clone,Debug,FromRow)]
pub struct DatabaseSession {
    user_id: i64,
    hash: Vec<u8>,
    timestamp: DateTime<Utc>
}

impl DatabaseSession {

    pub fn user_id(&self) -> i64 {
        self.user_id
    }

    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    pub fn hash(&self) -> &[u8] {
        &self.hash
    }

    /// insert session into database as transaction
    pub async fn into_db(user_id: i64, hash: &blake3::Hash, database: &DatabaseConnection) -> Result<RowsUpdated> {
        let slice = hash.as_bytes().as_slice();
        let sql = "INSERT INTO `session` (user_id,hash) VALUES (?,?) ON DUPLICATE KEY UPDATE hash = ?";
        let rows_updated = sqlx::query(sql)
            .bind(user_id)
            .bind(slice)
            .bind(slice)
            .execute(&database.pool)
            .await?
            .rows_affected()
            .to_updated_result();

        Ok(rows_updated)
    }

    pub async fn verify(keyed_hash:&blake3::Hash, database: &DatabaseConnection) -> Result<VerificationStatus> {
        let slice = keyed_hash.as_bytes().as_slice();
        let sql = "UPDATE `session` SET timestamp=CURRENT_TIMESTAMP WHERE session.hash = ?";
        let result = sqlx::query(sql)
            .bind(slice)
            .execute(&database.pool)
            .await?
            .rows_affected()
            .to_updated_result();

        Ok((result == RowsUpdated::RowsUpdated(1)).to_verification_status())
    }
}

impl Session {
    /// calculates the jitter factors
    #[inline]
    fn jitter_time() -> Instant {
        let now = Instant::now();
        let jitter = random_range(0.8..1.2);
        let duration_secs = (BASE_REFRESH_TIME as f32 * jitter).trunc() as u64;
        
        now.checked_add(Duration::from_secs(duration_secs))
           .or(Some(now))
           .expect("unreachable after .or()")
    }

    /// creates a new session container
    pub fn new(key_set: &KeySet, user: User) -> Self {
        let next_refresh = Self::jitter_time();

        Self {
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
        let expiration = self.next_refresh
            .checked_add(time_to_expiration)
            .or(Some(now))
            .expect("unreachable after .or()");

        if now > expiration {
            ExpiredStatus::Expired
        } else {
            ExpiredStatus::NotExpired
        }
    }

    /// sets the next refresh time
    pub fn update_next_refresh(&mut self) {
        self.next_refresh = Self::jitter_time();
    }
}


