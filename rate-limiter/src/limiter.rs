pub trait RateLimiter: Send + Sync {
    fn allow(&self, client_id: &str) -> bool;
}
