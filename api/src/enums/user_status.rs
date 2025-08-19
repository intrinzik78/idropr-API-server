#[derive(Clone,Debug,PartialEq)]
pub enum UserAccountStatus {
    Disabled,   // 0
    Enabled,    // 1
    Suspended,  // 2
    Banned      // 3
}