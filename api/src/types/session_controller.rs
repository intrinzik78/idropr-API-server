use std::{collections::HashMap, hash::{DefaultHasher,Hash,Hasher}, sync::Mutex};

use crate::{
    enums::{Error, Permission, User, VerificationStatus}, traits::HasPermission, types::{KeySet, Session, SoftwareAccess}
};

type Result<T> = std::result::Result<T,Error>;

#[derive(Debug)]
pub struct SessionController {
    list: Vec<Mutex<HashMap<[u8;16],Session>>>
}

impl SessionController {

    /// returns a new session controller
    pub fn new(capacity: usize, threads: usize) -> Self {
        let mut list = Vec::with_capacity(threads);
        let map_builder:HashMap<[u8;16],Session> = HashMap::with_capacity(capacity);
        
        for _ in 0..threads {
            let locked_map = Mutex::new(map_builder.clone());
            list.push(locked_map);
        }

        SessionController {
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

    /// inserts a new session into the controller
    pub fn insert(&self, session: Session, key_set: &KeySet) -> Result<()> {
        let key = key_set.key;
        let idx = self.idx(&key)?;

        // begin locked scope
        {
            let mut locked_list = self.list[idx]
                .lock()
                .map_err(|_e| Error::PosionedSessionList)?;
            
            let _ = locked_list.insert(key, session);
        }
        // end locked scope

        Ok(())
    }

    /// returns a reference to the list of sessions
    pub fn list(&self) -> &Vec<Mutex<HashMap<[u8;16],Session>>>  {
        &self.list
    }

    /// verify user has software access rights / permissions
    pub fn permission_check(&self, token: &str, required_rights: &SoftwareAccess) -> Result<Permission> {
        let token = token.as_bytes();

        // length check
        let () = match token.len() {
            ..32 => return Err(Error::SessionTokenLengthTooShort),
            32  => {},
            33.. => return Err(Error::SessionTokenLengthTooLong)
        };

        let key = &token[..16];
        let secret = &token[16..];

        // extract key
        let mut key_buf:[u8;16] = [0;16];
        key_buf.copy_from_slice(key);
        
        // extract secret
        let mut secret_buf:[u8;16] = [0;16];
        secret_buf.copy_from_slice(secret);

        // derive shard id
        let idx = self.idx(key)?;
        
        // begin locked scope
        let permission_result = {
            let locked_list = self.list[idx]
                .lock()  
                .map_err(|_e| Error::PosionedSessionList)?;

            match locked_list.get(key) {
                Some(session) => {
                    // user permission check
                    let permission = match &session.user {
                        User::Business(u) => u.software_access.has_permission(required_rights),
                        User::Community(c) => c.software_access.has_permission(required_rights),
                        User::System(s) => s.software_access.has_permission(required_rights)
                    };

                    // verify hash and return the permission check
                    match KeySet::verify(&key_buf,&secret_buf,&session.hash) {
                        VerificationStatus::Verified => permission,
                        VerificationStatus::Unverified => Permission::None
                    }
                },
                None => Permission::None // session key did not exist in shard list
            }
        };
        // end locked scope

        Ok(permission_result)
    }

}

impl Default for SessionController {
    fn default() -> Self {
        let default_map_capacity:usize = 1000;
        let threads:usize = 2;
        
        SessionController::new(default_map_capacity, threads)
    }
}

#[cfg(test)]
mod tests {
    use crate::enums::UserType;

    use super::*;

    #[test]
    fn collision_test() {
        let sessions_to_create = 1_000_000;
        let controller = SessionController::new(sessions_to_create, 4);

        for _ in 0..sessions_to_create {
            let key_set = KeySet::new().unwrap();
            let user_type = UserType::System;
            let session = Session::new(&key_set,user_type);
            controller.insert(session, &key_set).unwrap();
        }

        let list = controller.list();
        let mut total_sessions:usize = 0;
        
        for (idx,_) in list.iter().enumerate() {
            // begin locked scope
            {
                let locked_list = list[idx]
                    .lock()
                    .map_err(|_e| Error::PosionedSessionList).unwrap();
                
                total_sessions += locked_list.len();
            }
            // end locked scope
        }

        assert_eq!(total_sessions,sessions_to_create);
    }
}
