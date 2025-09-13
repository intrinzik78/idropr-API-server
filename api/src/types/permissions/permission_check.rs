use crate::enums::{AuthContext, Permission, sessions::RefreshStatus};

#[derive(Debug)]
pub struct PermissionCheck {
    pub permission: Permission,
    pub refresh_status: RefreshStatus,
    pub auth_context: AuthContext
}