use crate::types::sessions::SessionController;

#[derive(Debug)]
pub enum SessionControllerStatus {
    Disabled,
    Enabled(Box<SessionController>)
}