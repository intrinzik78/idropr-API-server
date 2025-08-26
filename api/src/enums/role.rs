#[derive(Clone,Debug)]
pub enum Role {
    SysAdmin,       // manage the entire system
    SysMod,         // manage all users and their content
    Editor,         // manage select groups of users and their content
    User,           // manage self
}