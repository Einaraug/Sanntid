use std::time::Instant;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Timer {
    end_time: Option<Instant>,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            end_time: None,
        }
    }

    pub fn start(&mut self, duration_secs: f64) {
        self.end_time = Some(Instant::now() + Duration::from_secs_f64(duration_secs));
    }

    pub fn stop(&mut self) {
        self.end_time = None;
    }

    pub fn timed_out(&self) -> bool {
        match self.end_time {
            Some(end) => Instant::now() > end,
            None => false,
        }
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
