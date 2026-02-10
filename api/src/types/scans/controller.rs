use std::{sync::Mutex, time::{Duration,Instant}};
use database::types::DatabaseConnection;
use doc_extractor::{self, enums::ScanStatus};
use tokio::time::MissedTickBehavior;

use crate::{
    enums::Error,
    traits::{ToJitter,ToLockError}, types::scans::{Extraction, ScanSession}
};

type Result<T> = std::result::Result<T,Error>;

const BASE_INTERVAL:u64 = 300;   // seconds
const WAKE_INTERVAL:u64 = 5;    // seconds

#[derive(Debug,Default)]
pub struct ScanController {
    // running: Mutex<bool>,
    wake: Mutex<bool>
}


impl ScanController {
    pub async fn run(&self, connection: &DatabaseConnection) {
        println!("running...");
        let status = ScanStatus::Queued;
        let mut sessions = match ScanSession::by_status(status, connection).await {
            Ok(list) => list,
            Err(e) => {
                // log error here
                println!("{e}");
                return
            }
        };

        // outer loop: sessions ready for work
        let session_status = ScanStatus::Running;
        
        while let Some(session) = sessions.pop() {
            // try next session if work can't be claimed
            if let Err(_) = ScanSession::update_status_by_id(session.id(), session_status, connection).await {
                continue;
            }

            // fetch docs from db
            let docs = match Extraction::by_session_id_with_status(session.id(),ScanStatus::Created,connection).await {
                Ok(v) => v,
                Err(e) => {
                    // log error here
                    println!("{e}");
                    continue;
                }
            };

            // inner loop: docs ready for work
            let doc_running_status = ScanStatus::Running;

            for doc in docs.iter() {
                
                // update status to running
                let update = match Extraction::set_status(doc_running_status, doc.id(), connection).await {
                    Ok(r) => r.require_one(Error::ScanDocStatusNotUpdated),
                    Err(e)              => {
                        // log error here
                        println!("{e}");
                        continue;
                    }
                };

                // try next on failed update
                if update.is_err() {
                    continue;
                }
                
                // load file
                let pathbuf = match doc.blob_path("/var/data") {
                    Ok(p) => p,
                    Err(e) => {
                        // log error here
                        println!("{e}");
                        continue;
                    }
                };

                let bytes = match tokio::fs::read(&pathbuf).await {
                    Ok(b) => b,
                    Err(e) => {
                        // log error here
                        println!("{e}");
                        continue;
                    }
                };

                println!("loaded doc_id={} bytes={} path={}", doc.id(), bytes.len(), &pathbuf.display());
            }

        }
    }

    pub fn wake(&self) -> Result<()> {
        let mut locked_flag = self.wake
            .lock()
            .to_lock_error(Error::ScannerReadLockNotAquired)?;

        *locked_flag = true;

        Ok(())
    }

    pub async fn watch(&self, connection: DatabaseConnection) {
        let base = BASE_INTERVAL;
        let mut next_refresh = base.to_jitter_secs();

        let mut interval = tokio::time::interval(Duration::from_secs(WAKE_INTERVAL));
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            // sleep for WAKE_INTERVAL seconds
            interval.tick().await;

            // get current Instant
            let now = Instant::now();

            // read wake flag
            let should_wake = match self.wake
                .lock()
                .to_lock_error(Error::ScannerReadLockNotAquired)
            {
                Ok(mut f) => {
                    let w = *f;
                    *f = false;
                    w
                }
                Err(e) => {
                    // log error here
                    println!("{e}");
                    false
                }
            };
            
            // run vision if ready
            if now >= next_refresh || should_wake {
                self.run(&connection).await;
                next_refresh = base.to_jitter_secs();
            }
        }
    }
}

