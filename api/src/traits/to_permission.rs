use crate::enums::Permission;

// permissions are stored in the database as a TINYINT(1) with DEFAULT(0)
// the ToPermission trait makes converting database values to enums states effortless
pub trait ToPermission {
    fn to_permission(self) -> Permission;
}

impl ToPermission for bool {
    fn to_permission(self) -> Permission {
        match self {
            true => Permission::Granted,
            false => Permission::Denied
        }
    }
}

impl ToPermission for i8 {
    fn to_permission(self) -> Permission {
        match self {
            1 => Permission::Granted,
            _ => Permission::Denied
        }
    }
}

impl ToPermission for u8 {
    fn to_permission(self) -> Permission {
        match self {
            1 => Permission::Granted,
            _ => Permission::Denied
        }
    }
}