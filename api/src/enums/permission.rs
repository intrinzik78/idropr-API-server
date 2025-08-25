#[derive(Copy,Clone,Debug,PartialEq)]
pub enum Permission {
    Denied,       // default deny
    Granted,      // explicitly granted
}

#[repr(u8)]
#[derive(Copy,Clone,Debug,PartialEq)]
pub enum Resource {
    Buckets = 0,
    Images = 1,
    Users = 2,
    Secrets = 3,
    Sessions = 4,
    System = 5
}

#[repr(u8)]
#[derive(Copy,Clone,Debug,PartialEq)]
pub enum Action {
    Read = 0,
    Write = 1,
    Delete = 2
}

#[repr(u8)]
#[derive(Copy,Clone,Debug,PartialEq)]
pub enum Scope {
    Self_ = 0,
    Any = 1
}