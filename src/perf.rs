/// Performance monitoring utilities

use std::time::{Duration, Instant};

/// Time a block of code and warn if it exceeds threshold
pub struct PerfTimer {
    name: &'static str,
    start: Instant,
    threshold_ms: u64,
}

impl PerfTimer {
    /// Create a new timer with default 200ms threshold
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: Instant::now(),
            threshold_ms: 200,
        }
    }

    /// Create timer with custom threshold
    pub fn with_threshold(name: &'static str, threshold_ms: u64) -> Self {
        Self {
            name,
            start: Instant::now(),
            threshold_ms,
        }
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Check if threshold exceeded
    pub fn is_slow(&self) -> bool {
        self.elapsed().as_millis() > self.threshold_ms as u128
    }
}

impl Drop for PerfTimer {
    fn drop(&mut self) {
        let elapsed = self.elapsed();
        if elapsed.as_millis() > self.threshold_ms as u128 {
            eprintln!(
                "PERF WARNING: {} took {}ms (threshold: {}ms)",
                self.name,
                elapsed.as_millis(),
                self.threshold_ms
            );
        }
    }
}

/// Macro for easy timing
#[macro_export]
macro_rules! time_it {
    ($name:expr, $block:expr) => {{
        let _timer = $crate::perf::PerfTimer::new($name);
        $block
    }};
}
