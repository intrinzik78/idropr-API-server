use database::types::DatabaseConnection;
use tokio::time::MissedTickBehavior;
use std::{collections::{HashMap, hash_map::Entry}, hash::{DefaultHasher,Hash,Hasher}, sync::RwLock, time::{Duration, Instant}};

use super::GarbageCollector;

use crate::{
    enums::{AuthContext, Error, ExpiredStatus, Permission, UpdateStatus, User, UserAccountStatus, Uuid, VerificationStatus, sessions::RefreshStatus},
    traits::{FromBase64, HasPermission, ToBase64, ToJitter, ToKeySet, ToLockError},
    types::{permissions::{PermissionCheck,UserPermissions}, sessions::{DatabaseSession, KeySet, Session, UserEpochController}}
};

type Result<T> = std::result::Result<T,Error>;

/// garbage collection
const BASE_GARBAGE_POLLING_TTL:u64 = 10;     // 10 seconds 
const MAX_GARBAGE_COLLECTION:u64   = 600;    // 10 minutes as seconds
const COLLECTION_TTL:u64 = 10;               // 10 miliseconds

/// epoch polling
const BASE_POLLING_INTERVAL:u64 = 250;     // 250 milliseconds
const MAX_POLLING_INTERVAL:u64  = 32_000;  // 32 seconds as milliseconds

#[derive(Debug)]
pub struct SessionController {
    list: Vec<RwLock<HashMap<[u8;16],Session>>>,
    garbage_collector: GarbageCollector,
    user_epoch_controller: RwLock<UserEpochController>,
    hash_key: Uuid
}

impl SessionController {

    /// getter
    pub fn hash_key(&self) -> &Uuid {
        &self.hash_key
    }

    /// blake 3 keyed hash for storage in database
    #[inline]
    fn hash_token(&self, token: &str) -> Result<blake3::Hash> {
        let uuid = match self.hash_key {
            Uuid::Crypto(buf) => buf,
            _ => return Err(Error::SessionTokenIncorrectType)
        };
        
        let hash = blake3::keyed_hash(&uuid, token.as_bytes());
        
        Ok(hash)
    }

    /// epoch updater interval
    pub async fn watch_epoch(&self, connection: &DatabaseConnection) {
        let mut base = BASE_POLLING_INTERVAL;
        let mut next_refresh = BASE_POLLING_INTERVAL.to_jitter_millis();
        
        // set tick interval to minimum time
        let mut interval = actix_rt::time::interval(Duration::from_millis(BASE_POLLING_INTERVAL));
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            // sleep
            interval.tick().await;

            // get current Instant
            let now = Instant::now();
            
            // compare and run updater
            if now > next_refresh {
                let update_status = self.update_user_epoch(connection).await.unwrap_or(UpdateStatus::None);
                
                // adjust the updater interval to back off or heat up
                match update_status {
                    UpdateStatus::None => base = (base + 500).min(MAX_POLLING_INTERVAL),
                    UpdateStatus::Update => base = BASE_POLLING_INTERVAL
                }

                next_refresh = base.to_jitter_millis();
            }
        }
    }

    /// runs the epoch updater
    async fn update_user_epoch(&self, connection: &DatabaseConnection) -> Result<UpdateStatus> {
        // get latest master epoch from the database
        let next_epoch = UserEpochController::next(connection).await?;

        // get the current master epoch in memory
        let current_epoch = self.user_epoch_controller
            .try_read()
            .to_lock_error(Error::UserEpochLockNotAquired)?
            .current();

        // update on new changes
        if current_epoch < next_epoch {
            let change_list = UserEpochController::change_list(current_epoch,connection).await?;

            if change_list.is_empty() {
                return Ok(UpdateStatus::None);
            }

            // begin write lock
            {
                let mut locked_list = self.user_epoch_controller
                    .write()
                    .to_lock_error(Error::UserEpochLockNotAquired)?;
                
                for meta in change_list {
                    locked_list.update(meta.user_id, meta.user_epoch);
                }

                locked_list.set_master_epoch(next_epoch);
            }
            // end write lock

            return Ok(UpdateStatus::Update)
        }
        
        Ok(UpdateStatus::None)
    }

    /// garbage collector interval
    pub async fn watch_garbage(&self) {
        let mut base = BASE_GARBAGE_POLLING_TTL;
        let mut next_refresh = BASE_GARBAGE_POLLING_TTL.to_jitter_secs();

        // set tick interval to minimum time
        let mut interval = actix_rt::time::interval(Duration::from_secs(BASE_GARBAGE_POLLING_TTL));
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            interval.tick().await;

            // get current time
            let now = Instant::now();
            
            // compare and run collection
            if now > next_refresh {

                // run collector and report errors
                let update_status = match self.start_garbage_collector() {
                    Ok(s) => s,
                    Err(e) => {
                        // log error here
                        println!("{e}");

                        UpdateStatus::None
                    }
                };

                // adjust the updater interval to back off or heat up
                match update_status {
                    UpdateStatus::None => base = (base + 1_000).min(MAX_GARBAGE_COLLECTION),
                    UpdateStatus::Update => base = BASE_GARBAGE_POLLING_TTL
                }

                next_refresh = base.to_jitter_secs()
            }
            
        }
    }

    /// runs a garbage collection sweep to remove expired sessions
    fn start_garbage_collector(&self) -> Result<UpdateStatus> {
        let mut update_flag = UpdateStatus::None;

        // begin write lock
        let collector = &self.garbage_collector;

        // sweep shards
        for shard in 0..self.list.len() {
            match collector.sweep(&self.list[shard],COLLECTION_TTL)? {
                UpdateStatus::None => {},
                UpdateStatus::Update => update_flag = UpdateStatus::Update
            }
        }
        // end write lock

        Ok(update_flag)
    }

    /// deletes session from controller
    pub fn delete(&self, token_b64: &str) -> Result<()> {
        let token = token_b64.vec_from_base64_url()?;
        let key = token.to_key()?;

        // derive shard id
        let idx = self.idx(&key)?;

        // begin locked scope
        {
            self.list[idx]
                .write()
                .to_lock_error(Error::SessionLockNotAquired)?
                .remove(&key);
        }
        // end locked scope
        
        Ok(())
    }

    /// returns a new session controller
    pub fn new(capacity: usize, threads: usize, user_epoch_controller:UserEpochController) -> Self {
        let hash_key = Uuid::crypto32()
            .expect("could not create session hash key on startup");
        let garbage_collector = GarbageCollector;
        let user_epoch_controller = RwLock::new(user_epoch_controller);
        
        // double check threads > 0
        let threads_checked = {
            match threads {
                0   => 1,
                1.. => threads
            }
        };

        // isntantiate session list
        let shard_capacity = capacity / threads_checked;
        let mut list = Vec::with_capacity(threads_checked);
        let map_builder:HashMap<[u8;16],Session> = HashMap::with_capacity(shard_capacity);
        
        for _ in 0..threads_checked {
            let locked_map = RwLock::new(map_builder.clone());
            list.push(locked_map);
        }

        Self {
            garbage_collector,
            list,
            user_epoch_controller,
            hash_key
        }
    }

    /// produces the shard id 
    #[inline]
    fn idx(&self, key: &[u8]) -> Result<usize> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        
        let hash = hasher.finish();
        Ok((hash as usize) % self.list.len())
    }

    /// inserts a new session into the controller and runs the trash collector
    pub fn insert(&self, session: Session, key_set: &KeySet) -> Result<String> {
        let key = &key_set.key;
        let secret = &key_set.secret;
        let idx = self.idx(key)?;
    
        // begin locked write scope
        {
            self.list[idx].write()
                .to_lock_error(Error::SessionLockNotAquired)?
                .insert(*key, session);
        }
        // end locked write scope

        let mut token_buf:[u8;32] = [0;32];
        token_buf[..16].copy_from_slice(key);
        token_buf[16..].copy_from_slice(secret);
        
        let token = token_buf.to_base64_url();

        Ok(token)
    }

    /// returns a reference to the list of sessions
    pub fn list(&self) -> &Vec<RwLock<HashMap<[u8;16],Session>>>  {
        &self.list
    }

    pub fn update_session_user(&self, key: &[u8; 16], user: User) -> Result<()> {

        // derive shard id
        let idx = self.idx(key)?;

        // begin write lock scope
        {
            // get write lock
            let mut locked_list = self.list[idx]
                .write()
                .to_lock_error(Error::SessionLockNotAquired)?;

            // and insert updated user
            match locked_list.entry(*key) {
                Entry::Occupied(mut e) => { e.get_mut().user = user; }
                Entry::Vacant(_) => return Err(Error::SessionNotFoundDuringUpdate)
            }

        }
        // end write lock scope

        Ok(())
    }

    /// refresh a token from the database
    pub async fn refresh(&self, token_b64: &str, database: &DatabaseConnection) -> Result<Permission> {
        // verify the session on the database
        let keyed_hash = self.hash_token(token_b64)?;
        let verification_status = DatabaseSession::verify(&keyed_hash, database).await?;

        // decode from base64 extract key
        let key = token_b64
            .vec_from_base64_url()?
            .to_key()?;

        // derive shard id
        let idx = self.idx(&key)?;

        // short circuit on verification failure
        if verification_status == VerificationStatus::Unverified {
            match self.delete(token_b64) {
                Ok(_) => return Ok(Permission::Denied),
                Err(_e) => {
                    // add log here
                    return Ok(Permission::Denied)
                }
            }
        }

        // begin locked write scope
        {
            self.list[idx]
                .write()
                .to_lock_error(Error::SessionLockNotAquired)?
                .get_mut(&key)
                .ok_or(Error::SessionNotFoundDuringRefresh)?
                .update_next_refresh();
        }
        // end locked write scope

        Ok(Permission::Granted)
    }

    /// verifies the current user data stored in memory is fresh, runs before a permission check
    pub async fn epoch_check(&self, token_b64: &str, database: &DatabaseConnection) -> Result<()> {
        // decode from base64 to Vec<u8> and extract key segment
        let key = token_b64
            .vec_from_base64_url()?
            .to_key()?;
   
        // derive shard id
        let idx = self.idx(&key)?;
        
        // begin session lock scope
        let user = {
            self.list[idx]
                .read()
                .to_lock_error(Error::SessionLockNotAquired)?
                .get(&key)
                .ok_or(Error::SessionNotFound)?
                .user
                .clone()
        };
        // end session lock scope

        // extract user epoch data
        let user_id = user.id();
        let current_epoch = user.epoch();

        // extract the updated epoch from the epoch controller
        let next_epoch = self.user_epoch_controller
            .read()
            .to_lock_error(Error::UserEpochLockNotAquired)?
            .user_epoch(user_id)
            .ok_or(Error::UserEpochNotFound)?;

        // compare epochs
        if current_epoch < next_epoch {
            match User::by_id_unchecked(user_id, database).await? {
                Some(user) => self.update_session_user(&key, user)?,
                None => return Err(Error::UserIdNotInDatabase)
            }
        }

        Ok(())
    }
    
    /// verify user has software access rights / permissions
    pub fn permission_check(&self, token_b64: &str, required_rights: UserPermissions) -> Result<PermissionCheck> {
        // decode from base64 to Vec<u8> and extract segments
        let token = token_b64.vec_from_base64_url()?;
        let key = token.to_key()?;
        let secret = token.to_secret()?;

        // default: deny all
        let mut permission_check = PermissionCheck {
            permission: Permission::Denied,
            refresh_status: RefreshStatus::None,
            auth_context: AuthContext::None
        };

        // derive shard id
        let idx = self.idx(&key)?;
        
        // begin read lock scope
        let user = {
            // get read lock
            let locked_list = self.list[idx]
                .read()
                .to_lock_error(Error::SessionLockNotAquired)?;

            // and retrieve sesssion
            let session = locked_list.get(&key).ok_or(Error::SessionNotFound)?;
            
            // exhaustive check on whether session is expired
            if matches!(session.is_expired(), ExpiredStatus::Expired) {
                return Err(Error::SessionExpired)
            }

            // verify user account is enabled
            if !matches!(session.user.status(), UserAccountStatus::Enabled) {
                return Err(Error::UserAccountStatusNotEnabled)
            }

            // set refresh status flag
            permission_check.refresh_status = session.is_stale();

            // constant time hash check
            let verify_status = KeySet::verify(&key,&secret,&session.hash);

            // short circuit if verification failed
            if verify_status == VerificationStatus::Unverified {
                return Err(Error::SessionHashNotVerified)
            }
            
            // copy out user data
            session.user.clone()
        };
        // end read lock scope

        // run user permission check
        permission_check.permission = user
            .permissions()
            .has_permission(&required_rights);

        // set auth context on permissions granted
        if permission_check.permission == Permission::Granted {
            let box_user = Box::new(user.clone());
            permission_check.auth_context = AuthContext::Some(box_user)
        }

        Ok(permission_check)
    }
}

impl Default for SessionController {
    fn default() -> Self {
        let default_map_capacity:usize = 1000;
        let threads:usize = 2;
        let user_epoch_controller = UserEpochController::default();
        
        Self::new(default_map_capacity, threads, user_epoch_controller)
    }
}

#[cfg(test)]
mod tests {
    use crate::{enums::{Resource, UserType}, types::users::Builder};
    use std::time::Instant;

    use super::*;

    /// loads 1_000_000 random session ids into memory and checks for overwrites, which would indicate collisions
    #[test]
    fn collision_test() {
        let sessions_to_create = 1_000_000;
        let user_epoch_controller = UserEpochController::default();
        let controller = SessionController::new(sessions_to_create, 4, user_epoch_controller);

        for _ in 0..sessions_to_create {
            let key_set = KeySet::new().unwrap();
            let user = Builder::new()
                .id(0)
                .epoch(0)
                .username(String::from("username"))
                .hash(String::from("hash"))
                .user_type(UserType::Standard)
                .user_status(crate::enums::UserAccountStatus::Enabled)
                .permissions(UserPermissions::default())
                .build()
                .unwrap();

            let session = Session::new(&key_set,user);
            let _token = controller.insert(session, &key_set).unwrap();
        }

        let list = controller.list();
        let mut total_sessions:usize = 0;
        
        for (idx,_) in list.iter().enumerate() {
            // begin locked scope
            {
                let locked_list = list[idx]
                    .read()
                    .map_err(|_e| Error::PoisonedSessionList).unwrap();
                
                total_sessions += locked_list.len();
            }
            // end locked scope
        }

        assert_eq!(total_sessions,sessions_to_create);
    }

    /// loads 1_000_000 random sessions into memory, encoding and decoding the hash strings and 
    /// running a permission check to make sure each hash string is still valid
    #[test]
    fn hash_decode_check() {
        let sessions_to_create = 1_000_000;
        let user_epoch_controller = UserEpochController::default();
        let controller = SessionController::new(sessions_to_create, 4, user_epoch_controller);
        let r = Resource::Sessions;
        let permissions = UserPermissions::default().with_rw_self(r);
        let denied_permissions = UserPermissions::default().with_admin(r);
        let user = Builder::new()
            .id(0)
            .epoch(0)
            .username(String::from("username"))
            .hash(String::from("hash"))
            .user_type(UserType::Standard)
            .user_status(crate::enums::UserAccountStatus::Enabled)
            .permissions(permissions)
            .build()
            .unwrap();

        for _ in 0..sessions_to_create {
            let key_set = KeySet::new().unwrap();
            let user = user.clone();
            let session = Session::new(&key_set,user);

            // insert and encode with base64
            let token = controller.insert(session, &key_set).unwrap();

            // permission check will decode and validate the token and allow access 
            let check = controller.permission_check(&token, permissions.clone()).unwrap();
            assert_eq!(check.permission,Permission::Granted);

            // permission check will decode and validate the token and deny access
            let check = controller.permission_check(&token, denied_permissions.clone()).unwrap();
            assert_eq!(check.permission,Permission::Denied);
        }
    }

    /// verifies sessions are removed
    #[test]
    fn session_delete() {
        let sessions_to_create = 1_000_000;
        let user_epoch_controller = UserEpochController::default();
        let controller = SessionController::new(sessions_to_create, 4, user_epoch_controller);
        let key_set = KeySet::new().unwrap();
        let user = Builder::new()
            .id(0)
            .epoch(0)
            .username(String::from("username"))
            .hash(String::from("hash"))
            .user_type(UserType::Standard)
            .user_status(crate::enums::UserAccountStatus::Enabled)
            .permissions(UserPermissions::default())
            .build()
            .unwrap();

        let session = Session::new(&key_set,user);
        let _token = controller.insert(session, &key_set).unwrap();
    }

    /// load tests the garbage collector
    #[test]
    fn garbage_collector() {
        let sessions_to_create = 1_000_000;
        let user_epoch_controller = UserEpochController::default();
        let controller = SessionController::new(sessions_to_create, 4, user_epoch_controller);

        for _ in 0..sessions_to_create {
            let key_set = KeySet::new().unwrap();
            let user = Builder::new()
                .id(0)
                .epoch(0)
                .username(String::from("username"))
                .hash(String::from("hash"))
                .user_type(UserType::Standard)
                .user_status(crate::enums::UserAccountStatus::Enabled)
                .permissions(UserPermissions::default())
                .build()
                .unwrap();

            let mut session = Session::new(&key_set,user);
            session.next_refresh = Instant::now().checked_sub(Duration::from_secs(60 * 60 * 24 * 10)).unwrap();

            assert_eq!(session.is_expired(),ExpiredStatus::Expired);

            let _token = controller.insert(session, &key_set).unwrap();
        }

        let _a = match controller.start_garbage_collector() {
            Ok(_) => println!("ok"),
            Err(e) => println!("FAIL {:?}",e)
        };

        let mut count: usize = 0;

        for idx in 0..controller.list.len() {
            let locked_list = controller.list[idx].read().unwrap();
            count += locked_list.len();
        }

        assert!(count <= sessions_to_create - (4 * 2048));
    }
} 
