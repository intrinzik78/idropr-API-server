#[derive(Clone,Debug,PartialEq)]
pub enum TimerStatus {
    Running,
    Expired,
    Poison
}