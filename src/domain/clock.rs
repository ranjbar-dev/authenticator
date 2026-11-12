use crate::error::AppError;
use std::time::{SystemTime, UNIX_EPOCH};

/// Injectable UTC seconds. Idle locking must use a separate monotonic clock.
pub trait Clock {
    fn unix_seconds(&self) -> Result<u64, AppError>;
}

#[derive(Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn unix_seconds(&self) -> Result<u64, AppError> {
        unix_seconds_at(SystemTime::now())
    }
}

/// UTC conversion; local timezone and daylight-saving settings never enter OTP.
pub fn unix_seconds_at(time: SystemTime) -> Result<u64, AppError> {
    time.duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|_| AppError::ClockUnavailable)
}

/// Test clock never reads or changes the machine's time.
pub struct FixedClock(pub u64);

impl Clock for FixedClock {
    fn unix_seconds(&self) -> Result<u64, AppError> {
        Ok(self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_clock_is_injected_without_wall_clock_dependency() -> Result<(), AppError> {
        let clock: &dyn Clock = &FixedClock(59);
        assert_eq!(clock.unix_seconds()?, 59);
        Ok(())
    }
}
