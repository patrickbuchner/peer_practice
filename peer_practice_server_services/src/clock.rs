use chrono::{DateTime, Duration, Utc};
use std::sync::{Arc, Mutex};

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

pub type ClockRef = Arc<dyn Clock>;

#[derive(Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

pub fn system_clock() -> ClockRef {
    Arc::new(SystemClock)
}

pub struct ManualClock {
    current: Mutex<DateTime<Utc>>,
}

impl ManualClock {
    pub fn new(start: DateTime<Utc>) -> Self {
        Self {
            current: Mutex::new(start),
        }
    }

    pub fn set(&self, now: DateTime<Utc>) {
        *self.current.lock().expect("manual clock lock poisoned") = now;
    }

    pub fn advance(&self, delta: Duration) {
        let mut current = self.current.lock().expect("manual clock lock poisoned");
        *current = *current + delta;
    }
}

impl Clock for ManualClock {
    fn now(&self) -> DateTime<Utc> {
        *self.current.lock().expect("manual clock lock poisoned")
    }
}
