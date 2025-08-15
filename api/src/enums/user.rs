use crate::types::users::{BusinessUser,CommunityUser,SystemUser};

#[derive(Clone,Debug,PartialEq)]
pub enum User {
    System(SystemUser),
    Business(BusinessUser),
    Community(CommunityUser)
}