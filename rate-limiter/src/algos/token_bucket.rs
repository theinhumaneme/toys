use std::{collections::HashMap, sync::Mutex};

use tokio::time::Instant;

use crate::limiter::RateLimiter;

struct Bucket {
    tokens: f64,
    last_refilled: Instant,
}

pub struct TokenBucket {
    capacity: f64,
    refill_rate: f64,
    buckets: Mutex<HashMap<String, Bucket>>,
}

impl TokenBucket {
    pub fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            capacity,
            refill_rate,
            buckets: Mutex::new(HashMap::new()),
        }
    }
}

impl RateLimiter for TokenBucket {
    fn allow(&self, client_id: &str) -> bool {
        let now = Instant::now();
        let mut buckets = self.buckets.lock().unwrap();
        let bucket = buckets
            .entry(client_id.to_owned())
            .or_insert_with(|| Bucket {
                tokens: self.capacity,
                last_refilled: Instant::now(),
            });
        let elapsed = now - bucket.last_refilled;
        bucket.last_refilled = now;
        bucket.tokens =
            (bucket.tokens + (elapsed.as_secs_f64() * self.refill_rate)).min(self.capacity);
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            return true;
        } else {
            return false;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use super::*;

    #[test]
    fn allows_up_to_capacity_then_rejects() {
        let tb = TokenBucket::new(3.0, 1.0);
        assert_eq!(tb.allow("1"), true);
        assert_eq!(tb.allow("1"), true);
        assert_eq!(tb.allow("1"), true);
        assert_eq!(tb.allow("1"), false);
    }

    #[test]
    fn allows_diff_clients() {
        let tb = TokenBucket::new(1.0, 1.0);
        tb.allow("1");
        assert_eq!(tb.allow("1"), false);
        assert_eq!(tb.allow("2"), true);
    }

    #[test]
    fn allow_client_after_refill() {
        let tb = TokenBucket::new(1.0, 10.0);
        assert_eq!(tb.allow("1"), true);
        assert_eq!(tb.allow("1"), false);
        sleep(Duration::from_millis(150));
        assert_eq!(tb.allow("1"), true);
    }

    #[test]
    fn capacity_check() {
        let tb = TokenBucket::new(2.0, 100.0);
        assert_eq!(tb.allow("1"), true);
        assert_eq!(tb.allow("1"), true);
        assert_eq!(tb.allow("1"), false);
        sleep(Duration::from_millis(500));
        assert_eq!(tb.allow("1"), true);
        assert_eq!(tb.allow("1"), true);
        assert_eq!(tb.allow("1"), false);
    }
}
