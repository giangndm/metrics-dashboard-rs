use std::sync::{atomic::Ordering, Arc};

use metrics::CounterFn;
use prometheus::core::{Atomic, AtomicU64};

#[derive(Debug, Clone)]
pub struct SimpleCounter {
    value: Arc<AtomicU64>,
}

impl SimpleCounter {
    pub fn value(&self) -> u64 {
        self.value.get()
    }
}

impl Default for SimpleCounter {
    fn default() -> Self {
        Self {
            value: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl CounterFn for SimpleCounter {
    fn increment(&self, value: u64) {
        self.value.inc_by_with_ordering(value, Ordering::SeqCst)
    }

    fn absolute(&self, value: u64) {
        self.value.swap(value, Ordering::SeqCst);
    }
}
