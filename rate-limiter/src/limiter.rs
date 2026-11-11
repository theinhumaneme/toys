use std::time::Duration;

pub trait RateLimiter: Send + Sync {
    fn allow(&self, client_id: &str) -> bool;
    fn check(&self, client_id: &str) -> Option<Duration> {
        if self.allow(client_id) {
            Some(Duration::ZERO)
        } else {
            None
        }
    }
}
