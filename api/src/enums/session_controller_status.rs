use crate::types::SessionController;

#[derive(Debug)]
pub enum SessionControllerStatus {
    Disabled,
    Enabled(Box<SessionController>)
}