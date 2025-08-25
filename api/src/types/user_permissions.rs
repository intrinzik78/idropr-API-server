// external libraries
use sqlx::{FromRow, MySql, Transaction};

// internal libraries
use crate::{
    enums::{Action,Error,Scope,Resource},
    traits::{U128Bits,HasPermission},
    types::DatabaseConnection
};

type Result<T> = std::result::Result<T,Error>;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct UserPermissions {
    mask: u128
}

#[derive(Debug, Clone, FromRow)]
struct DatabaseHelper {
    upper: u64,
    lower: u64
}

impl DatabaseHelper {
    pub fn transform(self) -> UserPermissions {
        let mask: u128 = u128::from_upper_lower(self.upper,self.lower);
        UserPermissions { mask }
    }
}

// async
impl UserPermissions {
    pub async fn into_db_as_transaction(user_id: i64, permissions: UserPermissions, tx: &mut Transaction<'static,MySql>) -> Result<u64> {
        let upper = permissions.mask.to_upper();
        let lower = permissions.mask.to_lower();
        let sql = "INSERT INTO `user_permissions` (upper,lower) VALUES (?,?) WHERE id = ?";
        let insert_id = sqlx::query(sql)
            .bind(upper)
            .bind(lower)
            .bind(user_id)
            .execute(&mut **tx)
            .await?
            .last_insert_id();

        Ok(insert_id)
    }

    pub async fn by_user_id(user_id: i64,database: &DatabaseConnection) -> Result<UserPermissions> {
        let sql = "SELECT upper,lower FROM `user_permissions` WHERE id = ?";
        let helper:DatabaseHelper = sqlx::query_as(sql)
            .bind(user_id)
            .fetch_one(&database.pool)
            .await?;

        let user_permissions = helper.transform();

        Ok(user_permissions)
    }
}

// builder functions
impl UserPermissions {

    pub fn new() -> Self {
        Self { mask: 0 }
    }

    #[inline]
    fn grant_bit(&mut self, resource: Resource, action: Action, scope: Scope) {
        let resource_bits = self.to_mask(resource, action, scope);
        self.mask |= resource_bits;
    }

    #[inline]
    fn grant_admin_bit(&mut self, bit: u128) {
        self.mask = (self.mask | bit) as u128;
    }

    pub fn with_read_self(mut self, resource: Resource) -> Self {
        let action = Action::Read;
        let scope = Scope::Self_;

        self.grant_bit(resource, action, scope);
        self
    }

    pub fn with_write_self(mut self, resource: Resource) -> Self {
        let action = Action::Write;
        let scope = Scope::Self_;

        self.grant_bit(resource, action, scope);
        self
    }

    pub fn with_delete_self(mut self, resource: Resource) -> Self {
        let action = Action::Delete;
        let scope = Scope::Self_;

        self.grant_bit(resource, action, scope);
        self
    }

    pub fn with_read_any(mut self, resource: Resource) -> Self {
        let action = Action::Read;
        let scope = Scope::Any;

        self.grant_bit(resource, action, scope);
        self
    }

    pub fn with_write_any(mut self, resource: Resource) -> Self {
        let action = Action::Write;
        let scope = Scope::Any;

        self.grant_bit(resource, action, scope);
        self
    }

    pub fn with_delete_any(mut self, resource: Resource) -> Self {
        let action = Action::Delete;
        let scope = Scope::Any;
        
        self.grant_bit(resource, action, scope);
        self
    }

    pub fn with_rw_self(mut self, resource: Resource) -> Self {
        let scope = Scope::Self_;
        
        self.grant_bit(resource, Action::Read, scope);
        self.grant_bit(resource, Action::Write, scope);
        self
    }

    pub fn with_rw_any(mut self, resource: Resource) -> Self {
        let scope = Scope::Any;

        self.grant_bit(resource, Action::Read, scope);
        self.grant_bit(resource, Action::Write, scope);
        self
    }

    pub fn with_admin(mut self, resource: Resource) -> Self {
        let bit = self.set_admin(resource);
        self.grant_admin_bit(bit);
        self
    }

    pub fn mask(&self) -> u128 {
        self.mask
    }
}


#[cfg(test)]
mod tests {
    use crate::{enums::Resource, types::UserPermissions};

    use super::*;

    #[test]
    fn create_upper_mask() {
        let mut permissions = UserPermissions::new();
        let resource_list:Vec<Resource> = vec![
            Resource::Buckets,
            Resource::Images,
            Resource::Secrets,
            Resource::Sessions,
            Resource::System,
            Resource::Users
        ];

        // build full admin permissions for all resources
        for idx in resource_list.iter() {
            permissions = permissions
                .with_rw_self(*idx)
                .with_rw_any(*idx)
                .with_delete_self(*idx)
                .with_delete_any(*idx)
                .with_admin(*idx);
        }

        let lower = permissions.mask.to_lower();
        let upper = permissions.mask.to_upper();

        println!("upper: {}", upper);
        println!("lower: {}", lower);

        assert_eq!(upper,0);
        assert_eq!(lower,210830276673471);
    }
}