use chrono::{TimeZone, Utc};
use uuid::Uuid;
use serde_json::json;
use crate::domain::dedupe::{group_events, normalize_title, DupeTier};
use crate::domain::entities::source_event::SourceEvent;

fn make_event(
    title: &str,
    start_hour: u32,
    ical_uid: Option<&str>,
    organizer: Option<&str>,
    attendees: Option<Vec<&str>>,
) -> SourceEvent {
    let start = Utc.with_ymd_and_hms(2024, 6, 15, start_hour, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2024, 6, 15, start_hour + 1, 0, 0).unwrap();
    let now = Utc::now();
    SourceEvent {
        id: Uuid::new_v4(),
        calendar_source_id: Uuid::new_v4(),
        google_event_id: format!("ev-{}", Uuid::new_v4()),
        ical_uid: ical_uid.map(|s| s.to_string()),
        title: title.to_string(),
        description: None,
        location: None,
        start_at: start,
        end_at: end,
        is_all_day: false,
        recurrence_rule: None,
        recurring_event_id: None,
        organizer: organizer.map(|s| s.to_string()),
        attendees: attendees.map(|v| v.into_iter().map(|s| s.to_string()).collect()),
        raw_json: json!({}),
        synced_at: now,
        created_at: now,
        updated_at: now,
    }
}

fn make_event_at(
    title: &str,
    day: u32,
    hour: u32,
    minute: u32,
    ical_uid: Option<&str>,
    organizer: Option<&str>,
) -> SourceEvent {
    let start = Utc.with_ymd_and_hms(2024, 6, day, hour, minute, 0).unwrap();
    let end = start + chrono::Duration::hours(1);
    let now = Utc::now();
    SourceEvent {
        id: Uuid::new_v4(),
        calendar_source_id: Uuid::new_v4(),
        google_event_id: format!("ev-{}", Uuid::new_v4()),
        ical_uid: ical_uid.map(|s| s.to_string()),
        title: title.to_string(),
        description: None,
        location: None,
        start_at: start,
        end_at: end,
        is_all_day: false,
        recurrence_rule: None,
        recurring_event_id: None,
        organizer: organizer.map(|s| s.to_string()),
        attendees: None,
        raw_json: json!({}),
        synced_at: now,
        created_at: now,
        updated_at: now,
    }
}

// ---- normalize_title ----

#[test]
fn test_normalize_title_lowercases() {
    assert_eq!(normalize_title("Team Standup"), "team standup");
}

#[test]
fn test_normalize_title_collapses_whitespace() {
    assert_eq!(normalize_title("team  standup"), "team standup");
    assert_eq!(normalize_title("  hello  world  "), "hello world");
}

#[test]
fn test_normalize_title_empty() {
    assert_eq!(normalize_title(""), "");
}

// ---- group_events: edge cases ----

#[test]
fn test_empty_events() {
    let groups = group_events(&[]);
    assert!(groups.is_empty());
}

#[test]
fn test_single_event_no_dup() {
    let events = vec![make_event("Standup", 9, None, None, None)];
    let groups = group_events(&events);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].members.len(), 1);
    assert!(groups[0].dupe_tier.is_none());
}

#[test]
fn test_two_unrelated_events() {
    let events = vec![
        make_event("Standup", 9, None, None, None),
        make_event("Lunch", 12, None, None, None),
    ];
    let groups = group_events(&events);
    assert_eq!(groups.len(), 2);
    for g in &groups {
        assert!(g.dupe_tier.is_none());
        assert_eq!(g.members.len(), 1);
    }
}

// ---- Tier 1: exact duplicates ----

#[test]
fn test_tier1_exact_same_ical_uid_and_start() {
    let uid = "abc123@google.com";
    let e1 = make_event("Standup", 9, Some(uid), Some("alice@example.com"), None);
    let mut e2 = make_event("Standup", 9, Some(uid), Some("alice@example.com"), None);
    // Same start time
    e2.start_at = e1.start_at;
    e2.end_at = e1.end_at;
    // Same calendar source to ensure dedup is cross-calendar
    let events = vec![e1, e2];
    let groups = group_events(&events);
    assert_eq!(groups.len(), 1, "exact dupe should produce 1 group");
    assert_eq!(groups[0].members.len(), 2);
    assert_eq!(groups[0].dupe_tier, Some(DupeTier::Exact));
}

#[test]
fn test_tier1_same_uid_different_starts_are_not_dupes() {
    let uid = "recurring@google.com";
    let e1 = make_event_at("Weekly sync", 1, 9, 0, Some(uid), None);
    let e2 = make_event_at("Weekly sync", 8, 9, 0, Some(uid), None);
    // Same UID but different occurrence dates → not exact dupes
    let events = vec![e1, e2];
    let groups = group_events(&events);
    // They could match on strong/probable, but their starts differ by 7 days so no tier2/3
    assert_eq!(groups.len(), 2);
}

// ---- Tier 2: strong duplicates ----

#[test]
fn test_tier2_strong_same_title_close_times_same_organizer() {
    let e1 = make_event_at("Team standup", 15, 9, 0, None, Some("alice@example.com"));
    let e2 = make_event_at("Team standup", 15, 9, 3, None, Some("alice@example.com")); // 3 min off
    let events = vec![e1, e2];
    let groups = group_events(&events);
    assert_eq!(groups.len(), 1, "should be grouped as strong dupe");
    assert_eq!(groups[0].dupe_tier, Some(DupeTier::Strong));
}

#[test]
fn test_tier2_strong_does_not_match_different_organizers() {
    let e1 = make_event_at("Team standup", 15, 9, 0, None, Some("alice@example.com"));
    let e2 = make_event_at("Team standup", 15, 9, 1, None, Some("bob@example.com"));
    let events = vec![e1, e2];
    let groups = group_events(&events);
    // Different organizers → not strong dupe; title+time match but organizer mismatch
    // Actually our logic says: org_ok = true only if BOTH are Some and equal
    // With Some("alice") vs Some("bob") → org_ok = false → not strong dupe
    // They could fall into probable if attendees overlap, but attendees are None here
    assert_eq!(groups.len(), 2);
}

#[test]
fn test_tier2_strong_requires_title_match() {
    let e1 = make_event_at("Team standup", 15, 9, 0, None, Some("alice@example.com"));
    let e2 = make_event_at("Team sync", 15, 9, 1, None, Some("alice@example.com"));
    let events = vec![e1, e2];
    let groups = group_events(&events);
    assert_eq!(groups.len(), 2, "different titles should not be grouped");
}

// ---- Tier 3: probable duplicates ----

#[test]
fn test_tier3_probable_same_title_same_hour_overlapping_attendees() {
    let mut e1 = make_event_at("All hands", 15, 14, 0, None, Some("organizer@example.com"));
    e1.attendees = Some(vec!["alice@example.com".to_string(), "bob@example.com".to_string()]);
    let mut e2 = make_event_at("All hands", 15, 14, 15, None, Some("organizer@example.com"));
    e2.attendees = Some(vec!["alice@example.com".to_string(), "charlie@example.com".to_string()]);
    // e1 and e2: same normalized title, same day, same hour, overlapping attendee (alice)
    // organizer matches so strong tier should catch them first (within 5 min? 15 min diff → no)
    // 15 min diff = not strong; check probable
    let events = vec![e1, e2];
    let groups = group_events(&events);
    // 15 min apart → not strong; same title + same hour + overlapping attendee → probable
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].dupe_tier, Some(DupeTier::Probable));
}

#[test]
fn test_tier3_requires_overlapping_attendees() {
    let mut e1 = make_event_at("Planning", 15, 10, 0, None, None);
    e1.attendees = Some(vec!["alice@example.com".to_string()]);
    let mut e2 = make_event_at("Planning", 15, 10, 20, None, None);
    e2.attendees = Some(vec!["bob@example.com".to_string()]); // no overlap
    let events = vec![e1, e2];
    let groups = group_events(&events);
    // No attendee overlap → not probable
    assert_eq!(groups.len(), 2);
}

// ---- Primary event marking ----

#[test]
fn test_exactly_one_primary_per_group() {
    let uid = "test-uid@google.com";
    let e1 = make_event("Standup", 9, Some(uid), None, None);
    let mut e2 = make_event("Standup", 9, Some(uid), None, None);
    e2.start_at = e1.start_at;
    e2.end_at = e1.end_at;
    let e3 = make_event_at("Team standup", 20, 9, 0, None, Some("a@b.com"));
    let mut e4 = make_event_at("Team standup", 20, 9, 2, None, Some("a@b.com"));
    e4.start_at = e3.start_at + chrono::Duration::minutes(2);

    let events = vec![e1, e2, e3, e4];
    let groups = group_events(&events);
    for group in &groups {
        let primary_count = group.members.iter().filter(|(_, p)| *p).count();
        assert_eq!(primary_count, 1, "Each group should have exactly one primary");
    }
}

// ---- Canonical fields ----

#[test]
fn test_canonical_fields_come_from_primary() {
    let uid = "uid@test.com";
    let e1 = make_event("Standup", 9, Some(uid), None, None);
    let mut e2 = make_event("standup", 9, Some(uid), None, None);
    e2.start_at = e1.start_at;
    e2.end_at = e1.end_at;
    let events = vec![e1.clone(), e2];
    let groups = group_events(&events);
    assert_eq!(groups.len(), 1);
    // Canonical title should come from the first/primary event
    assert_eq!(groups[0].canonical_title, e1.title);
    assert_eq!(groups[0].canonical_start, e1.start_at);
}

// ---- All events accounted for ----

#[test]
fn test_all_events_end_up_in_a_group() {
    let events: Vec<SourceEvent> = (0..10).map(|i| {
        make_event(&format!("Event {}", i), (9 + i) as u32, None, None, None)
    }).collect();
    let total = events.len();
    let groups = group_events(&events);
    let members_total: usize = groups.iter().map(|g| g.members.len()).sum();
    assert_eq!(members_total, total, "All events should be in exactly one group");
}
