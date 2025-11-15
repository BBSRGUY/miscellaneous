//! Scheduling support for jobs

use crate::error::{JobError, JobResult};
use std::time::Duration;

/// Schedule interval types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleInterval {
    /// Run once immediately
    Once,

    /// Run every N seconds
    Seconds(u64),

    /// Run every N minutes
    Minutes(u64),

    /// Run every N hours
    Hours(u64),

    /// Run every N days
    Days(u64),

    /// Cron expression (requires 'scheduling' feature)
    #[cfg(feature = "scheduling")]
    Cron(String),
}

impl ScheduleInterval {
    /// Convert interval to duration (for simple intervals)
    pub fn to_duration(&self) -> Option<Duration> {
        match self {
            Self::Once => None,
            Self::Seconds(n) => Some(Duration::from_secs(*n)),
            Self::Minutes(n) => Some(Duration::from_secs(n * 60)),
            Self::Hours(n) => Some(Duration::from_secs(n * 3600)),
            Self::Days(n) => Some(Duration::from_secs(n * 86400)),
            #[cfg(feature = "scheduling")]
            Self::Cron(_) => None, // Cron needs special handling
        }
    }

    /// Check if this is a repeating schedule
    pub fn is_repeating(&self) -> bool {
        !matches!(self, Self::Once)
    }
}

/// A schedule for job execution
pub struct Schedule {
    /// The schedule interval
    pub interval: ScheduleInterval,

    /// Whether the schedule is enabled
    pub enabled: bool,
}

impl Schedule {
    /// Create a new schedule
    pub fn new(interval: ScheduleInterval) -> Self {
        Self {
            interval,
            enabled: true,
        }
    }

    /// Create a one-time schedule
    pub fn once() -> Self {
        Self::new(ScheduleInterval::Once)
    }

    /// Create a schedule that runs every N seconds
    pub fn every_seconds(n: u64) -> Self {
        Self::new(ScheduleInterval::Seconds(n))
    }

    /// Create a schedule that runs every N minutes
    pub fn every_minutes(n: u64) -> Self {
        Self::new(ScheduleInterval::Minutes(n))
    }

    /// Create a schedule that runs every N hours
    pub fn every_hours(n: u64) -> Self {
        Self::new(ScheduleInterval::Hours(n))
    }

    /// Create a schedule that runs every N days
    pub fn every_days(n: u64) -> Self {
        Self::new(ScheduleInterval::Days(n))
    }

    /// Create a cron-based schedule
    #[cfg(feature = "scheduling")]
    pub fn cron(expr: impl Into<String>) -> JobResult<Self> {
        let expr = expr.into();
        // Validate cron expression
        cron::Schedule::from_str(&expr)
            .map_err(|e| JobError::InvalidSchedule(e.to_string()))?;

        Ok(Self::new(ScheduleInterval::Cron(expr)))
    }

    /// Enable the schedule
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable the schedule
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if the schedule is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Parse a schedule from a configuration string
///
/// Formats:
/// - "once" - run once
/// - "30s" - every 30 seconds
/// - "5m" - every 5 minutes
/// - "2h" - every 2 hours
/// - "1d" - every 1 day
/// - "0 0 * * *" - cron expression (requires 'scheduling' feature)
pub fn parse_schedule(s: &str) -> JobResult<Schedule> {
    let s = s.trim();

    if s == "once" {
        return Ok(Schedule::once());
    }

    // Try to parse as interval (e.g., "30s", "5m", "2h", "1d")
    if let Some(duration) = parse_duration(s) {
        let interval = if s.ends_with('s') {
            ScheduleInterval::Seconds(duration)
        } else if s.ends_with('m') {
            ScheduleInterval::Minutes(duration)
        } else if s.ends_with('h') {
            ScheduleInterval::Hours(duration)
        } else if s.ends_with('d') {
            ScheduleInterval::Days(duration)
        } else {
            return Err(JobError::InvalidSchedule(format!(
                "Invalid duration format: {}",
                s
            )));
        };

        return Ok(Schedule::new(interval));
    }

    // Try to parse as cron expression
    #[cfg(feature = "scheduling")]
    {
        Schedule::cron(s)
    }

    #[cfg(not(feature = "scheduling"))]
    {
        Err(JobError::InvalidSchedule(format!(
            "Unknown schedule format: {}",
            s
        )))
    }
}

/// Parse a duration string like "30s", "5m", "2h", "1d"
fn parse_duration(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let (num_str, suffix) = s.split_at(s.len() - 1);
    let num: u64 = num_str.parse().ok()?;

    match suffix {
        "s" | "m" | "h" | "d" => Some(num),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_schedule() {
        assert_eq!(parse_schedule("once").unwrap().interval, ScheduleInterval::Once);
        assert_eq!(parse_schedule("30s").unwrap().interval, ScheduleInterval::Seconds(30));
        assert_eq!(parse_schedule("5m").unwrap().interval, ScheduleInterval::Minutes(5));
        assert_eq!(parse_schedule("2h").unwrap().interval, ScheduleInterval::Hours(2));
        assert_eq!(parse_schedule("1d").unwrap().interval, ScheduleInterval::Days(1));
    }

    #[test]
    fn test_schedule_to_duration() {
        assert_eq!(Schedule::every_seconds(30).interval.to_duration(), Some(Duration::from_secs(30)));
        assert_eq!(Schedule::every_minutes(5).interval.to_duration(), Some(Duration::from_secs(300)));
        assert_eq!(Schedule::every_hours(2).interval.to_duration(), Some(Duration::from_secs(7200)));
    }
}
