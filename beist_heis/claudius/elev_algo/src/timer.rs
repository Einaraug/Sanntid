use std::time::Instant;

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
        self.end_time = Some(Instant::now() + std::time::Duration::from_secs_f64(duration_secs));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_timer_new_not_active() {
        let timer = Timer::new();
        assert!(!timer.timed_out());
    }

    #[test]
    fn test_timer_start_not_immediately_timed_out() {
        let mut timer = Timer::new();
        timer.start(1.0);
        assert!(!timer.timed_out());
    }

    #[test]
    fn test_timer_times_out() {
        let mut timer = Timer::new();
        timer.start(0.05); // 50ms
        sleep(Duration::from_millis(60));
        assert!(timer.timed_out());
    }

    #[test]
    fn test_timer_stop_prevents_timeout() {
        let mut timer = Timer::new();
        timer.start(0.05);
        timer.stop();
        sleep(Duration::from_millis(60));
        assert!(!timer.timed_out());
    }
}
