//! Deduplication and event merging.
//!
//! Three tiers:
//!   Tier 1 exact:   same iCalUID + normalized start/end
//!   Tier 2 strong:  same normalized title + similar start/end (plus/minus 5 min) + same organizer
//!   Tier 3 probable: same normalized title + same day/hour bucket + overlapping attendees

use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration, Timelike};

use crate::domain::entities::lane::LaneAssignmentRule;
use crate::domain::entities::source_event::SourceEvent;

#[derive(Debug, Clone, PartialEq)]
pub enum DupeTier {
    Exact,
    Strong,
    Probable,
}

impl DupeTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            DupeTier::Exact => "exact",
            DupeTier::Strong => "strong",
            DupeTier::Probable => "probable",
        }
    }
}

#[derive(Debug)]
pub struct EventGroup {
    pub canonical_title: String,
    pub canonical_start: DateTime<Utc>,
    pub canonical_end: DateTime<Utc>,
    pub is_all_day: bool,
    pub dupe_tier: Option<DupeTier>,
    /// (source_event_id, is_primary)
    pub members: Vec<(Uuid, bool)>,
}

pub fn normalize_title(title: &str) -> String {
    title
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn group_events(events: &[SourceEvent]) -> Vec<EventGroup> {
    if events.is_empty() {
        return Vec::new();
    }

    let mut assigned = vec![false; events.len()];
    let mut groups: Vec<EventGroup> = Vec::new();

    // Tier 1: exact duplicates by iCalUID + start timestamp
    let mut exact_map: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, event) in events.iter().enumerate() {
        if let Some(uid) = &event.ical_uid {
            let key = format!("{}|{}", uid, event.start_at.timestamp());
            exact_map.entry(key).or_default().push(i);
        }
    }
    for (_, indices) in &exact_map {
        if indices.len() <= 1 { continue; }
        let primary = &events[indices[0]];
        let members: Vec<_> = indices.iter().enumerate().map(|(j, &i)| (events[i].id, j == 0)).collect();
        for &i in indices { assigned[i] = true; }
        groups.push(EventGroup {
            canonical_title: primary.title.clone(),
            canonical_start: primary.start_at,
            canonical_end: primary.end_at,
            is_all_day: primary.is_all_day,
            dupe_tier: Some(DupeTier::Exact),
            members,
        });
    }

    // Tier 2: strong duplicates
    let unassigned: Vec<usize> = (0..events.len()).filter(|&i| !assigned[i]).collect();
    let mut strong_assigned = vec![false; unassigned.len()];

    for (a, &i) in unassigned.iter().enumerate() {
        if strong_assigned[a] { continue; }
        let ei = &events[i];
        let mut group_pos = vec![a];

        for (b, &j) in unassigned.iter().enumerate().skip(a + 1) {
            if strong_assigned[b] { continue; }
            let ej = &events[j];
            let title_match = normalize_title(&ei.title) == normalize_title(&ej.title);
            let time_close = (ei.start_at - ej.start_at).abs() <= Duration::minutes(5);
            let org_ok = match (&ei.organizer, &ej.organizer) {
                (Some(a), Some(b)) => a == b,
                _ => true,
            };
            if title_match && time_close && org_ok {
                group_pos.push(b);
            }
        }

        if group_pos.len() > 1 {
            for &p in &group_pos { strong_assigned[p] = true; }
            let indices: Vec<usize> = group_pos.iter().map(|&p| unassigned[p]).collect();
            let primary = &events[indices[0]];
            let members: Vec<_> = indices.iter().enumerate().map(|(j, &i)| (events[i].id, j == 0)).collect();
            for &i in &indices { assigned[i] = true; }
            groups.push(EventGroup {
                canonical_title: primary.title.clone(),
                canonical_start: primary.start_at,
                canonical_end: primary.end_at,
                is_all_day: primary.is_all_day,
                dupe_tier: Some(DupeTier::Strong),
                members,
            });
        }
    }

    // Tier 3: probable duplicates
    let unassigned2: Vec<usize> = (0..events.len()).filter(|&i| !assigned[i]).collect();
    let mut prob_assigned = vec![false; unassigned2.len()];

    for (a, &i) in unassigned2.iter().enumerate() {
        if prob_assigned[a] { continue; }
        let ei = &events[i];
        let mut group_pos = vec![a];

        for (b, &j) in unassigned2.iter().enumerate().skip(a + 1) {
            if prob_assigned[b] { continue; }
            let ej = &events[j];
            let title_match = normalize_title(&ei.title) == normalize_title(&ej.title);
            let same_day = ei.start_at.date_naive() == ej.start_at.date_naive();
            let same_hour = ei.start_at.hour() == ej.start_at.hour();
            let attendee_overlap = match (&ei.attendees, &ej.attendees) {
                (Some(a), Some(b)) => a.iter().any(|e| b.contains(e)),
                _ => false,
            };
            if title_match && same_day && same_hour && attendee_overlap {
                group_pos.push(b);
            }
        }

        if group_pos.len() > 1 {
            for &p in &group_pos { prob_assigned[p] = true; }
            let indices: Vec<usize> = group_pos.iter().map(|&p| unassigned2[p]).collect();
            let primary = &events[indices[0]];
            let members: Vec<_> = indices.iter().enumerate().map(|(j, &i)| (events[i].id, j == 0)).collect();
            for &i in &indices { assigned[i] = true; }
            groups.push(EventGroup {
                canonical_title: primary.title.clone(),
                canonical_start: primary.start_at,
                canonical_end: primary.end_at,
                is_all_day: primary.is_all_day,
                dupe_tier: Some(DupeTier::Probable),
                members,
            });
        }
    }

    // Singletons
    for (i, event) in events.iter().enumerate() {
        if !assigned[i] {
            groups.push(EventGroup {
                canonical_title: event.title.clone(),
                canonical_start: event.start_at,
                canonical_end: event.end_at,
                is_all_day: event.is_all_day,
                dupe_tier: None,
                members: vec![(event.id, true)],
            });
        }
    }

    groups
}

pub fn glob_match(text: &str, pattern: &str) -> bool {
    if pattern == "*" { return true; }
    let text = text.to_lowercase();
    let pattern = pattern.to_lowercase();
    if !pattern.contains('*') {
        return text == pattern;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    let mut pos = 0;
    for (idx, part) in parts.iter().enumerate() {
        if part.is_empty() { continue; }
        if idx == 0 {
            if !text.starts_with(*part) { return false; }
            pos = part.len();
        } else if let Some(found) = text[pos..].find(part) {
            pos += found + part.len();
        } else {
            return false;
        }
    }
    true
}

pub fn apply_lane_rules(event: &SourceEvent, rules: &[LaneAssignmentRule]) -> Option<Uuid> {
    for rule in rules {
        let cal_match = rule.calendar_source_id
            .map(|id| id == event.calendar_source_id)
            .unwrap_or(true);

        let email_match = rule.email_pattern.as_deref()
            .map(|pat| {
                let org_ok = event.organizer.as_deref().map(|o| glob_match(o, pat)).unwrap_or(false);
                let att_ok = event.attendees.as_deref().unwrap_or(&[]).iter().any(|a| glob_match(a, pat));
                org_ok || att_ok
            })
            .unwrap_or(true);

        if cal_match && email_match {
            return rule.person_id;
        }
    }
    None
}

#[cfg(test)]
mod tests;
