use rand::{Rng,{rngs::OsRng, TryRngCore}};
use std::num::NonZeroU8;

use crate::Error;

type Result<T> = std::result::Result<T,Error>;

const ALPHABET_LENGTH: usize  = 52;
const NUMS_LENGTH: usize = 10;

#[derive(Debug, Clone, PartialEq)]
pub enum Uuid {
    Crypto([u8;32]),
    WebSafe(String),
    WebSafeNums(String),
}

impl Uuid {
    pub fn crypto32() -> Result<Uuid> {
        let mut buf = [0u8;32];
        OsRng.try_fill_bytes(&mut buf)?;
        
        Ok(Uuid::Crypto(buf))
    }

    pub fn web_safe(length_opt: Option<NonZeroU8>) -> Result<Uuid> {
        let length = match length_opt {
            Some(opt) => opt.get(),
            None => return Err(Error::ZeroLengthUUIDFound)
        };

        let mut generator = rand::rng();
        let mut uuid = String::with_capacity(length as usize);

        for _ in 0..length {
            let idx = generator.random_range(0..ALPHABET_LENGTH) as u8;

            let ch = match idx {
                ..26 => (idx + 65) as char,
                26.. => (idx + 71) as char
            };

            uuid.push(ch);
        }

        Ok(Uuid::WebSafe(uuid))
    }

    pub fn web_safe_with_nums(length_opt: Option<NonZeroU8>) -> Result<Uuid> {
        let length = match length_opt {
            Some(opt) => opt.get(),
            None => return Err(Error::ZeroLengthUUIDFound)
        };

        let mut generator = rand::rng();
        let mut uuid = String::with_capacity(length as usize);

        for _ in 0..length {
            let idx = generator.random_range(0..ALPHABET_LENGTH + NUMS_LENGTH) as u8;

            let ch = match idx {
                ..26    => (idx + 65) as char,
                26..52  => (idx + 71) as char,
                52..    => (idx - 4) as char
            };

            uuid.push(ch);
        }

        Ok(Uuid::WebSafeNums(uuid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_set() {
        for len in 2..128 {
            let len = len as u8;
            let length_opt = NonZeroU8::new(len);
            let web_result = Uuid::web_safe(length_opt);
            let crypto_result = Uuid::crypto32();
            let nums_result = Uuid::web_safe_with_nums(length_opt);

            assert!(web_result.is_ok());
            assert!(crypto_result.is_ok());
            assert!(nums_result.is_ok());

            let () = match web_result.expect("panicked creating web safe uuid") {
                Uuid::WebSafe(uuid) => assert_eq!(uuid.len() as u8,len),
                _ => panic!("unreachable")
            };

            let () = match crypto_result.expect("panicked creating crypto uuid") {
                Uuid::Crypto(buf) => assert_eq!(buf.len(),32),
                _ => panic!("unreachable")
            };

            let () = match nums_result.expect("panicked creating crypto uuid") {
                Uuid::WebSafeNums(uuid) => assert_eq!(uuid.len() as u8,len),
                _ => panic!("unreachable")
            };
        }
    }
}
