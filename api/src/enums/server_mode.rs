/// current server mode
#[derive(Copy,Clone,Debug,PartialEq)]
pub enum ServerMode {
    Development,    // offline development
    Maintenance,    // online, api requests denied
    Production,     // online, api requests accepted
}