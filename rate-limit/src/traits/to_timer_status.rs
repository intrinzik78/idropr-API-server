use std::time::Instant;

use crate::{enums::TimerStatus,types::Timer};

pub trait ToTimerStatus {
    fn to_timer_status(self) -> TimerStatus;
}

impl ToTimerStatus for Timer {
    fn to_timer_status(self) -> TimerStatus {
        let expires_opt = self.expires(); 
        
        if let Some(expires) = *expires_opt {
            let now = Instant::now();
            
            match now.gt(&expires) {
                true => TimerStatus::Expired,
                false => TimerStatus::Running
            }
        } else {
            TimerStatus::Poison
        }
    }
}