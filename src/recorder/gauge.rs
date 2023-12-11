use std::sync::Arc;

use metrics::GaugeFn;
use prometheus::core::{Atomic, AtomicF64};

#[derive(Debug, Clone)]
pub struct SimpleGauge {
    value: Arc<AtomicF64>,
}

impl SimpleGauge {
    pub fn value(&self) -> f64 {
        self.value.get()
    }
}

impl Default for SimpleGauge {
    fn default() -> Self {
        Self {
            value: Arc::new(AtomicF64::new(0.0)),
        }
    }
}

impl GaugeFn for SimpleGauge {
    fn increment(&self, value: f64) {
        self.value.inc_by(value);
    }

    fn decrement(&self, value: f64) {
        self.value.dec_by(value);
    }

    fn set(&self, value: f64) {
        self.value.set(value);
    }
}
