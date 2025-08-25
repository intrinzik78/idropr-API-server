// a fast and efficient way to determine whether a user has the appropriate permissions set to access an API endpoint
use crate::{
    enums::{Action,Permission,Scope,Resource},
    types::UserPermissions
};

const BITS_PER_RESOURCE:u8 = 8;
const ADMIN_OFFSET:u8 = 7;

pub trait HasPermission {
    fn has_permission(&self, required_permissions: &UserPermissions) -> Permission;
    fn to_mask(&self, resource: Resource, action: Action, scope: Scope) -> u128;
    fn set_admin(&self, resource: Resource) -> u128;
}

impl HasPermission for UserPermissions {
    
    #[inline]
    fn to_mask(&self, resource: Resource, action: Action, scope: Scope) -> u128 {
        let block_offset = resource as u8 * BITS_PER_RESOURCE;
        let bit_offset = match (action as u8, scope as u8) {
            (0,0) => 0, // READ SELF
            (0,1) => 1, // READ ANY
            (1,0) => 2, // WRITE SELF
            (1,1) => 3, // WRITE ANY
            (2,0) => 4, // DELETE SELF
            (2,1) => 5, // DELETE ANY
            _     => 6, // RESERVED
        };


        let bit:u128 = 1_u128 << ((block_offset + bit_offset) as u32);
        bit
    }

    #[inline]
    fn has_permission(&self, required_permissions: &UserPermissions) -> Permission {
        let compared_permissions = self.mask() & required_permissions.mask();

        match compared_permissions == required_permissions.mask() {
            true  => Permission::Granted,
            false => Permission::Denied
        }
    }

    #[inline]
    fn set_admin(&self, resource: Resource) -> u128 {
        let admin_bit: u128 = 1_u128 << (((resource as u8 * BITS_PER_RESOURCE) + ADMIN_OFFSET) as u32);

        admin_bit
    }
}

#[cfg(test)]
mod tests { 
    use super::*;
    use crate::enums::{Action as A, Scope as S, Resource as R, Permission as P};
    use crate::traits::HasPermission;

    // Helpers to compute expected indices/bits for assertions
    const fn base(res: R) -> u8 { (res as u8) * BITS_PER_RESOURCE }
    const fn idx(res: R, act: A, sc: S) -> u32 {
        let off = match (act as u8, sc as u8) {
            (0,0) => 0, // READ SELF
            (0,1) => 1, // READ ANY
            (1,0) => 2, // WRITE SELF
            (1,1) => 3, // WRITE ANY
            (2,0) => 4, // DELETE SELF
            (2,1) => 5, // DELETE ANY
            _     => 6, // RESERVED
        };
        (base(res) + off) as u32
    }
    const fn bit(res: R, act: A, sc: S) -> u128 {
        1u128 << idx(res, act, sc)
    }
    const fn admin_bit(res: R) -> u128 {
        1u128 << ((base(res) + ADMIN_OFFSET) as u32)
    }

    #[test]
    fn bit_positions_users() {
        // Sanity: Users block starts at bit 16 (2 * 8)
        assert_eq!(base(R::Users), 16);

        let u = UserPermissions::new();
        assert_eq!(u.to_mask(R::Users, A::Read,  S::Self_), bit(R::Users, A::Read,  S::Self_));
        assert_eq!(u.to_mask(R::Users, A::Read,  S::Any ),  bit(R::Users, A::Read,  S::Any ));
        assert_eq!(u.to_mask(R::Users, A::Write, S::Self_), bit(R::Users, A::Write, S::Self_));
        assert_eq!(u.to_mask(R::Users, A::Write, S::Any ),  bit(R::Users, A::Write, S::Any ));
        assert_eq!(u.to_mask(R::Users, A::Delete,S::Self_), bit(R::Users, A::Delete,S::Self_));
        assert_eq!(u.to_mask(R::Users, A::Delete,S::Any ),  bit(R::Users, A::Delete,S::Any ));
    }

    #[test]
    fn admin_bit_users() {
        let u = UserPermissions::new();
        let b = u.set_admin(R::Users);
        assert_eq!(b, admin_bit(R::Users));
        // shifting index is exactly base + ADMIN_OFFSET
        assert_eq!(b, 1u128 << ((base(R::Users) + ADMIN_OFFSET) as u32));               
    }

    #[test]
    fn builder_chaining_or_not_overwrite() {
        // with_rw_self should set ReadSelf and WriteSelf
        let p = UserPermissions::new().with_rw_self(R::Users);
        let expected = bit(R::Users, A::Read, S::Self_) | bit(R::Users, A::Write, S::Self_);
        assert_eq!(p.mask(), expected);

        // Chaining should OR additional bits, not overwrite
        let p2 = p.with_read_any(R::Users);
        let expected2 = expected | bit(R::Users, A::Read, S::Any);
        assert_eq!(p2.mask(), expected2);
    }

    #[test]
    fn has_permission_single_and_multi() {
        let held = UserPermissions::new()
            .with_read_self(R::Users)
            .with_write_self(R::Users);

        // Single requirement granted
        let req_read = UserPermissions::new().with_read_self(R::Users);
        assert_eq!(held.has_permission(&req_read), P::Granted);

        // Multi requirement granted (held has both)
        let req_rw = UserPermissions::new().with_rw_self(R::Users);
        assert_eq!(held.has_permission(&req_rw), P::Granted);

        // Multi requirement denied (missing DeleteSelf)
        let req_rwd = UserPermissions::new()
            .with_read_self(R::Users)
            .with_write_self(R::Users)
            .with_delete_self(R::Users);
        assert_eq!(held.has_permission(&req_rwd), P::Denied);
    }

    #[test]
    fn scope_mismatch_denied() {
        // Holder has ReadSelf on Users
        let held = UserPermissions::new().with_read_self(R::Users);
        // Endpoint requires ReadAny on Users
        let req_any = UserPermissions::new().with_read_any(R::Users);
        assert_eq!(held.has_permission(&req_any), P::Denied);
    }

    #[test]
    fn admin_short_circuit_when_granted() {
        // check admin bit is being set
        let user = UserPermissions::default().with_admin(R::Users);
        let admin_mask = admin_bit(R::Users);
        
        assert_eq!(user.mask() & admin_mask, admin_mask);

        // check admin bit plays nicely when other bits are being set
        let admin = user.with_read_self(R::Users);
        let expected = admin_mask | bit(R::Users, A::Read, S::Self_);
        
        assert_eq!(admin.mask(), expected);
    }

    #[test]
    fn bounds_do_not_overshift() {
        // Highest defined resource in your enum is System=5 => base = 40
        // Admin bit is base+7 = 47 (well under 128)
        let highest_admin_idx = (base(R::System) + ADMIN_OFFSET) as u32;
        assert!(highest_admin_idx < 128);
        let _ = 1u128 << highest_admin_idx; // should not panic in debug
    }
}