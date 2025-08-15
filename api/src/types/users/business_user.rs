use crate::{
    enums::UserAccountStatus,
    types::SoftwareAccess
};

#[derive(Clone,Debug,PartialEq)]
pub struct BusinessUser {
    pub id: i64,
    pub business_id: i64,
    pub username: Option<String>,
    pub hash: String,
    pub status: UserAccountStatus,
    pub software_access: SoftwareAccess
}

impl BusinessUser {

}