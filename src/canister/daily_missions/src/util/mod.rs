use std::time::{Duration, SystemTime};

/// Utility functions for daily missions canister

/// Check if a given timestamp is from today
pub fn is_same_day(timestamp: SystemTime, reference: SystemTime) -> bool {
    let ts_days = timestamp
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
        / (24 * 60 * 60);

    let ref_days = reference
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
        / (24 * 60 * 60);

    ts_days == ref_days
}

/// Check if a timestamp is from yesterday relative to reference
pub fn is_yesterday(timestamp: SystemTime, reference: SystemTime) -> bool {
    let ts_days = timestamp
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
        / (24 * 60 * 60);

    let ref_days = reference
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
        / (24 * 60 * 60);

    ts_days == ref_days - 1
}

/// Get the number of days between two timestamps
pub fn days_between(start: SystemTime, end: SystemTime) -> u64 {
    if end < start {
        return 0;
    }

    let start_days = start
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
        / (24 * 60 * 60);

    let end_days = end
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
        / (24 * 60 * 60);

    end_days - start_days
}

/// Get the start of the day for a given timestamp
pub fn get_day_start(timestamp: SystemTime) -> SystemTime {
    let seconds_since_epoch = timestamp
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs();

    let seconds_in_day = 24 * 60 * 60;
    let day_start_seconds = (seconds_since_epoch / seconds_in_day) * seconds_in_day;

    SystemTime::UNIX_EPOCH + Duration::from_secs(day_start_seconds)
}

/// Get the end of the day for a given timestamp
pub fn get_day_end(timestamp: SystemTime) -> SystemTime {
    let day_start = get_day_start(timestamp);
    day_start + Duration::from_secs(24 * 60 * 60 - 1)
}

/// Calculate hours remaining until next day (UTC)
pub fn hours_until_next_day(timestamp: SystemTime) -> u32 {
    let seconds_since_epoch = timestamp
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs();

    let seconds_in_day = 24 * 60 * 60;
    let seconds_into_day = seconds_since_epoch % seconds_in_day;
    let seconds_until_next_day = seconds_in_day - seconds_into_day;

    (seconds_until_next_day / 3600) as u32
}

/// Validate if a reward can be claimed based on progress and requirements
pub fn can_claim_reward(current_progress: u32, target: u32, already_claimed: bool) -> bool {
    current_progress >= target && !already_claimed
}

/// Format duration in a human-readable way
pub fn format_duration_hours(hours: u32) -> String {
    if hours == 0 {
        "Now".to_string()
    } else if hours == 1 {
        "1 hour".to_string()
    } else if hours < 24 {
        format!("{} hours", hours)
    } else {
        let days = hours / 24;
        let remaining_hours = hours % 24;
        if remaining_hours == 0 {
            if days == 1 {
                "1 day".to_string()
            } else {
                format!("{} days", days)
            }
        } else {
            format!("{} days {} hours", days, remaining_hours)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[test]
    fn test_is_same_day() {
        let base_time = UNIX_EPOCH + Duration::from_secs(24 * 60 * 60); // Day 1
        let same_day = base_time + Duration::from_secs(3600); // 1 hour later same day
        let different_day = base_time + Duration::from_secs(24 * 60 * 60); // Next day

        assert!(is_same_day(base_time, same_day));
        assert!(!is_same_day(base_time, different_day));
    }

    #[test]
    fn test_is_yesterday() {
        let today = UNIX_EPOCH + Duration::from_secs(2 * 24 * 60 * 60); // Day 2
        let yesterday = UNIX_EPOCH + Duration::from_secs(24 * 60 * 60); // Day 1
        let two_days_ago = UNIX_EPOCH; // Day 0

        assert!(is_yesterday(yesterday, today));
        assert!(!is_yesterday(two_days_ago, today));
    }

    #[test]
    fn test_days_between() {
        let start = UNIX_EPOCH;
        let end = UNIX_EPOCH + Duration::from_secs(3 * 24 * 60 * 60);

        assert_eq!(days_between(start, end), 3);
        assert_eq!(days_between(end, start), 0);
    }

    #[test]
    fn test_can_claim_reward() {
        assert!(can_claim_reward(10, 5, false));
        assert!(!can_claim_reward(3, 5, false));
        assert!(!can_claim_reward(10, 5, true));
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration_hours(0), "Now");
        assert_eq!(format_duration_hours(1), "1 hour");
        assert_eq!(format_duration_hours(5), "5 hours");
        assert_eq!(format_duration_hours(24), "1 day");
        assert_eq!(format_duration_hours(25), "1 days 1 hours");
        assert_eq!(format_duration_hours(48), "2 days");
    }
}
