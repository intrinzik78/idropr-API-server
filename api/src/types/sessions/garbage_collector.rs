use std::{sync::RwLock,collections::HashMap,time::{Duration,Instant}};
use crate::enums::{Error,ExpiredStatus};

use super::{Session};

type Result<T> = std::result::Result<T,Error>;

#[derive(Debug,Default)]
pub struct GarbageCollector;

impl GarbageCollector {
    /// accepts a locked shard and removes expired sessions
    pub fn sweep(&self, list: &RwLock<HashMap<[u8;16],Session>>, collection_ttle:u64) -> Result<()> {
        let time = Duration::from_millis(collection_ttle);
        let stop_time = Instant::now().checked_add(time).ok_or(Error::SessionGarbageInstantFailed)?;
        let mut now = Instant::now();
        let mut sessions_to_remove: Vec<[u8;16]> = Vec::with_capacity(2048);

        // begin locked read scope
        {
            let locked_list = list.read().map_err(|_e| Error::PoisonedSessionList)?;
            let mut list = locked_list.iter();

            while let Some((key,session)) = list.next() {
                if session.is_expired() == ExpiredStatus::Expired {
                    sessions_to_remove.push(*key);
                }

                // qty and time cap
                if sessions_to_remove.len() == 2048 || now > stop_time {
                    break;
                }

                now = Instant::now();
            }
        }
        // end locked read scope

        // begin locked write scope
        if !sessions_to_remove.is_empty() {
            let mut locked_list = list.write().map_err(|_e| Error::PoisonedSessionList)?;

            for k in sessions_to_remove {
                locked_list.remove(&k);
            }
        }
        // end locked write scope

        Ok(())
    }

}