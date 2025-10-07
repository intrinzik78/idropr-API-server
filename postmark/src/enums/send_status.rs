#[repr(u32)]
#[derive(Clone,Debug,PartialEq)]
pub enum SendStatus {
    Accepted = 1,
    Delivered = 2,
    Failed = 3,
    Rejected = 4,
    Bounced = 5,
    Complained = 6,
    Suppressed = 7
}