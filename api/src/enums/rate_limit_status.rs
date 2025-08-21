use rate_limit::RateLimiter;

#[derive(Debug)]
pub enum RateLimiterStatus {
    Disabled,
    Enabled(Box<RateLimiter>)
}