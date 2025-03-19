use std::{ops::Deref, sync::Arc, time::Duration};

use async_lock::Semaphore;

pub(crate) struct RateLimiter {
    semaphore: Arc<Semaphore>,
}

impl RateLimiter {
    pub fn new(max_requests: usize, duration: Duration) -> Self {
        let semaphore = Arc::new(Semaphore::new(max_requests));
        let semaphore_clone = semaphore.clone();
        std::thread::spawn(move || {
            loop {
                if semaphore_clone.try_acquire().is_none() {
                  semaphore_clone.add_permits(max_requests); 
                }
                std::thread::sleep(duration);
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
