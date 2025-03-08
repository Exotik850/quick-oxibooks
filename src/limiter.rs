use std::{ops::Deref, sync::Arc, time::Duration};

use tokio::sync::Semaphore;

pub(crate) struct RateLimiter {
    semaphore: Arc<Semaphore>,
}

impl RateLimiter {
    pub fn new(max_requests: usize, duration: Duration) -> Self {
        let semaphore = Arc::new(Semaphore::new(max_requests));
        let semaphore_clone = semaphore.clone();
        tokio::spawn(async move {
            loop {
                let permits = semaphore_clone.available_permits();
                if permits < max_requests {
                    semaphore_clone.add_permits(max_requests - permits);
                }
                tokio::time::sleep(duration).await;
            }
        });
        RateLimiter { semaphore }
    }
}

impl Deref for RateLimiter {
    type Target = Semaphore;
    fn deref(&self) -> &Self::Target {
        &self.semaphore
    }
}
