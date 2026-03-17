use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default)]
pub struct Timer {
    end_time: Option<Instant>,
}

impl Timer {
    pub fn new() -> Self {
        Self { end_time: None }
    }

    pub fn start(&mut self, duration: Duration) {
        self.end_time = Some(Instant::now() + duration);
    }

    pub fn timed_out(&self) -> bool {
        match self.end_time {
            Some(end) => Instant::now() > end,
            None => false,
        }
    }

    pub fn remaining(&self) -> Option<Duration> {
        self.end_time.map(|end| end.saturating_duration_since(Instant::now()))
    }

    pub fn cancel(&mut self) {
        self.end_time = None;
    }
}
