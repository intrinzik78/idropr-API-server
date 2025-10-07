use crate::enums::Error;

pub trait ToHash {
    fn to_hash(&self) -> Result<blake3::Hash,Error>;
}

impl ToHash for &str {
    #[inline]
    fn to_hash(&self) -> Result<blake3::Hash,Error> {
        let mut buf:[u8;32] = [0;32];
        let bytes = self.as_bytes();
        buf[..bytes.len()].copy_from_slice(bytes);
                
        let hash = blake3::hash(&buf);

        Ok(hash)
    }
}

impl ToHash for String {
    #[inline]
    fn to_hash(&self) -> Result<blake3::Hash,Error> {
        let mut buf:[u8;32] = [0;32];
        let bytes = self.as_bytes();
        buf[..bytes.len()].copy_from_slice(bytes);
                
        let hash = blake3::hash(&buf);

        Ok(hash)
    }
}