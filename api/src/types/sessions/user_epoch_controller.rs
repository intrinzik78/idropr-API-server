use std::collections::HashMap;

use database::types::DatabaseConnection;
use sqlx::prelude::FromRow;

use crate::{
    enums::Error,
    types::sessions::EpochMeta
};

const EPOCH_TABLE_LIMIT:i64 = 1000;

type Result<T> = std::result::Result<T,Error>;



#[derive(Debug,FromRow)]
struct CurrentUserEpoch {
    current:u64
}

#[derive(Debug,Default)]
pub struct UserEpochController {
    master_epoch:u64,
    user_epoch_map:HashMap<i64,u64>
}

/// sync
impl UserEpochController {
    /// getter for the master epoch
    pub fn current(&self) -> u64 { self.master_epoch }

    /// getter for user epoch
    pub fn user_epoch(&self,user_id:i64) -> Option<u64> {
        self.user_epoch_map.get(&user_id).copied()
    }

    /// updates the master epoch
    pub fn set_master_epoch(&mut self, epoch:u64) { self.master_epoch = epoch; }

    /// updates the list with the latest user epoch
    pub fn update(&mut self, user_id:i64, user_epoch:u64) {
        self.user_epoch_map
            .entry(user_id)
            .and_modify(|e| *e = user_epoch)
            .or_insert(user_epoch);
    }

    /// constructs new controller
    pub async fn new(connection: &DatabaseConnection) -> Result<Self> {
        // get the last epoch_events_id from the database
        let master_epoch = Self::last_db_epoch(connection).await?;

        // get the list of user id and user epoch from the database
        let sql = "SELECT user.id AS user_id,user.epoch as user_epoch FROM `user`";
        let user_epoch_list:Vec<EpochMeta> = sqlx::query_as(sql)
            .fetch_all(&connection.pool)
            .await?;

        // allocate
        let mut user_epoch_map = HashMap::with_capacity(user_epoch_list.len());

        // insert into hash map
        for record in user_epoch_list {
            user_epoch_map.insert(record.user_id, record.user_epoch);
        }
    
        // returns the built controller
        Ok(Self {
            master_epoch,
            user_epoch_map
        })
    }
}

/// async
impl UserEpochController {
    /// ergonomic getter for the lastest master epoch
    pub async fn next(connection: &DatabaseConnection) -> Result<u64> {
        Self::last_db_epoch(connection).await
    }

    /// canonical getter for the last entry in the master table
    #[inline]
    async fn last_db_epoch(connection: &DatabaseConnection) -> Result<u64> {
        let sql = "SELECT MAX(id) AS current FROM `user_epoch_events`";
        let epoch:CurrentUserEpoch = sqlx::query_as(sql).fetch_one(&connection.pool).await?;

        Ok(epoch.current)
    }

    /// gets the list of users whose epochs have changed since the last poll
    pub async fn change_list(master_epoch:u64, connection: &DatabaseConnection) -> Result<Vec<EpochMeta>> {
        let sql = "SELECT DISTINCT user.id AS user_id,user.epoch AS user_epoch
                        FROM (SELECT user_epoch_events.user_id FROM `user_epoch_events` WHERE user_epoch_events.id > ?) AS list
                        JOIN `user` on list.user_id = user.id LIMIT ?";
        
        let results:Vec<EpochMeta> = sqlx::query_as(sql)
            .bind(master_epoch)
            .bind(EPOCH_TABLE_LIMIT)
            .fetch_all(&connection.pool)
            .await?;

        Ok(results)
    }
}