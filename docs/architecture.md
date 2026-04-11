# Architecture Notes

## Overview

Family Center is a pragmatic monolith. One Rust server, one Ionic React app. No microservices.

```
┌─────────────────────────────────────────────────────────────┐
│  Android Touchscreen (wall display)                          │
│                                                             │
│  Ionic React + Capacitor App                                │
│  ┌────────────────────────────────────┐                     │
│  │  Week Board / Day Board (screens)  │                     │
│  │  Swim-lane layout                  │                     │
│  │  TanStack Query ↔ API client       │                     │
│  │  Zustand (auth/app state)          │                     │
│  │  SQLite (local schedule cache)     │                     │
│  └─────────────────┬──────────────────┘                     │
│                    │ HTTP REST                               │
└────────────────────│────────────────────────────────────────┘
                     │
┌────────────────────▼───────────────────┐
│  Rust Axum Server (Leapcell)           │
│                                        │
│  handlers/ — REST endpoint logic       │
│  sync/     — Google Calendar sync      │
│  dedupe/   — Event deduplication       │
│  google/   — OAuth + API client        │
│  models/   — SQLx data models          │
│                                        │
│  PostgreSQL                            │
└────────────────────────────────────────┘
```

## Local-First Model

**Strategy:** read SQLite first, sync in background, update SQLite on completion.

1. App starts → reads SQLite cache → renders immediately
2. Network check → if online, call `/sync/run` → server fetches new events
3. App polls `/schedule` → server returns merged events → stored to SQLite
4. If offline → SQLite cache serves last-known schedule (up to 24h)

**Tradeoffs:**
- Simple: no CRDT, no conflict resolution, server is authoritative
- Cache TTL = 24h: stale but readable for a wall display
- Local activities are stored on the server and cached locally; edits require network
- Gap: local activities created offline are not yet queued for sync. To fix: add an outbox table in SQLite.

## Google Calendar Integration

**Auth flow:**
1. `POST /google/connect/start` → returns OAuth2 authorization URL
2. User approves in browser → redirect to `/google/connect/callback?code=...`
3. Server exchanges code for tokens, stores in `google_accounts`
4. `GET /google/accounts/:id/calendars` → server fetches calendar list from Google API
5. User selects calendars via toggle → `POST /google/calendars/select`
6. `POST /sync/run` → server fetches events for all selected calendars

**Mock mode:** set `MOCK_CALENDAR=true` (default in dev). Skips real OAuth. Creates a mock account with three calendars and generates deterministic fake events.

**Incremental sync:** `sync_checkpoints` table stores `syncToken` per calendar. Currently not fully implemented — full sync on each run. Gap: implement `syncToken` incremental path in `sync/mod.rs`.

## Deduplication

Three tiers, applied in order:

| Tier | Signal | Action |
|------|--------|--------|
| Exact | Same `iCalUID` + same start timestamp | Collapse to one; mark `dupe_tier = 'exact'` |
| Strong | Same normalized title + start within ±5 min + same organizer | Collapse; `dupe_tier = 'strong'` |
| Probable | Same normalized title + same day/hour + overlapping attendees | Collapse; `dupe_tier = 'probable'` |

Source events are **never deleted**. `merged_event_groups` is rebuilt on each sync.

The `dedupe_mode` setting controls what the app renders:
- `show_all`: show all merged groups including duplicates
- `exact_only`: hide groups where `dupe_tier = 'exact'` and `is_primary = false`
- `strong`: hide exact + strong
- `probable`: hide all three tiers

## Lane Assignment

Rules are evaluated in priority order (lower number = higher priority):

1. Calendar-based: if `calendar_source_id` matches the event's source → assign to `person_id`
2. Email pattern: if organizer or attendee email matches glob pattern → assign
3. Fallback: assign to shared lane (`person_id = NULL`)

Lane assignment is independent of sync. It runs during `rebuild_merged_events`.

## Data Model Summary

```
households → people → lane_assignment_rules
           → settings
           → local_activities → local_activity_recurrences
           → merged_event_groups → merged_event_sources → source_events
           → google_accounts → calendar_sources → source_events
                             → sync_checkpoints
```

## Known Gaps

1. **Incremental sync**: full sync on every run. Fix: use `syncToken` from `sync_checkpoints`.
2. **Offline activity creation**: activities created offline are lost until network returns. Fix: SQLite outbox queue.
3. **Activity detail screen**: shows event ID only. Fix: wire up to fetch `/activities/:id` or `/schedule`.
4. **Token refresh**: access tokens expire. Fix: implement refresh token flow in `google/client.rs`.
5. **Lane rules UI**: no screen to manage lane assignment rules. Fix: add to Settings.
6. **Android build**: requires `npm run cap:sync` after `npm run build` and Android Studio for first install.
