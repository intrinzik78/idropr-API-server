// a fast and efficient way to determine whether a user has the appropriate permissions set to access an API endpoint
use crate::{
    enums::Permission,
    types::UserPermissions
};

// creates bit flags for each permission toggle
const ADMIN_READ:u16        = 0b0000_0000_0000_0001;
const ADMIN_WRITE:u16       = 0b0000_0000_0000_0010;
const ADMIN_DELETE:u16      = 0b0000_0000_0000_0100;

const BUCKETS_READ:u16      = 0b0000_0000_0000_1000;
const BUCKETS_WRITE:u16     = 0b0000_0000_0001_0000;
const BUCKETS_DELETE:u16    = 0b0000_0000_0010_0000;

const IMAGES_READ:u16       = 0b0000_0000_0100_0000;
const IMAGES_WRITE:u16      = 0b0000_0000_1000_0000;
const IMAGES_DELETE:u16     = 0b0000_0001_0000_0000;

const SESSIONS_READ:u16     = 0b0000_0010_0000_0000;
const SESSIONS_WRITE:u16    = 0b0000_0100_0000_0000;
const SESSIONS_DELETE:u16   = 0b0000_1000_0000_0000;

const USERS_READ:u16        = 0b0001_0000_0000_0000;
const USERS_WRITE:u16       = 0b0010_0000_0000_0000;
const USERS_DELETE:u16      = 0b0100_0000_0000_0000;

// single method called on a user's SoftwareAccess struct. 
// Required rights are passed into the method where a comparison is made and Pass/Fail result returned.
pub trait HasPermission {
    fn has_permission(self, required_rights: &UserPermissions) -> Permission;
}

impl HasPermission for &UserPermissions {
    fn has_permission(self, required_rights: &UserPermissions) -> Permission {
        let mut user_flags = 0_u16;

        if self.admin_read == Permission::Granted       { user_flags |= ADMIN_READ; }
        if self.admin_write == Permission::Granted      { user_flags |= ADMIN_WRITE; }
        if self.admin_delete == Permission::Granted     { user_flags |= ADMIN_DELETE; }
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
        if required_rights.admin_read == Permission::Granted       { required_flags |= ADMIN_READ; }
        if required_rights.admin_write == Permission::Granted      { required_flags |= ADMIN_WRITE; }
        if required_rights.admin_delete == Permission::Granted     { required_flags |= ADMIN_DELETE; }
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
    use super::*;

    #[test]
    // tests bit-flag check for granting permissions
    fn account_permissions() {
        let mut test_permissions = 0b0_u16;
        let full_permissions: u16 = 0b0111_1111_1111_1111;
        test_permissions |= ADMIN_READ;
        test_permissions |= ADMIN_WRITE;
        test_permissions |= ADMIN_DELETE;
        test_permissions |= BUCKETS_READ;
        test_permissions |= BUCKETS_WRITE;
        test_permissions |= BUCKETS_DELETE;
        test_permissions |= IMAGES_READ;
        test_permissions |= IMAGES_WRITE;
        test_permissions |= IMAGES_DELETE;
        test_permissions |= SESSIONS_READ;
        test_permissions |= SESSIONS_WRITE;
        test_permissions |= SESSIONS_DELETE;
        test_permissions |= USERS_READ;
        test_permissions |= USERS_WRITE;
        test_permissions |= USERS_DELETE;

        assert_eq!(test_permissions,full_permissions);

        // creates test users
        let user_with_no_rights = UserPermissions::default();
        let user_with_all_rights = UserPermissions::default()
            .with_admin_read()
            .with_admin_write()
            .with_admin_delete()
            .with_buckets_read()
            .with_buckets_write()
            .with_buckets_delete()
            .with_images_read()
            .with_images_write()
            .with_images_delete()
            .with_sessions_read()
            .with_sessions_write()
            .with_sessions_delete()
            .with_users_read()
            .with_users_write()
            .with_users_delete();

        assert_eq!(user_with_no_rights.has_permission(&user_with_all_rights), Permission::None);
        assert_eq!(user_with_no_rights.has_permission(&user_with_no_rights), Permission::Granted);

        // full admin permissions
        let admin_user = UserPermissions::default()
            .with_admin_read()
            .with_admin_write()
            .with_admin_delete();
        
        assert_eq!(user_with_no_rights.has_permission(&admin_user), Permission::None);
        assert_eq!(user_with_all_rights.has_permission(&admin_user), Permission::Granted);

        // full buckets permissions
        let buckets_user = UserPermissions::default()
            .with_buckets_read()
            .with_buckets_write()
            .with_buckets_delete();
        
        assert_eq!(user_with_no_rights.has_permission(&buckets_user), Permission::None);
        assert_eq!(user_with_all_rights.has_permission(&buckets_user), Permission::Granted);

        // full images permissions
        let images_user = UserPermissions::default()
            .with_images_read()
            .with_images_write()
            .with_images_delete();
        
        assert_eq!(user_with_no_rights.has_permission(&images_user), Permission::None);
        assert_eq!(user_with_all_rights.has_permission(&images_user), Permission::Granted);

        // full sessions permissions
        let sessions_user = UserPermissions::default()
            .with_sessions_read()
            .with_sessions_write()
            .with_sessions_delete();
        
        assert_eq!(user_with_no_rights.has_permission(&sessions_user), Permission::None);
        assert_eq!(user_with_all_rights.has_permission(&sessions_user), Permission::Granted);

        // full users permissions
        let users_user = UserPermissions::default()
            .with_users_read()
            .with_users_write()
            .with_users_delete();
        
        assert_eq!(user_with_no_rights.has_permission(&users_user), Permission::None);
        assert_eq!(user_with_all_rights.has_permission(&users_user), Permission::Granted);
        
        //fail case
        assert_eq!(user_with_no_rights.has_permission(&admin_user), Permission::None);
        assert_eq!(user_with_no_rights.has_permission(&buckets_user), Permission::None);
        assert_eq!(user_with_no_rights.has_permission(&users_user), Permission::None);
        assert_eq!(user_with_no_rights.has_permission(&images_user), Permission::None);
        assert_eq!(user_with_no_rights.has_permission(&sessions_user), Permission::None);

        //success case
        assert_eq!(user_with_all_rights.has_permission(&admin_user), Permission::Granted);
        assert_eq!(user_with_all_rights.has_permission(&buckets_user), Permission::Granted);
        assert_eq!(user_with_all_rights.has_permission(&users_user), Permission::Granted);
        assert_eq!(user_with_all_rights.has_permission(&images_user), Permission::Granted);
        assert_eq!(user_with_all_rights.has_permission(&sessions_user), Permission::Granted);
    
    }
}