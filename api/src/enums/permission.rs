#[derive(Clone,Debug,PartialEq)]
pub enum Permission {
    Denied,         // default deny
    Granted,  // explicitly granted
}

#[repr(u8)]
#[derive(Copy,Clone,Debug,PartialEq)]
pub enum Resource {
    Buckets = 0,        // image buckets
    Images = 1,         // individual images
    Users = 2,          // user details
    Secrets = 3,        // system secrets
    Sessions = 4,       // user session
    System = 5,         // system settings
    Business = 6,       // business account
    Locations = 7,      // business locations
    DocExtraction = 8   // access to scan & vision api
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