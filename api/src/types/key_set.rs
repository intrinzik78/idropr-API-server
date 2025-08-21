use blake3::{self, Hash};

use crate::enums::{Error, Uuid, VerificationStatus};

type Result<T> = std::result::Result<T,Error>;

const BUF_SIZE:usize = 32;
const KEY_SIZE:usize = 16;
const SECRET_SIZE:usize = KEY_SIZE;

#[derive(Debug)]
pub struct KeySet {
    pub key: [u8;16],
    pub secret: [u8;16],
    pub hash: blake3::Hash
}

impl KeySet {
    pub fn new() -> Result<KeySet> {
        // create 32 bytes of random data
        let uuid: [u8; BUF_SIZE] = match Uuid::crypto32()? {
            Uuid::Crypto(buf) => buf,
            _ => return  Err(Error::WrongUuidTypeForSessionHash)
        };

        // first 16 bytes will be the session id
        let mut key: [u8;KEY_SIZE] = [0;KEY_SIZE];  // reserve 16 bytes
        key.copy_from_slice(&uuid[..KEY_SIZE]); // copy 16 bytes

        // last 16 bytes will be the session secret
        let mut secret: [u8;SECRET_SIZE] = [0;SECRET_SIZE]; // reserve 16 bytes
        secret.copy_from_slice(&uuid[KEY_SIZE..]);   // copy 16 bytes

        // hash the entire 32 bytes
        let hash =  blake3::hash(&uuid);
        
        let key_set = KeySet {
            key,
            secret,
            hash
        };

        Ok(key_set)
    }

    pub fn verify(key: &[u8;KEY_SIZE], secret: &[u8;KEY_SIZE], hash: &Hash) -> VerificationStatus {
        let mut combined_set= [0;BUF_SIZE];
        combined_set[..KEY_SIZE].copy_from_slice(key);
        combined_set[KEY_SIZE..].copy_from_slice(secret);

        let test_hash = blake3::hash(&combined_set);

        match &test_hash == hash {
            true => VerificationStatus::Verified,
            false => VerificationStatus::Unverified
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Instant};

    #[test]
    fn test_key_set_builder() {
        let start = Instant::now();
        for _ in 0..100_000 {
            let mut key_set = KeySet::new().unwrap();
            
            let verfied = KeySet::verify(&key_set.key, &key_set.secret, &key_set.hash.clone());
            assert_eq!(verfied,VerificationStatus::Verified);

            if key_set.secret[0] == 0 {
                key_set.secret[0] = 1;
            } else {
                key_set.secret[0] = 0;
            }

            let unverified = KeySet::verify(&key_set.key, &key_set.secret, &key_set.hash.clone());
            assert_eq!(unverified,VerificationStatus::Unverified);
        }
        let end = Instant::now();

        let dur = end - start;
        
        println!("\nTime elapsed hashing session key-sets: {:?}\n", dur.as_millis());
    }
}