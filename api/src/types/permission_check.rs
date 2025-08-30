use crate::enums::{AuthContext, Permission, RefreshStatus};

pub struct PermissionCheck {
    pub permission: Permission,
    pub refresh_status: RefreshStatus,
    pub auth_context: AuthContext
}