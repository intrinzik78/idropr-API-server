use blake3::Hash;
use std::time::Instant;

use crate::{
    enums::{User, UserAccountStatus, UserType},
    types::{users::{BusinessUser, CommunityUser, SystemUser}, KeySet, SoftwareAccess}
};

#[derive(Clone,Debug)]
pub struct Session {
    pub hash: Hash,
    pub last_access: Instant,
    pub user: User
}

impl Session {

    /// creates a new session container
    pub fn new(key_set: &KeySet, user_type: UserType) -> Self {
        let last_access = Instant::now();
        let user = match user_type {
            UserType::System => {
                User::System(SystemUser {
                    id: 0,
                    username: None,
                    hash: String::from("abc"),
                    status: UserAccountStatus::Enabled,
                    software_access: SoftwareAccess::default().with_accounts()
                })
            },
            UserType::Community => {
                User::Community(CommunityUser {
                    id: 0,
                    username: None,
                    hash: String::from("abc"),
                    status: UserAccountStatus::Enabled,
                    software_access: SoftwareAccess::default().with_accounts()
                })
            },
            UserType::Business => {
                User::Business(BusinessUser {
                    id: 0,
                    business_id: 0,
                    username: None,
                    hash: String::from("abc"),
                    status: UserAccountStatus::Enabled,
                    software_access: SoftwareAccess::default().with_accounts()
                })
            },
        };

        Session {
            hash: key_set.hash,
            last_access,
            user
        }
    }
}