//! Recurrence expansion for local activities.
//!
//! Given a LocalActivityWithRecurrence and a time window [start, end),
//! returns all occurrence start times within that window.

use chrono::{DateTime, Datelike, Duration, TimeZone, Timelike, Utc};
use crate::domain::entities::local_activity::LocalActivityRecurrence;

/// Expand a recurrence rule into occurrence dates within [window_start, window_end).
/// Returns a Vec of (start, end) pairs, each shifted from the activity's base start/end.
pub fn expand_occurrences(
    base_start: DateTime<Utc>,
    base_end: DateTime<Utc>,
    rule: &LocalActivityRecurrence,
    window_start: DateTime<Utc>,
    window_end: DateTime<Utc>,
) -> Vec<(DateTime<Utc>, DateTime<Utc>)> {
    let duration = base_end - base_start;
    let interval = rule.interval_val.max(1) as i64;
    let mut occurrences = Vec::new();

    // Generate candidate dates
    let mut current = base_start;
    let max_count = rule.count.unwrap_or(i32::MAX) as usize;
    let until = rule.until.map(|d| {
        Utc.with_ymd_and_hms(d.year(), d.month(), d.day(), 23, 59, 59).unwrap()
    });

    let mut count = 0;
    // Safety limit to prevent infinite loops
    let max_iterations = 10_000;
    let mut iterations = 0;

    loop {
        iterations += 1;
        if iterations > max_iterations { break; }
        if count >= max_count { break; }
        if let Some(until_dt) = until {
            if current > until_dt { break; }
        }
        if current >= window_end { break; }

        // Check day-of-week filter
        let day_of_week_ok = match &rule.by_day_of_week {
            Some(days) if !days.is_empty() => {
                let weekday_num = current.weekday().num_days_from_monday() as i32;
                days.contains(&weekday_num)
            }
            _ => true,
        };

        // Check day-of-month filter
        let day_of_month_ok = match &rule.by_day_of_month {
            Some(days) if !days.is_empty() => {
                days.contains(&(current.day() as i32))
            }
            _ => true,
        };

        if day_of_week_ok && day_of_month_ok {
            if current >= window_start {
                occurrences.push((current, current + duration));
            }
            count += 1;
        }

        // Advance to next candidate
        current = match rule.freq.as_str() {
            "daily" => current + Duration::days(interval),
            "weekly" => current + Duration::weeks(interval),
            "monthly" => {
                let next_month = if current.month() == 12 {
                    current.with_year(current.year() + 1).unwrap().with_month(1).unwrap()
                } else {
                    current.with_month(current.month() + interval as u32).unwrap_or_else(|| {
                        // Handle month overflow (e.g., Jan 31 + 1 month)
                        let y = current.year() + ((current.month() as i32 - 1 + interval as i32) / 12) as i32;
                        let m = ((current.month() as i32 - 1 + interval as i32) % 12) as u32 + 1;
                        Utc.with_ymd_and_hms(y, m, 1, current.hour(), current.minute(), current.second()).unwrap()
                    })
                };
                next_month
            }
            "yearly" => {
                current.with_year(current.year() + interval as i32).unwrap_or(current + Duration::days(365))
            }
            _ => break,
        };
    }

    occurrences
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use uuid::Uuid;
    use crate::domain::entities::local_activity::LocalActivityRecurrence;

    fn make_rule(freq: &str, interval: i32, by_day_of_week: Option<Vec<i32>>) -> LocalActivityRecurrence {
        LocalActivityRecurrence {
            id: Uuid::new_v4(),
            local_activity_id: Uuid::new_v4(),
            freq: freq.to_string(),
            interval_val: interval,
            by_day_of_week,
            by_day_of_month: None,
            until: None,
            count: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_daily_recurrence() {
        let base_start = Utc.with_ymd_and_hms(2024, 1, 1, 9, 0, 0).unwrap();
        let base_end = Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();
        let rule = make_rule("daily", 1, None);
        let window_start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let window_end = Utc.with_ymd_and_hms(2024, 1, 8, 0, 0, 0).unwrap();

        let occurrences = expand_occurrences(base_start, base_end, &rule, window_start, window_end);
        assert_eq!(occurrences.len(), 7, "Should have 7 daily occurrences");
        assert_eq!(occurrences[0].0, base_start);
        assert_eq!(occurrences[6].0, Utc.with_ymd_and_hms(2024, 1, 7, 9, 0, 0).unwrap());
    }

    #[test]
    fn test_weekly_recurrence() {
        let base_start = Utc.with_ymd_and_hms(2024, 1, 1, 9, 0, 0).unwrap(); // Monday
        let base_end = base_start + Duration::hours(1);
        let rule = make_rule("weekly", 1, None);
        let window_start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let window_end = Utc.with_ymd_and_hms(2024, 1, 29, 0, 0, 0).unwrap();

        let occurrences = expand_occurrences(base_start, base_end, &rule, window_start, window_end);
        assert_eq!(occurrences.len(), 4, "Should have 4 weekly occurrences");
    }

    #[test]
    fn test_by_day_of_week_filter() {
        let base_start = Utc.with_ymd_and_hms(2024, 1, 1, 9, 0, 0).unwrap(); // Monday
        let base_end = base_start + Duration::hours(1);
        // Only Mon (0) and Wed (2)
        let rule = make_rule("daily", 1, Some(vec![0, 2]));
        let window_start = base_start;
        let window_end = Utc.with_ymd_and_hms(2024, 1, 8, 0, 0, 0).unwrap();

        let occurrences = expand_occurrences(base_start, base_end, &rule, window_start, window_end);
        // Jan 1 Mon=0 ✓, Jan 2 Tue=1 ✗, Jan 3 Wed=2 ✓, Jan 4 Thu=3 ✗, Jan 5 Fri=4 ✗, Jan 6 Sat=5 ✗, Jan 7 Sun=6 ✗
        assert_eq!(occurrences.len(), 2);
    }

    #[test]
    fn test_count_limit() {
        let base_start = Utc.with_ymd_and_hms(2024, 1, 1, 9, 0, 0).unwrap();
        let base_end = base_start + Duration::hours(1);
        let mut rule = make_rule("daily", 1, None);
        rule.count = Some(3);
        let window_start = base_start;
        let window_end = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

        let occurrences = expand_occurrences(base_start, base_end, &rule, window_start, window_end);
        assert_eq!(occurrences.len(), 3, "Should stop at count=3");
    }
}
