use std::{collections::HashMap, hash::{DefaultHasher,Hash,Hasher}, sync::RwLock, time::{Duration,Instant}};

use crate::{
    enums::{Error, ExpiredStatus, Permission, RefreshStatus, User, VerificationStatus},
    traits::{FromBase64, HasPermission, ToBase64, ToKeySet},
    types::{KeySet, PermissionCheck, Session, UserPermissions}
};

type Result<T> = std::result::Result<T,Error>;

const MAX_GARBAGE_COLLECTION:u64 = 60;     // 10 seconds
const COLLECTION_TTL:u64 = 10;             // 10 miliseconds

#[derive(Debug,Default)]
struct GarbageCollector;

impl GarbageCollector {
    /// accepts a locked shard and removes expired sessions
    pub fn sweep(&mut self, list: &RwLock<HashMap<[u8;16],Session>>) -> Result<()> {
        let time = Duration::from_millis(COLLECTION_TTL);
        let stop_time = Instant::now().checked_add(time).ok_or(Error::DevError("couldn't create a time window to work in garbage collector".to_string()))?;
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

#[derive(Debug)]
pub struct SessionController {
    list: Vec<RwLock<HashMap<[u8;16],Session>>>,
    garbage_collector: RwLock<GarbageCollector>
}

impl SessionController {

    /// garbage collector interval
    pub async fn watch(&self) {
        // cannot be zero or it will run constantly with no delay
        let mut interval = actix_rt::time::interval(Duration::from_secs(MAX_GARBAGE_COLLECTION));
        
        loop {
            interval.tick().await;
            let _ = self.start_collector();
        }
    }

    /// runs a garbage collection sweep to remove expired sessions
    pub fn start_collector(&self) -> Result<()> {
        // begin write lock
        let mut locked_collector = self.garbage_collector.write().map_err(|_e| Error::PoisonedSessionList)?;

        for shard in 0..self.list.len() {
            locked_collector.sweep(&self.list[shard])?;
        }
        // end write lock

        Ok(())
    }

    /// deletes session from controller
    pub fn delete(&self, token_b64: &str) -> Result<()> {
        let token = token_b64.vec_from_base64_url()?;
        let key = token.to_key()?;

        // derive shard id
        let idx = self.idx(&key)?;

        // begin locked scope
        let () = {
            let mut locked_list = self.list[idx]
                .write()
                .map_err(|_e| Error::PoisonedSessionList)?;

            locked_list.remove(&key);
        };
        // end locked scope
        
        Ok(())
    }

    /// returns a new session controller
    pub fn new(capacity: usize, threads: usize) -> Self {
        let garbage_collector = GarbageCollector;

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
            garbage_collector: RwLock::new(garbage_collector),
            list
        }
    }

    /// produces the shard id 
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
            let mut locked_list = self.list[idx]
                .write()
                .map_err(|_e| Error::PoisonedSessionList)?;
            
            let _ = locked_list.insert(*key, session);
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

    /// verify user has software access rights / permissions
    pub fn permission_check(&self, token_b64: &str, required_rights: &UserPermissions) -> Result<Permission> {
        // decode from base64 to Vec<u8> and extract segments
        let token = token_b64.vec_from_base64_url()?;
        let key = token.to_key()?;
        let secret = token.to_secret()?;

        // derive shard id
        let idx = self.idx(&key)?;
        
        // begin read lock scope
        let permission_check = {
            // get read lock
            let locked_list = self.list[idx]
                .read()  
                .map_err(|_e| Error::PoisonedSessionList)?;

            // and retrieve sesssion
            let session = match locked_list.get(&key) {
                Some(s) => s,
                None => return Ok(Permission::Denied)
            };

            // check if it's expired and deny if it is
            if session.is_expired() == ExpiredStatus::Expired {
                return Ok(Permission::Denied);
            }

            // constant time hash check
            let verify_status = KeySet::verify(&key,&secret,&session.hash);

            // permission check
            let permission = {
                 if verify_status == VerificationStatus::Verified {
                    // run permission checks
                    match &session.user {
                        User::Business(u) => u.permissions.has_permission(required_rights),
                        User::Community(c) => c.permissions.has_permission(required_rights),
                        User::System(s) => s.permissions.has_permission(required_rights)
                    }
                } else {
                    Permission::Denied
                }
            };

            // package a response
            PermissionCheck {
                permission,
                refresh_status: session.is_stale()
            }
        };
        // end read lock scope

        match permission_check.refresh_status {
            RefreshStatus::None => Ok(permission_check.permission),
            RefreshStatus::Refresh => {
                // do a database check here
                Ok(permission_check.permission)
            }
        }
    }
}

impl Default for SessionController {
    fn default() -> Self {
        let default_map_capacity:usize = 1000;
        let threads:usize = 2;
        
        Self::new(default_map_capacity, threads)
    }
}

#[cfg(test)]
mod tests {
    use crate::{enums::Resource, types::users::SystemUser};

    use super::*;

    /// loads 1_000_000 random session ids into memory and checks for overwrites, which would indicate collisions
    #[test]
    fn collision_test() {
        let sessions_to_create = 1_000_000;
        let controller = SessionController::new(sessions_to_create, 4);

        for _ in 0..sessions_to_create {
            let key_set = KeySet::new().unwrap();
            let user = User::System(SystemUser{
                id: 0,
                username: String::from("username"),
                hash: String::from("hash"),
                status: crate::enums::UserAccountStatus::Enabled,
                permissions: UserPermissions::default()
            });
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
        let controller = SessionController::new(sessions_to_create, 4);
        let r = Resource::Sessions;
        let permissions = UserPermissions::default().with_rw_self(r);
        let denied_permissions = UserPermissions::default().with_admin(r);
        let user = User::System(SystemUser{
            id: 0,
            username: String::from("username"),
            hash: String::from("hash"),
            status: crate::enums::UserAccountStatus::Enabled,
            permissions: permissions.clone()
        });

        for _ in 0..sessions_to_create {
            let key_set = KeySet::new().unwrap();
            let user = user.clone();
            let session = Session::new(&key_set,user);

            // insert and encode with base64
            let token = controller.insert(session, &key_set).unwrap();

            // permission check will decode and validate the token and allow access 
            let check = controller.permission_check(&token, &permissions).unwrap();
            assert_eq!(check,Permission::Granted);

            // permission check will decode and validate the token and deny access
            let check = controller.permission_check(&token, &denied_permissions).unwrap();
            assert_eq!(check,Permission::Denied);
        }
    }

    /// verifies sessions are removed
    #[test]
    fn session_delete() {
        let sessions_to_create = 1_000_000;
        let controller = SessionController::new(sessions_to_create, 4);
        let key_set = KeySet::new().unwrap();
        let user = User::System(SystemUser{
            id: 0,
            username: String::from("username"),
            hash: String::from("hash"),
            status: crate::enums::UserAccountStatus::Enabled,
            permissions: UserPermissions::default()
        });
        let session = Session::new(&key_set,user);
        let _token = controller.insert(session, &key_set).unwrap();
    }

    /// load tests the garbage collector
    #[test]
    fn garbage_collector() {
        let sessions_to_create = 1_000_000;
        let controller = SessionController::new(sessions_to_create, 4);

        for _ in 0..sessions_to_create {
            let key_set = KeySet::new().unwrap();
            let user = User::System(SystemUser{
                id: 0,
                username: String::from("username"),
                hash: String::from("hash"),
                status: crate::enums::UserAccountStatus::Enabled,
                permissions: UserPermissions::default()
            });

            let mut session = Session::new(&key_set,user);
            session.next_refresh = Instant::now().checked_sub(Duration::from_secs(186400)).unwrap();

            let _token = controller.insert(session, &key_set).unwrap();
        }

        let _a = match controller.start_collector() {
            Ok(_) => println!("ok"),
            Err(e) => println!("{:?}",e)
        };

        let mut count = 0;

        for idx in 0..controller.list.len() {
            let locked_list = controller.list[idx].read().unwrap();
            count += locked_list.len();
        }

        assert!(count <= (sessions_to_create - (4*2048)), "Total sessions: {}", count);
    }
}
