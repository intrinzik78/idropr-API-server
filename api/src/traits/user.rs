use std::future::Future;
use database::types::DatabaseConnection;

use crate::{
    enums::{Error,UserAccountStatus, UserType},
    types::{permissions::UserPermissions, users::Builder}};

pub trait User<T> {
    fn id(&self) -> i64;
    fn epoch(&self) -> u64;
    fn hash(&self) -> &str;
    fn permissions(&self) -> UserPermissions;
    fn username(&self) -> &str;
    fn user_type(&self) -> UserType;
    fn status(&self) -> UserAccountStatus;
    fn new(builder:Builder) -> Result<T,Error>;
    fn by_id_unchecked(user_id: i64, connection:&DatabaseConnection) -> impl Future<Output = Result<Option<T>,Error>> + Send;
    fn by_id_checked(user_id: i64, account_status:UserAccountStatus, connection:&DatabaseConnection) -> impl Future<Output = Result<Option<T>,Error>> + Send;
}