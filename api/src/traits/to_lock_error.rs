use std::collections::HashMap;
use std::sync::{PoisonError, RwLockReadGuard, RwLockWriteGuard, TryLockError};

use crate::{
    enums::Error,
    types::sessions::{Session,UserEpochController}
};

pub trait ToLockError<T> {
    fn to_lock_error(self) -> Result<T,Error>;
}

/// HashMap<[u8; 16], Session>> READ LOCK ERROR
impl <'a> ToLockError <RwLockReadGuard<'a, HashMap<[u8; 16], Session>>> for Result<RwLockReadGuard<'a, HashMap<[u8; 16], Session>>, PoisonError<RwLockReadGuard<'_, HashMap<[u8; 16], Session>>>> {
    fn to_lock_error(self) -> Result<RwLockReadGuard<'a, HashMap<[u8; 16], Session>>,Error> {
        match self {
            Self::Ok(t) => Ok(t),
            Self::Err(_e) => Err(Error::PoisonedSessionList)
        }
    }
}

/// HashMap<[u8; 16], Session>> READ LOCK ERROR
impl <'a> ToLockError <RwLockReadGuard<'a, HashMap<[u8; 16], Session>>> for Result<RwLockReadGuard<'a, HashMap<[u8; 16], Session>>, TryLockError<RwLockReadGuard<'_, HashMap<[u8; 16], Session>>>> {
    fn to_lock_error(self) -> Result<RwLockReadGuard<'a, HashMap<[u8; 16], Session>>,Error> {
        match self {
            Self::Ok(t) => Ok(t),
            Self::Err(_e) => Err(Error::PoisonedSessionList)
        }
    }
}

/// HashMap<[u8; 16], Session>> READ LOCK ERROR
impl <'a> ToLockError <RwLockWriteGuard<'a, HashMap<[u8; 16], Session>>> for Result<RwLockWriteGuard<'a, HashMap<[u8; 16], Session>>, PoisonError<RwLockWriteGuard<'_, HashMap<[u8; 16], Session>>>> {
    fn to_lock_error(self) -> Result<RwLockWriteGuard<'a, HashMap<[u8; 16], Session>>,Error> {
        match self {
            Self::Ok(t) => Ok(t),
            Self::Err(_e) => Err(Error::PoisonedSessionList)
        }
    }
}

/// poison error
impl <'a> ToLockError <RwLockReadGuard<'a, UserEpochController>> for Result<RwLockReadGuard<'a, UserEpochController>, PoisonError<RwLockReadGuard<'_, UserEpochController>>> {
    fn to_lock_error(self) -> Result<RwLockReadGuard<'a, UserEpochController>,Error> {
        match self {
            Self::Ok(t) => Ok(t),
            Self::Err(_e) => Err(Error::UserEpochPoisoned)
        }
    }
}

/// try read lock failed error
impl <'a> ToLockError <RwLockReadGuard<'a, UserEpochController>> for Result<RwLockReadGuard<'a, UserEpochController>, TryLockError<RwLockReadGuard<'_, UserEpochController>>> {
    fn to_lock_error(self) -> Result<RwLockReadGuard<'a, UserEpochController>,Error> {
        match self {
            Self::Ok(t) => Ok(t),
            Self::Err(_e) => Err(Error::UserEpochLockNotAquired)
        }
    }
}

/// write lock failed error
impl <'a> ToLockError <RwLockWriteGuard<'a, UserEpochController>> for Result<RwLockWriteGuard<'a, UserEpochController>, PoisonError<RwLockWriteGuard<'_, UserEpochController>>> {
    fn to_lock_error(self) -> Result<RwLockWriteGuard<'a, UserEpochController>,Error> {
        match self {
            Self::Ok(t) => Ok(t),
            Self::Err(_e) => Err(Error::UserEpochPoisoned)
        }
    }
}

/// try write lock failed error
impl <'a> ToLockError <RwLockWriteGuard<'a, UserEpochController>> for Result<RwLockWriteGuard<'a, UserEpochController>, TryLockError<RwLockWriteGuard<'_, UserEpochController>>> {
    fn to_lock_error(self) -> Result<RwLockWriteGuard<'a, UserEpochController>,Error> {
        match self {
            Self::Ok(t) => Ok(t),
            Self::Err(_e) => Err(Error::UserEpochLockNotAquired)
        }
    }
}