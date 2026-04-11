//! Mock calendar provider — returns deterministic fake events for local dev/testing.

use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};
use serde_json::json;
use uuid::Uuid;

use crate::domain::entities::source_event::SourceEvent;

pub fn mock_events_for_range(
    calendar_source_id: Uuid,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Vec<SourceEvent> {
    let mut events = Vec::new();
    let now = Utc::now();

    // Define a set of recurring mock events spread over the week
    let templates = vec![
        ("Team standup", 9, 0, 30, "#4A90D9", "alice@example.com"),
        ("Swim practice", 17, 30, 60, "#50C878", "bob@example.com"),
        ("Family dinner", 18, 30, 90, "#E85555", "alice@example.com"),
        ("Morning run", 7, 0, 45, "#F5A623", "charlie@example.com"),
        ("Weekly planning", 10, 0, 60, "#9B59B6", "alice@example.com"),
    ];

    let mut day = start.date_naive();
    while day < end.date_naive() {
        for (i, (title, hour, minute, duration_mins, _color, organizer)) in templates.iter().enumerate() {
            // Only some events appear each day (simulate realistic sparseness)
            if (day.num_days_from_ce() as usize + i) % 3 != 0 {
                continue;
            }

            let event_start = Utc.with_ymd_and_hms(day.year(), day.month(), day.day(), *hour as u32, *minute as u32, 0)
                .unwrap();
            let event_end = event_start + Duration::minutes(*duration_mins);

            if event_start < start || event_start >= end {
                day = day.succ_opt().unwrap();
                continue;
            }

            let event_id = format!("mock-{}-{}", i, day);
            let ical_uid = format!("{}@mock.calendar.google.com", event_id);

            events.push(SourceEvent {
                id: Uuid::new_v4(),
                calendar_source_id,
                google_event_id: event_id.clone(),
                ical_uid: Some(ical_uid.clone()),
                title: title.to_string(),
                description: Some(format!("Mock event: {}", title)),
                location: None,
                start_at: event_start,
                end_at: event_end,
                is_all_day: false,
                recurrence_rule: None,
                recurring_event_id: None,
                organizer: Some(organizer.to_string()),
                attendees: Some(vec![organizer.to_string()]),
                raw_json: json!({
                    "id": event_id,
                    "summary": title,
                    "start": { "dateTime": event_start.to_rfc3339() },
                    "end": { "dateTime": event_end.to_rfc3339() },
                    "iCalUID": ical_uid,
                    "organizer": { "email": organizer },
                    "mock": true
                }),
                synced_at: now,
                created_at: now,
                updated_at: now,
            });
        }
        day = day.succ_opt().unwrap();
    }

    events
}
