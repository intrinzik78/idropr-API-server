use chrono::{DateTime, Local, Utc};

pub trait ToLocalTime {
    fn to_local_time(&self) -> DateTime<Local>;
}

impl ToLocalTime for DateTime<Utc> {
    fn to_local_time(&self) -> DateTime<Local> {
        self.with_timezone(&Local)
    }
}