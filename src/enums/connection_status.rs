/// database connection state

#[derive(Copy,Clone,Debug,PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected
}