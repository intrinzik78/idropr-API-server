#[derive(Clone,Debug,Default,PartialEq)]
pub enum MasterPassword {
    #[default]
    None,
    Some(String)
}