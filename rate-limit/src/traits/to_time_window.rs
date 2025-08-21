use crate::enums::{TimeWindow,RateLimitError};

pub trait ToTimeWindow {
    fn to_time_window(self) -> Result<TimeWindow,RateLimitError>;
}

impl ToTimeWindow for &String {
    fn to_time_window(self) -> Result<TimeWindow,RateLimitError> {
        match self.as_str() {
            "SECOND" => Ok(TimeWindow::Second),
            "MINUTE" => Ok(TimeWindow::Minute),
            "HOUR"   => Ok(TimeWindow::Hour),
            "DAY"    => Ok(TimeWindow::Day),
            _ => Err(RateLimitError::TimeWindowOutOfBounds)
        }
    }
}