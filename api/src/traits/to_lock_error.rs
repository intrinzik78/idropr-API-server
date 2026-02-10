use std::sync::{PoisonError, TryLockError};
use crate::enums::Error;

pub trait LockErr {}
impl<T> LockErr for PoisonError<T> {}
impl<T> LockErr for TryLockError<T> {}

pub trait ToLockError<T> {
    fn to_lock_error(self, e: Error) -> Result<T, Error>;
}

impl<T, E: LockErr> ToLockError<T> for Result<T, E> {
    #[inline]
    fn to_lock_error(self, e: Error) -> Result<T, Error> {
        self.map_err(|_| e)
    }
}