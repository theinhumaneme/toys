use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use crate::limiter::RateLimiter;

struct ClientQueue {
    next_free: Instant, // one client's state: when its next slot is
}

pub struct LeakyBucket {
    capacity: f64,
    leak_rate: f64,
    buckets: Mutex<HashMap<String, ClientQueue>>,
}

impl LeakyBucket {
    pub fn new(capacity: f64, leak_rate: f64) -> Self {
        Self {
            capacity,
            leak_rate,
            buckets: Mutex::new(HashMap::new()),
        }
    }
}

impl RateLimiter for LeakyBucket {
    fn allow(&self, client_id: &str) -> bool {
        self.check(client_id).is_some()
    }
    fn check(&self, client_id: &str) -> Option<Duration> {
        let now = Instant::now();
        let mut buckets = self.buckets.lock().unwrap();
        let client_queue = buckets
            .entry(client_id.to_owned())
            .or_insert_with(|| ClientQueue { next_free: now });
        let start = now.max(client_queue.next_free);
        let wait = start - now;
        let ahead = wait.as_secs_f64() * self.leak_rate;
        if ahead >= self.capacity {
            return None;
        } else {
            client_queue.next_free = start + Duration::from_secs_f64(1.0 / self.leak_rate);
            Some(wait)
        }
    }
}
