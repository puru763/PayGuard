use chrono::{DateTime, Utc};

/// A trait abstracting time so our business logic remains deterministic and testable.
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

/// The real production clock using the system time.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// A mock/frozen clock for unit testing and historical replays.
pub struct MockClock {
    current_time: std::sync::Mutex<DateTime<Utc>>,
}

impl MockClock {
    pub fn new(initial_time: DateTime<Utc>) -> Self {
        Self {
            current_time: std::sync::Mutex::new(initial_time),
        }
    }

    pub fn advance(&self, duration: chrono::Duration) {
        // Production-safe mutex lock with poison recovery instead of raw unwrap
        let mut time = match self.current_time.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        *time = *time + duration;
    }
}

impl Clock for MockClock {
    fn now(&self) -> DateTime<Utc> {
        let time = match self.current_time.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        *time
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_mock_clock() {
        let fixed_time = Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap();
        let clock = MockClock::new(fixed_time);

        assert_eq!(clock.now(), fixed_time);

        // Advance time by 5 minutes
        clock.advance(chrono::Duration::minutes(5));
        let expected_later = Utc.with_ymd_and_hms(2026, 1, 1, 12, 5, 0).unwrap();
        assert_eq!(clock.now(), expected_later);
    }
}
