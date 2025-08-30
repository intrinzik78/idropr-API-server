use blake3::Hash;
use blake3;
use chrono::{DateTime,Utc};
use sqlx::{prelude::FromRow, MySql, Transaction};
use std::time::{Duration, Instant};
use rand::random_range;

use crate::{
    enums::{Error, ExpiredStatus, RefreshStatus, User, Uuid, VerificationStatus},
    traits::{ToBase64, ToVerificationStatus},
    types::{DatabaseConnection, KeySet}
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
    id: i64,
    user_id: i64,
    hash: String,
    timestamp: DateTime<Utc>
}

impl DatabaseSession {

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn user_id(&self) -> i64 {
        self.user_id
    }

    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }

    /// insert session into database as transaction
    pub async fn into_db(user_id: i64, hash: &str, tx: &mut Transaction<'_,MySql>) -> Result<i64> {
        let sql = "INSERT INTO `session` (user_id,hash) VALUES (?,?) ON DUPLICATE KEY UPDATE hash = ?";
        let insert_id = sqlx::query(sql)
            .bind(user_id)
            .bind(hash)
            .bind(hash)
            .execute(&mut **tx)
            .await?
            .last_insert_id() as i64;

        Ok(insert_id)
    }

    /// check and refresh the database if a valid entry exists
    pub async fn by_user_id(user_id: i64, database: &DatabaseConnection) -> Result<DatabaseSession> {
        let sql = "SELECT id,user_id,hash,timestamp FROM `session` WHERE session.user_id = ?";
        let session_opt:Option<DatabaseSession> = sqlx::query_as(sql)
            .bind(user_id)
            .fetch_optional(&database.pool)
            .await?;

        if let Some(session) = session_opt {
            Ok(session)
        } else {
            Err(Error::SessionNotFoundDuringRefresh)
        }
    }

    /// verify database session with token
    pub async fn verify(&self, uuid: Uuid, token: &str) -> Result<VerificationStatus> {
        // extract uuid from Uuid enum
        let key = match uuid {
            Uuid::Crypto(buf) => buf,
            _ => return Err(Error::SessionTokenIncorrectType)
        };

        let memory_hash = blake3::keyed_hash(&key, token.as_bytes()).as_bytes().to_base64_url();
        let db_hash = &self.hash;

        // constant time compare
        Ok((db_hash == &memory_hash).to_verification_status())
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


