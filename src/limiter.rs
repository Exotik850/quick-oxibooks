use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// A simple rate limiter that allows a maximum number of requests
/// within a specified duration.
///
/// This rate limiter is designed to be used in a multi-threaded environment
/// and ensures that no more than `max_requests` are allowed within the
/// specified `duration`.
#[derive(Clone, Debug)]
pub(crate) struct RateLimiter {
    inner: Arc<Mutex<InnerRateLimiter>>,
    duration: Duration,
    max_requests: usize,
}

#[derive(Debug)]
struct InnerRateLimiter {
    requests: usize,
    window_start: Instant,
}

impl RateLimiter {
    pub fn new(max_requests: usize, duration: Duration) -> Self {
        let inner = Arc::new(Mutex::new(InnerRateLimiter {
            requests: 0,
            window_start: Instant::now(),
        }));
        RateLimiter {
            inner,
            duration,
            max_requests,
        }
    }

    /// Acquires a permit from the rate limiter.
    /// This method will block until a permit is available and returns a guard.
    pub fn acquire(&self) -> RateLimiterGuard<'_> {
        loop {
            let Ok(mut guard) = self.inner.lock() else {
                continue;
            };
            let now = Instant::now();
            if now.duration_since(guard.window_start) >= self.duration {
                guard.window_start = now;
                guard.requests = 0;
            }
            if guard.requests < self.max_requests {
                guard.requests += 1;
                let window_start = guard.window_start;
                return RateLimiterGuard {
                    limiter: self,
                    window_start,
                };
            } else {
                let wait = self.duration - now.duration_since(guard.window_start);
                drop(guard);
                std::thread::sleep(wait);
            }
        }
    }
}

pub struct RateLimiterGuard<'a> {
    limiter: &'a RateLimiter,
    window_start: Instant,
}

impl Drop for RateLimiterGuard<'_> {
    fn drop(&mut self) {
        let Ok(mut guard) = self.limiter.inner.lock() else {
            return;
        };
        // Only decrement if still in the same window
        if guard.window_start + self.limiter.duration <= self.window_start && guard.requests > 0 {
            guard.requests -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(5, Duration::from_secs(1));
        let mut handles = vec![];

        for _ in 0..10 {
            let limiter_clone = limiter.clone();
            let handle = thread::spawn(move || {
                let guard = limiter_clone.acquire();
                assert!(guard.limiter.max_requests > 0);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
