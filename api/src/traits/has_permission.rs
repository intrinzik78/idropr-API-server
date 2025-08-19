// a fast and efficient way to determine whether a user has the appropriate permissions set to access an API endpoint
use crate::{
    enums::Permission,
    types::UserPermissions
};

// creates bit flags for each permission toggle
const BUCKETS_READ:u16      = 0b0000_0000_0001;
const BUCKETS_WRITE:u16     = 0b0000_0000_0010;
const BUCKETS_DELETE:u16    = 0b0000_0000_0100;
const IMAGES_READ:u16       = 0b0000_0000_1000;
const IMAGES_WRITE:u16      = 0b0000_0001_0000;
const IMAGES_DELETE:u16     = 0b0000_0010_0000;
const SESSIONS_READ:u16     = 0b0000_0100_0000;
const SESSIONS_WRITE:u16    = 0b0000_1000_0000;
const SESSIONS_DELETE:u16   = 0b0001_0000_0000;
const USERS_READ:u16        = 0b0010_0000_0000;
const USERS_WRITE:u16       = 0b0100_0000_0000;
const USERS_DELETE:u16      = 0b1000_0000_0000;

// single method called on a user's SoftwareAccess struct. 
// Required rights are passed into the method where a comparison is made and Pass/Fail result returned.
pub trait HasPermission {
    fn has_permission(self, required_rights: &UserPermissions) -> Permission;
}

impl HasPermission for &UserPermissions {
    fn has_permission(self, required_rights: &UserPermissions) -> Permission {
        let mut user_flags = 0_u16;

        if self.buckets_read == Permission::Granted     { user_flags |= BUCKETS_READ; }
        if self.buckets_write == Permission::Granted    { user_flags |= BUCKETS_WRITE; }
        if self.buckets_delete == Permission::Granted   { user_flags |= BUCKETS_DELETE; }
        if self.images_read == Permission::Granted      { user_flags |= IMAGES_READ; }
        if self.images_write == Permission::Granted     { user_flags |= IMAGES_WRITE; }
        if self.images_delete == Permission::Granted    { user_flags |= IMAGES_DELETE; }
        if self.sessions_read == Permission::Granted    { user_flags |= SESSIONS_READ; }
        if self.sessions_write == Permission::Granted   { user_flags |= SESSIONS_WRITE }
        if self.sessions_delete == Permission::Granted  { user_flags |= SESSIONS_DELETE; }
        if self.users_read == Permission::Granted       { user_flags |= USERS_READ; }
        if self.users_write == Permission::Granted      { user_flags |= USERS_WRITE; }
        if self.users_delete == Permission::Granted     { user_flags |= USERS_DELETE; }

        let mut required_flags = 0_u16;

        if required_rights.buckets_read == Permission::Granted     { required_flags |= BUCKETS_READ; }
        if required_rights.buckets_write == Permission::Granted    { required_flags |= BUCKETS_WRITE; }
        if required_rights.buckets_delete == Permission::Granted   { required_flags |= BUCKETS_DELETE; }
        if required_rights.images_read == Permission::Granted      { required_flags |= IMAGES_READ; }
        if required_rights.images_write == Permission::Granted     { required_flags |= IMAGES_WRITE; }
        if required_rights.images_delete == Permission::Granted    { required_flags |= IMAGES_DELETE; }
        if required_rights.sessions_read == Permission::Granted    { required_flags |= SESSIONS_READ; }
        if required_rights.sessions_write == Permission::Granted   { required_flags |= SESSIONS_WRITE }
        if required_rights.sessions_delete == Permission::Granted  { required_flags |= SESSIONS_DELETE; }
        if required_rights.users_read == Permission::Granted       { required_flags |= USERS_READ; }
        if required_rights.users_write == Permission::Granted      { required_flags |= USERS_WRITE; }
        if required_rights.users_delete == Permission::Granted     { required_flags |= USERS_DELETE; }

        // bitwise comparison
        if required_flags & user_flags == required_flags {
            Permission::Granted
        } else {
            Permission::None
        }
    }
}

#[cfg(test)]
pub mod test {
    // use super::*;

    #[test]
    // tests bit-flag check for granting permissions
    fn account_permissions() {
        // creates test users
        // let user_with_no_rights = SoftwareAccess::default();
        // let user_with_all_rights = SoftwareAccess::default()
        //     .with_accounts()
        //     .with_api()
        //     .with_bookings()
        //     .with_clients()
        //     .with_leads()
        //     .with_reports()
        //     .with_sales()
        //     .with_surveys()
        //     .with_users();

        // creates permission requirements to test users against
        // let user_accounts = SoftwareAccess::default().with_accounts();
        // let user_api = SoftwareAccess::default().with_api();
        // let user_users = SoftwareAccess::default().with_users();
        // let user_bookings = SoftwareAccess::default().with_bookings();
        // let user_clients = SoftwareAccess::default().with_clients();
        // let user_leads = SoftwareAccess::default().with_leads();
        // let user_surveys = SoftwareAccess::default().with_surveys();
        // let user_reports = SoftwareAccess::default().with_reports();
        // let user_sales = SoftwareAccess::default().with_sales();

        //fail case
        // assert_eq!(user_with_no_rights.has_permission(&user_accounts), Permission::None);
        // assert_eq!(user_with_no_rights.has_permission(&user_api), Permission::None);
        // assert_eq!(user_with_no_rights.has_permission(&user_users), Permission::None);
        // assert_eq!(user_with_no_rights.has_permission(&user_bookings), Permission::None);
        // assert_eq!(user_with_no_rights.has_permission(&user_clients), Permission::None);
        // assert_eq!(user_with_no_rights.has_permission(&user_leads), Permission::None);
        // assert_eq!(user_with_no_rights.has_permission(&user_surveys), Permission::None);
        // assert_eq!(user_with_no_rights.has_permission(&user_reports), Permission::None);
        // assert_eq!(user_with_no_rights.has_permission(&user_sales), Permission::None);

        //success case
        // assert_eq!(user_with_all_rights.has_permission(&user_accounts), Permission::Granted);
        // assert_eq!(user_with_all_rights.has_permission(&user_api), Permission::Granted);
        // assert_eq!(user_with_all_rights.has_permission(&user_users), Permission::Granted);
        // assert_eq!(user_with_all_rights.has_permission(&user_bookings), Permission::Granted);
        // assert_eq!(user_with_all_rights.has_permission(&user_clients), Permission::Granted);
        // assert_eq!(user_with_all_rights.has_permission(&user_leads), Permission::Granted);
        // assert_eq!(user_with_all_rights.has_permission(&user_surveys), Permission::Granted);
        // assert_eq!(user_with_all_rights.has_permission(&user_reports), Permission::Granted);
        // assert_eq!(user_with_all_rights.has_permission(&user_sales), Permission::Granted);
    
    }
}