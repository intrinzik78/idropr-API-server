// external libraries
use sqlx::{FromRow, MySql, Transaction};

// internal libraries
use crate::{
    enums::{Error,Permission},
    traits::{ToNumber,ToPermission},
    types::DatabaseConnection
};

type Result<T> = std::result::Result<T,Error>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UserPermissions {
    pub admin_read: Permission,
    pub admin_write: Permission,
    pub admin_delete: Permission,
    pub buckets_read: Permission,
    pub buckets_write: Permission,
    pub buckets_delete: Permission,
    pub images_read: Permission,
    pub images_write: Permission,
    pub images_delete: Permission,
    pub users_read: Permission,
    pub users_write: Permission,
    pub users_delete: Permission,
    pub sessions_read: Permission,
    pub sessions_write: Permission,
    pub sessions_delete: Permission,
}

#[derive(Debug, Clone, FromRow)]
struct DatabaseHelper {
    admin_read: i8,
    admin_write: i8,
    admin_delete: i8,
    buckets_read: i8,
    buckets_write: i8,
    buckets_delete: i8,
    images_read: i8,
    images_write: i8,
    images_delete: i8,
    users_read: i8,
    users_write: i8,
    users_delete: i8,
    sessions_read: i8,
    sessions_write: i8,
    sessions_delete: i8,
}

impl DatabaseHelper {
    pub fn transform(self) -> UserPermissions {
        UserPermissions {
            admin_read: self. admin_read.to_permission(),
            admin_write: self. admin_write.to_permission(),
            admin_delete: self. admin_delete.to_permission(),
            buckets_read: self.buckets_read.to_permission(),
            buckets_write: self.buckets_write.to_permission(),
            buckets_delete: self.buckets_delete.to_permission(),
            images_read: self.images_read.to_permission(),
            images_write: self.images_write.to_permission(),
            images_delete: self.images_delete.to_permission(),
            users_read: self.users_read.to_permission(),
            users_write: self.users_write.to_permission(),
            users_delete: self.users_delete.to_permission(),
            sessions_read: self.sessions_read.to_permission(),
            sessions_write: self.sessions_write.to_permission(),
            sessions_delete: self.sessions_delete.to_permission(),
        }
    }
}

// async
impl UserPermissions {
    pub async fn into_db_as_transaction(user_id: i64, access_rights: UserPermissions, tx: &mut Transaction<'static,MySql>) -> Result<u64> {
        let admin_read = access_rights.admin_read.to_i8();        
        let admin_write = access_rights.admin_write.to_i8();        
        let admin_delete = access_rights.admin_delete.to_i8();            
        let buckets_read = access_rights.buckets_read.to_i8();
        let buckets_write = access_rights.buckets_write.to_i8();
        let buckets_delete = access_rights.buckets_delete.to_i8();
        let images_read = access_rights.images_read.to_i8();
        let images_write = access_rights.images_write.to_i8();
        let images_delete = access_rights.images_delete.to_i8();
        let users_read = access_rights.users_read.to_i8();
        let users_write = access_rights.users_write.to_i8();
        let users_delete = access_rights.users_delete.to_i8();
        let sessions_read = access_rights.sessions_read.to_i8();
        let sessions_write = access_rights.sessions_write.to_i8();
        let sessions_delete = access_rights.sessions_delete.to_i8();

        let sql = "INSERT INTO `user_permissions` (id,buckets_read,buckets_write,buckets_delete,images_read,images_write,images_delete,users_read,users_write,users_delete,sessions_read,sessions_write,sessions_delete) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?)";
        let insert_id = sqlx::query(sql)
            .bind(admin_read)                    
            .bind(admin_write)            
            .bind(admin_delete)                    
            .bind(user_id)
            .bind(buckets_read)
            .bind(buckets_write)
            .bind(buckets_delete)
            .bind(images_read)
            .bind(images_write)
            .bind(images_delete)
            .bind(users_read)
            .bind(users_write)
            .bind(users_delete)
            .bind(sessions_read)
            .bind(sessions_write)
            .bind(sessions_delete)
            .execute(&mut **tx)
            .await?
            .last_insert_id();

        Ok(insert_id)
    }

    pub async fn from_user_id(user_id: i64,database: &DatabaseConnection) -> Result<UserPermissions> {
        let sql = "SELECT buckets_read, buckets_write, buckets_delete, images_read, images_write, images_delete, sessions_read, sessions_write, sessions_delete, users_read, users_write, users_delete FROM `user_permissions` WHERE id = ? LIMIT 1";
        let helper: DatabaseHelper = sqlx::query_as(sql)
            .bind(user_id)
            .fetch_one(&database.pool)
            .await?;

        let user_permissions = helper.transform();

        Ok(user_permissions)
    }
}

// builder functions
impl UserPermissions {
    pub fn with_admin_read(mut self) -> Self {
        self.admin_read = Permission::Granted;
        self
    }

    pub fn with_admin_write(mut self) -> Self {
        self.admin_write = Permission::Granted;
        self
    }

    pub fn with_admin_delete(mut self) -> Self {
        self.admin_delete = Permission::Granted;
        self
    }

    pub fn with_buckets_read(mut self) -> Self {
        self.buckets_read = Permission::Granted;
        self
    }

    pub fn with_buckets_write(mut self) -> Self {
        self.buckets_write = Permission::Granted;
        self
    }

    pub fn with_buckets_delete(mut self) -> Self {
        self.buckets_delete = Permission::Granted;
        self
    }

    pub fn with_images_read(mut self) -> Self {
        self.images_read = Permission::Granted;
        self
    }

    pub fn with_images_write(mut self) -> Self {
        self.images_write = Permission::Granted;
        self
    }

    pub fn with_images_delete(mut self) -> Self {
        self.images_delete = Permission::Granted;
        self
    }

    pub fn with_sessions_read(mut self) -> Self {
        self.sessions_read = Permission::Granted;
        self
    }

    pub fn with_sessions_write(mut self) -> Self {
        self.sessions_write = Permission::Granted;
        self
    }

    pub fn with_sessions_delete(mut self) -> Self {
        self.sessions_delete = Permission::Granted;
        self
    }

    pub fn with_users_read(mut self) -> Self {
        self.users_read = Permission::Granted;
        self
    }

    pub fn with_users_write(mut self) -> Self {
        self.users_write = Permission::Granted;
        self
    }

    pub fn with_users_delete(mut self) -> Self {
        self.users_delete = Permission::Granted;
        self
    }

    pub fn with_users_full(mut self) -> Self {
        self.users_read = Permission::Granted;
        self.users_write = Permission::Granted;
        self.users_delete = Permission::Granted;
        self
    }

    pub fn with_buckets_full(mut self) -> Self {
        self.buckets_read = Permission::Granted;
        self.buckets_write = Permission::Granted;
        self.buckets_delete = Permission::Granted;
        self
    }

    pub fn with_images_full(mut self) -> Self {
        self.images_read = Permission::Granted;
        self.images_write = Permission::Granted;
        self.images_delete = Permission::Granted;
        self
    }

    pub fn with_sessions_full(mut self) -> Self {
        self.sessions_read = Permission::Granted;
        self.sessions_write = Permission::Granted;
        self.sessions_delete = Permission::Granted;
        self
    }

}

impl Default for UserPermissions {
    fn default() -> Self {
        UserPermissions {
            admin_read: Permission::None,
            admin_write: Permission::None,
            admin_delete: Permission::None,
            buckets_read: Permission::None,
            buckets_write: Permission::None,
            buckets_delete: Permission::None,
            images_read: Permission::None,
            images_write: Permission::None,
            images_delete: Permission::None,
            users_read: Permission::None,
            users_write: Permission::None,
            users_delete: Permission::None,
            sessions_read: Permission::None,
            sessions_write: Permission::None,
            sessions_delete: Permission::None,
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::traits::HasPermission;

    use super::*;

    /// tests the default build has all required permissions and they are set to Permission::None
    #[test]
    fn default_permissions_builder() {
        let rights = UserPermissions {
            admin_read: Permission::None,
            admin_write: Permission::None,
            admin_delete: Permission::None,
            buckets_read: Permission::None,
            buckets_write: Permission::None,
            buckets_delete: Permission::None,
            images_read: Permission::None,
            images_write: Permission::None,
            images_delete: Permission::None,
            users_read: Permission::None,
            users_write: Permission::None,
            users_delete: Permission::None,
            sessions_read: Permission::None,
            sessions_write: Permission::None,
            sessions_delete: Permission::None,
        };
        let build_test = UserPermissions::default();
        assert_eq!(rights,build_test);
    }

    /// tests the builder can set all required permissions to Permission::Granted
    #[test]
    fn full_permissions_builder() {
        let rights = UserPermissions {
            admin_read: Permission::Granted,
            admin_write: Permission::Granted,
            admin_delete: Permission::Granted,
            buckets_read: Permission::Granted,
            buckets_write: Permission::Granted,
            buckets_delete: Permission::Granted,
            images_read: Permission::Granted,
            images_write: Permission::Granted,
            images_delete: Permission::Granted,
            users_read: Permission::Granted,
            users_write: Permission::Granted,
            users_delete: Permission::Granted,
            sessions_read: Permission::Granted,
            sessions_write: Permission::Granted,
            sessions_delete: Permission::Granted,
        };

        let build_test = UserPermissions::default()
            .with_admin_read()
            .with_admin_write()
            .with_admin_delete()
            .with_buckets_read()
            .with_buckets_write()
            .with_buckets_delete()
            .with_images_read()
            .with_images_write()
            .with_images_delete()
            .with_users_read()
            .with_users_write()
            .with_users_delete()
            .with_sessions_read()
            .with_sessions_write()
            .with_sessions_delete();

        assert_eq!(rights,build_test);
    }

    /// runs tests against both Permission::Granted & Permission::None builder functions
    #[test]
    fn permissions_check() {
        let no_rights = UserPermissions::default();
        let full_rights = UserPermissions::default()
            .with_buckets_full()
            .with_images_full()
            .with_sessions_full()
            .with_users_full();

        let test_full = UserPermissions::default()
            .with_buckets_full()
            .with_images_full()
            .with_sessions_full()
            .with_users_full();

        assert_eq!(no_rights.has_permission(&full_rights),Permission::None);
        assert_eq!(no_rights.has_permission(&no_rights),Permission::Granted);
        assert_eq!(test_full.has_permission(&no_rights),Permission::Granted);
        assert_eq!(test_full.has_permission(&full_rights),Permission::Granted);
    }
}