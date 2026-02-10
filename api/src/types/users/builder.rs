use crate::{
    enums::{Error, User, UserAccountStatus, UserType},
    traits::User as UserTrait,
    types::{permissions::UserPermissions, users::{StandardUser, SystemUser}}
};

type Result<T> = std::result::Result<T,Error>;

#[derive(Default)]
pub struct Builder {
    pub id: Option<i64>,
    pub epoch:Option<u64>,
    pub username: Option<String>,
    pub hash: Option<String>,
    pub user_status: Option<UserAccountStatus>,
    pub user_type: Option<UserType>,
    pub permissions: Option<UserPermissions>
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, user_id: i64) -> Self {
        self.id = Some(user_id);
        self
    }

    pub fn epoch(mut self, epoch:u64) -> Self {
        self.epoch = Some(epoch);
        self
    }

    pub fn username(mut self, username: String) -> Self {
        self.username = Some(username);
        self
    }

    pub fn hash(mut self, hash: String) -> Self {
        self.hash = Some(hash);
        self
    }

    pub fn permissions(mut self, permissions: UserPermissions) -> Self {
        self.permissions = Some(permissions);
        self
    }

    pub fn user_status(mut self, user_status: UserAccountStatus) -> Self {
        self.user_status = Some(user_status);
        self
    }

    pub fn user_type(mut self, user_type: UserType) -> Self {
        self.user_type = Some(user_type);
        self
    }

    pub fn build(self) -> Result<User> {
        let user_type = self.user_type.clone().ok_or(Error::RequiredUserBuildDataMissing)?;

        let user = match &user_type {
            UserType::Standard => self.build_standard_user()?,
            UserType::System   => self.build_system_user()?,
        };

        Ok(user)
    }

    fn build_standard_user(self) -> Result<User> {
        let user = StandardUser::new(self)?;
        Ok(User::Standard(user))
    }

    fn build_system_user(self) -> Result<User> {
        let user = SystemUser::new(self)?;
        Ok(User::System(user))
    }
}
