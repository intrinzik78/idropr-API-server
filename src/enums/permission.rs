#[derive(Copy,Clone,Debug,PartialEq)]
pub enum Permission {
    None,       // default permissions state
    Granted,    // explicitly granted
}