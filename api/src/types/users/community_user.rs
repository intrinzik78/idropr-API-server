use crate::{
    enums::UserAccountStatus,
    types::SoftwareAccess
};

#[derive(Clone,Debug,PartialEq)]
pub struct CommunityUser {
    pub id: i64,
    pub username: Option<String>,
    pub hash: String,
    pub status: UserAccountStatus,
    pub software_access: SoftwareAccess
}

impl CommunityUser {

}