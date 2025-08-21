#[derive(Clone,Debug,PartialEq)]
pub enum RefillRate {
    PerSecond(f32),
    PerMinute(f32),
    PerHour(f32),
    PerDay(f32)
}