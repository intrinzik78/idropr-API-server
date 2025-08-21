use std::{time::Instant,net::IpAddr};

#[derive(Clone,Debug,PartialEq, Eq, PartialOrd, Ord)]
pub struct HeapKey {
    pub expires_at: Instant,
    pub ver: u64,
    pub ip: IpAddr
}