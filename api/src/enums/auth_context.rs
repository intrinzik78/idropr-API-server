use crate::enums::User;

#[derive(Clone,Debug)]
pub enum AuthContext {
    None,
    Some(Box<User>)
}