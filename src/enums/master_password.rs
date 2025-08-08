#[derive(Clone,Debug,PartialEq)]
pub enum MasterPassword {
    None,
    Some(String)
}