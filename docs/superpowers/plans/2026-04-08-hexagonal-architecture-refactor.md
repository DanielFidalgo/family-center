# Hexagonal Architecture Refactor — Rust Server

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor the Rust server to match the Montra hexagonal architecture: domain owns repository traits and service traits, infrastructure implements data access, `AppContext` is the DI composition root, and startup uses a bootstrapper pattern.

**Architecture:** The domain layer becomes the stability center — it defines `I*Repository` traits, `I*Service` traits, entity types, and errors. Infrastructure implements repository traits via SQLx. Application route handlers receive `Arc<dyn IAppContext>` and delegate to services. A bootstrapper in configuration wires everything at startup (config → DB → repos → services → context → routes → serve).

**Tech Stack:** Rust, Poem 3.1.12, poem-openapi, SQLx (PostgreSQL), async-trait, tokio, anyhow/thiserror

---

## File Structure

### Domain Layer (`src/domain/`)
New or modified files — **no SQLx, no Poem imports allowed here**:

| File | Responsibility |
|------|---------------|
| `domain/mod.rs` | Re-exports all domain submodules |
| `domain/entities/mod.rs` | Re-exports entity modules |
| `domain/entities/household.rs` | `Household` entity struct (plain Rust, serde + poem_openapi derives) |
| `domain/entities/person.rs` | `Person`, `CreatePerson`, `UpdatePerson` |
| `domain/entities/google_account.rs` | `GoogleAccount` |
| `domain/entities/calendar_source.rs` | `CalendarSource`, `CalendarSelection`, `SelectCalendarsBody` |
| `domain/entities/source_event.rs` | `SourceEvent` |
| `domain/entities/merged_event.rs` | `MergedEventGroup`, `MergedEventSource`, `MergedEventGroupWithSources` |
| `domain/entities/local_activity.rs` | `LocalActivity`, `LocalActivityRecurrence`, `LocalActivityWithRecurrence`, DTOs |
| `domain/entities/settings.rs` | `Settings`, `UpdateSettings` |
| `domain/entities/lane.rs` | `LaneAssignmentRule` |
| `domain/entities/sync.rs` | `SyncCheckpoint`, `SyncRunRequest`, `SyncRunResponse`, `SyncError` |
| `domain/repositories/mod.rs` | Re-exports repository trait modules |
| `domain/repositories/household_repository.rs` | `IHouseholdRepository` trait |
| `domain/repositories/person_repository.rs` | `IPersonRepository` trait |
| `domain/repositories/google_account_repository.rs` | `IGoogleAccountRepository` trait |
| `domain/repositories/calendar_source_repository.rs` | `ICalendarSourceRepository` trait |
| `domain/repositories/source_event_repository.rs` | `ISourceEventRepository` trait |
| `domain/repositories/merged_event_repository.rs` | `IMergedEventRepository` trait |
| `domain/repositories/local_activity_repository.rs` | `ILocalActivityRepository` trait |
| `domain/repositories/settings_repository.rs` | `ISettingsRepository` trait |
| `domain/repositories/lane_rule_repository.rs` | `ILaneRuleRepository` trait |
| `domain/error.rs` | `DomainError` with `From<RepositoryError>` |
| `domain/dedupe/mod.rs` | Pure dedup logic (already mostly pure — remove `sqlx` usage) |
| `domain/recurrence/mod.rs` | Pure recurrence logic (already mostly pure — fix entity import) |
| `domain/sync/mod.rs` | `ISyncService` trait |

### Infrastructure Layer (`src/infrastructure/`)
Implements repository traits via SQLx:

| File | Responsibility |
|------|---------------|
| `infrastructure/mod.rs` | Re-exports |
| `infrastructure/db.rs` | `connect()` — unchanged |
| `infrastructure/error.rs` | `RepositoryError` — unchanged |
| `infrastructure/auth.rs` | JWT create/verify — unchanged |
| `infrastructure/repositories/mod.rs` | Re-exports repository implementations |
| `infrastructure/repositories/household_repository.rs` | `HouseholdRepository` implements `IHouseholdRepository` |
| `infrastructure/repositories/person_repository.rs` | `PersonRepository` implements `IPersonRepository` |
| `infrastructure/repositories/google_account_repository.rs` | `GoogleAccountRepository` implements `IGoogleAccountRepository` |
| `infrastructure/repositories/calendar_source_repository.rs` | `CalendarSourceRepository` implements `ICalendarSourceRepository` |
| `infrastructure/repositories/source_event_repository.rs` | `SourceEventRepository` implements `ISourceEventRepository` |
| `infrastructure/repositories/merged_event_repository.rs` | `MergedEventRepository` implements `IMergedEventRepository` |
| `infrastructure/repositories/local_activity_repository.rs` | `LocalActivityRepository` implements `ILocalActivityRepository` |
| `infrastructure/repositories/settings_repository.rs` | `SettingsRepository` implements `ISettingsRepository` |
| `infrastructure/repositories/lane_rule_repository.rs` | `LaneRuleRepository` implements `ILaneRuleRepository` |
| `infrastructure/google/client.rs` | OAuth client — remove direct SQLx calls, use repository |
| `infrastructure/google/mock.rs` | Mock events — unchanged |
| `infrastructure/services/mod.rs` | Re-exports service implementations |
| `infrastructure/services/sync_service.rs` | `SyncService` implements `ISyncService` |

### Configuration Layer (`src/configuration/`)

| File | Responsibility |
|------|---------------|
| `configuration/mod.rs` | Re-exports |
| `configuration/config.rs` | `AppConfig` with `IAppConfig` trait |
| `configuration/app_context.rs` | `AppContext` implementing `IAppContext` — the DI composition root |
| `configuration/bootstrapper.rs` | `bootstrap()` — wires config → DB → repos → services → context → routes → serve |

### Application Layer (`src/application/routes/`)

| File | Responsibility |
|------|---------------|
| `routes/mod.rs` | `build_routes()` takes `Arc<dyn IAppContext>` |
| `routes/security.rs` | `ApiError` with `From<DomainError>` |
| `routes/auth.rs` | `AuthApi` holds `Arc<dyn IAppContext>` |
| `routes/google.rs` | `GoogleApi` holds `Arc<dyn IAppContext>` |
| `routes/sync.rs` | `SyncApi` holds `Arc<dyn IAppContext>` |
| `routes/schedule.rs` | `ScheduleApi` holds `Arc<dyn IAppContext>` |
| `routes/people.rs` | `PeopleApi` holds `Arc<dyn IAppContext>` |
| `routes/activities.rs` | `ActivitiesApi` holds `Arc<dyn IAppContext>` |
| `routes/settings.rs` | `SettingsApi` holds `Arc<dyn IAppContext>` |

---

## Task 1: Move Entity Structs from Infrastructure to Domain

Move all model structs from `infrastructure/models/` to `domain/entities/`. Remove `sqlx::FromRow` derives (those will live on infra-specific row types or be re-added as the infra layer needs). Keep `serde` and `poem_openapi::Object` derives since these are framework-agnostic serialization.

**Files:**
- Create: `src/domain/entities/mod.rs`
- Create: `src/domain/entities/household.rs`
- Create: `src/domain/entities/person.rs`
- Create: `src/domain/entities/google_account.rs`
- Create: `src/domain/entities/calendar_source.rs`
- Create: `src/domain/entities/source_event.rs`
- Create: `src/domain/entities/merged_event.rs`
- Create: `src/domain/entities/local_activity.rs`
- Create: `src/domain/entities/settings.rs`
- Create: `src/domain/entities/lane.rs`
- Create: `src/domain/entities/sync.rs`
- Delete: `src/infrastructure/models/` (entire directory, after migration)
- Modify: `src/domain/mod.rs`

- [ ] **Step 1: Create `domain/entities/mod.rs`**

```rust
pub mod calendar_source;
pub mod google_account;
pub mod household;
pub mod lane;
pub mod local_activity;
pub mod merged_event;
pub mod person;
pub mod settings;
pub mod source_event;
pub mod sync;
```

- [ ] **Step 2: Create each entity file**

Move structs from their `infrastructure/models/` counterpart. Keep `serde`, `poem_openapi::Object`, and `Clone`/`Debug` derives. **Remove** `sqlx::FromRow`. Example for `domain/entities/household.rs`:

```rust
use chrono::{DateTime, Utc};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct Household {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

For `domain/entities/person.rs`:

```rust
use chrono::{DateTime, Utc};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    pub id: Uuid,
    pub household_id: Uuid,
    pub name: String,
    pub color: String,
    pub avatar_url: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct CreatePerson {
    pub name: String,
    pub color: String,
    pub avatar_url: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePerson {
    pub name: Option<String>,
    pub color: Option<String>,
    pub avatar_url: Option<Option<String>>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}
```

For `domain/entities/google_account.rs`:

```rust
use chrono::{DateTime, Utc};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct GoogleAccount {
    pub id: Uuid,
    pub household_id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    #[serde(skip_serializing)]
    pub access_token: Option<String>,
    #[serde(skip_serializing)]
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

For `domain/entities/calendar_source.rs`:

```rust
use chrono::{DateTime, Utc};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct CalendarSource {
    pub id: Uuid,
    pub google_account_id: Uuid,
    pub calendar_id: String,
    pub name: String,
    pub description: Option<String>,
    pub color_hex: Option<String>,
    pub is_selected: bool,
    pub access_role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct CalendarSelection {
    pub calendar_source_id: Uuid,
    pub is_selected: bool,
}

#[derive(Debug, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct SelectCalendarsBody {
    pub selections: Vec<CalendarSelection>,
}
```

For `domain/entities/source_event.rs`:

```rust
use chrono::{DateTime, Utc};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct SourceEvent {
    pub id: Uuid,
    pub calendar_source_id: Uuid,
    pub google_event_id: String,
    pub ical_uid: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub is_all_day: bool,
    pub recurrence_rule: Option<String>,
    pub recurring_event_id: Option<String>,
    pub organizer: Option<String>,
    pub attendees: Option<Vec<String>>,
    #[oai(skip)]
    pub raw_json: Value,
    pub synced_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

For `domain/entities/merged_event.rs` — copy entire file from `infrastructure/models/merged_event.rs`, remove `sqlx::FromRow`.

For `domain/entities/local_activity.rs` — copy entire file from `infrastructure/models/local_activity.rs`, remove `sqlx::FromRow`.

For `domain/entities/settings.rs` — copy from `infrastructure/models/settings.rs`, remove `sqlx::FromRow`.

For `domain/entities/lane.rs` — copy from `infrastructure/models/lane.rs`, remove `sqlx::FromRow`.

For `domain/entities/sync.rs` — copy from `infrastructure/models/sync.rs` (the `SyncCheckpoint`, `SyncRunRequest`, `SyncRunResponse`, `SyncError` structs), remove `sqlx::FromRow`.

- [ ] **Step 3: Update `domain/mod.rs`**

```rust
pub mod dedupe;
pub mod entities;
pub mod error;
pub mod recurrence;
pub mod sync;
```

- [ ] **Step 4: Delete `src/infrastructure/models/` directory**

Remove the entire `infrastructure/models/` directory and remove `pub mod models;` from `infrastructure/mod.rs`.

- [ ] **Step 5: Update all imports across the codebase**

Every file that previously imported from `crate::infrastructure::models::*` must now import from `crate::domain::entities::*`. This includes:
- All files in `application/routes/`
- `domain/dedupe/mod.rs` (change `use crate::infrastructure::models::source_event::SourceEvent` → `use crate::domain::entities::source_event::SourceEvent`)
- `domain/recurrence/mod.rs` (change `use crate::infrastructure::models::local_activity::LocalActivityRecurrence` → `use crate::domain::entities::local_activity::LocalActivityRecurrence`)
- `domain/sync/mod.rs`
- `infrastructure/google/client.rs`
- `infrastructure/google/mock.rs`

- [ ] **Step 6: Add `sqlx::FromRow` derives back in infrastructure via re-derive**

The infrastructure layer needs `sqlx::FromRow` on the entities it queries. The cleanest approach: add `sqlx::FromRow` back to the domain entity structs that are directly queried. This is a pragmatic choice — `sqlx::FromRow` is a derive macro, not a runtime dependency on sqlx internals. The alternative (separate row types with `From` conversions) adds boilerplate for no real benefit in this codebase.

Add `sqlx::FromRow` back to: `Household`, `Person`, `GoogleAccount`, `CalendarSource`, `SourceEvent`, `MergedEventGroup`, `MergedEventSource`, `LocalActivity`, `LocalActivityRecurrence`, `Settings`, `LaneAssignmentRule`, `SyncCheckpoint`.

This means the domain `Cargo.toml` would need `sqlx` as a dependency for the derive only. Since this is a single-crate project (not a workspace with separate domain crate), this is acceptable.

- [ ] **Step 7: Verify it compiles**

Run: `cd /Users/fidalgo/personal/repos/family-center/apps/server && cargo check 2>&1`
Expected: No errors. All imports resolve.

- [ ] **Step 8: Run existing tests**

Run: `cd /Users/fidalgo/personal/repos/family-center/apps/server && cargo test 2>&1`
Expected: All existing dedupe and recurrence tests pass.

- [ ] **Step 9: Commit**

```bash
cd /Users/fidalgo/personal/repos/family-center
git add apps/server/src/domain/entities/ apps/server/src/domain/mod.rs
git add -u apps/server/src/
git commit -m "refactor: move entity structs from infrastructure/models to domain/entities"
```

---

## Task 2: Define Repository Traits in Domain

Create async trait interfaces in `domain/repositories/` for each entity's data access needs. These traits define the contract — infrastructure will implement them.

**Files:**
- Create: `src/domain/repositories/mod.rs`
- Create: `src/domain/repositories/household_repository.rs`
- Create: `src/domain/repositories/person_repository.rs`
- Create: `src/domain/repositories/google_account_repository.rs`
- Create: `src/domain/repositories/calendar_source_repository.rs`
- Create: `src/domain/repositories/source_event_repository.rs`
- Create: `src/domain/repositories/merged_event_repository.rs`
- Create: `src/domain/repositories/local_activity_repository.rs`
- Create: `src/domain/repositories/settings_repository.rs`
- Create: `src/domain/repositories/lane_rule_repository.rs`
- Modify: `src/domain/mod.rs`

- [ ] **Step 1: Create `domain/repositories/mod.rs`**

```rust
pub mod calendar_source_repository;
pub mod google_account_repository;
pub mod household_repository;
pub mod lane_rule_repository;
pub mod local_activity_repository;
pub mod merged_event_repository;
pub mod person_repository;
pub mod settings_repository;
pub mod source_event_repository;
```

- [ ] **Step 2: Create `domain/repositories/household_repository.rs`**

```rust
use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::household::Household;

#[async_trait]
pub trait IHouseholdRepository: Send + Sync {
    async fn find_first(&self) -> anyhow::Result<Option<Household>>;
    async fn create(&self, id: Uuid, name: &str) -> anyhow::Result<Household>;
}
```

- [ ] **Step 3: Create `domain/repositories/person_repository.rs`**

```rust
use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::person::{CreatePerson, Person, UpdatePerson};

#[async_trait]
pub trait IPersonRepository: Send + Sync {
    async fn find_by_household(&self, household_id: Uuid) -> anyhow::Result<Vec<Person>>;
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Person>>;
    async fn create(&self, household_id: Uuid, input: &CreatePerson) -> anyhow::Result<Person>;
    async fn update(&self, id: Uuid, existing: &Person, input: &UpdatePerson) -> anyhow::Result<Person>;
}
```

- [ ] **Step 4: Create `domain/repositories/google_account_repository.rs`**

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entities::google_account::GoogleAccount;

#[async_trait]
pub trait IGoogleAccountRepository: Send + Sync {
    async fn find_by_household(&self, household_id: Uuid) -> anyhow::Result<Vec<GoogleAccount>>;
    async fn upsert_mock(&self, household_id: Uuid) -> anyhow::Result<GoogleAccount>;
    async fn upsert(
        &self,
        household_id: Uuid,
        email: &str,
        display_name: Option<&str>,
        avatar_url: Option<&str>,
        access_token: Option<&str>,
        refresh_token: Option<&str>,
        token_expires_at: Option<DateTime<Utc>>,
    ) -> anyhow::Result<GoogleAccount>;
}
```

- [ ] **Step 5: Create `domain/repositories/calendar_source_repository.rs`**

```rust
use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::calendar_source::{CalendarSelection, CalendarSource};

#[async_trait]
pub trait ICalendarSourceRepository: Send + Sync {
    async fn find_by_account(&self, google_account_id: Uuid) -> anyhow::Result<Vec<CalendarSource>>;
    async fn create_mock_calendars(&self, google_account_id: Uuid) -> anyhow::Result<()>;
    async fn update_selection(&self, selection: &CalendarSelection) -> anyhow::Result<Option<CalendarSource>>;
    async fn find_selected_with_tokens(&self) -> anyhow::Result<Vec<CalendarSourceWithToken>>;
    async fn find_selected_with_tokens_by_ids(&self, ids: &[Uuid]) -> anyhow::Result<Vec<CalendarSourceWithToken>>;
}

/// Joined data needed for sync — calendar source + Google account tokens.
pub struct CalendarSourceWithToken {
    pub id: Uuid,
    pub google_account_id: Uuid,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}
```

- [ ] **Step 6: Create `domain/repositories/source_event_repository.rs`**

```rust
use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::source_event::SourceEvent;

#[async_trait]
pub trait ISourceEventRepository: Send + Sync {
    async fn upsert(&self, event: &SourceEvent) -> anyhow::Result<bool>;
    async fn find_by_household_selected(&self, household_id: Uuid) -> anyhow::Result<Vec<SourceEvent>>;
}
```

- [ ] **Step 7: Create `domain/repositories/merged_event_repository.rs`**

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entities::merged_event::{MergedEventGroup, MergedEventGroupWithSources, MergedEventSource};
use crate::domain::dedupe::EventGroup;

#[async_trait]
pub trait IMergedEventRepository: Send + Sync {
    async fn find_by_household_in_range(
        &self,
        household_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<Vec<MergedEventGroupWithSources>>;
    async fn delete_by_household(&self, household_id: Uuid) -> anyhow::Result<()>;
    async fn insert_group(
        &self,
        household_id: Uuid,
        group: &EventGroup,
        person_id: Option<Uuid>,
    ) -> anyhow::Result<()>;
}
```

- [ ] **Step 8: Create `domain/repositories/local_activity_repository.rs`**

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entities::local_activity::{
    CreateLocalActivity, LocalActivity, LocalActivityRecurrence,
    LocalActivityWithRecurrence, RecurrenceInput, UpdateLocalActivity,
};

#[async_trait]
pub trait ILocalActivityRepository: Send + Sync {
    async fn find_by_household(&self, household_id: Uuid) -> anyhow::Result<Vec<LocalActivityWithRecurrence>>;
    async fn find_by_household_in_range(
        &self,
        household_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<Vec<LocalActivityWithRecurrence>>;
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<LocalActivity>>;
    async fn create(
        &self,
        household_id: Uuid,
        input: &CreateLocalActivity,
    ) -> anyhow::Result<LocalActivityWithRecurrence>;
    async fn update(
        &self,
        id: Uuid,
        input: &UpdateLocalActivity,
    ) -> anyhow::Result<LocalActivityWithRecurrence>;
}
```

- [ ] **Step 9: Create `domain/repositories/settings_repository.rs`**

```rust
use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::settings::{Settings, UpdateSettings};

#[async_trait]
pub trait ISettingsRepository: Send + Sync {
    async fn find_by_household(&self, household_id: Uuid) -> anyhow::Result<Option<Settings>>;
    async fn create_default(&self, household_id: Uuid) -> anyhow::Result<Settings>;
    async fn update(&self, household_id: Uuid, input: &UpdateSettings) -> anyhow::Result<Settings>;
}
```

- [ ] **Step 10: Create `domain/repositories/lane_rule_repository.rs`**

```rust
use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::lane::LaneAssignmentRule;

#[async_trait]
pub trait ILaneRuleRepository: Send + Sync {
    async fn find_by_household(&self, household_id: Uuid) -> anyhow::Result<Vec<LaneAssignmentRule>>;
}
```

- [ ] **Step 11: Update `domain/mod.rs`**

```rust
pub mod dedupe;
pub mod entities;
pub mod error;
pub mod recurrence;
pub mod repositories;
pub mod sync;
```

- [ ] **Step 12: Verify it compiles**

Run: `cd /Users/fidalgo/personal/repos/family-center/apps/server && cargo check 2>&1`
Expected: No errors. Traits are defined but not yet implemented.

- [ ] **Step 13: Commit**

```bash
cd /Users/fidalgo/personal/repos/family-center
git add apps/server/src/domain/repositories/
git add apps/server/src/domain/mod.rs
git commit -m "feat: define repository traits in domain layer"
```

---

## Task 3: Implement Repository Traits in Infrastructure

Create concrete SQLx-backed implementations of each repository trait. Move all `sqlx::query!` / `sqlx::query_as!` calls from route handlers and domain functions into these repositories.

**Files:**
- Create: `src/infrastructure/repositories/mod.rs`
- Create: `src/infrastructure/repositories/household_repository.rs`
- Create: `src/infrastructure/repositories/person_repository.rs`
- Create: `src/infrastructure/repositories/google_account_repository.rs`
- Create: `src/infrastructure/repositories/calendar_source_repository.rs`
- Create: `src/infrastructure/repositories/source_event_repository.rs`
- Create: `src/infrastructure/repositories/merged_event_repository.rs`
- Create: `src/infrastructure/repositories/local_activity_repository.rs`
- Create: `src/infrastructure/repositories/settings_repository.rs`
- Create: `src/infrastructure/repositories/lane_rule_repository.rs`
- Modify: `src/infrastructure/mod.rs`

- [ ] **Step 1: Create `infrastructure/repositories/mod.rs`**

```rust
pub mod calendar_source_repository;
pub mod google_account_repository;
pub mod household_repository;
pub mod lane_rule_repository;
pub mod local_activity_repository;
pub mod merged_event_repository;
pub mod person_repository;
pub mod settings_repository;
pub mod source_event_repository;
```

- [ ] **Step 2: Create `infrastructure/repositories/household_repository.rs`**

```rust
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::household::Household;
use crate::domain::repositories::household_repository::IHouseholdRepository;

pub struct HouseholdRepository {
    pool: PgPool,
}

impl HouseholdRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IHouseholdRepository for HouseholdRepository {
    async fn find_first(&self) -> anyhow::Result<Option<Household>> {
        let row = sqlx::query_as!(
            Household,
            "SELECT id, name, created_at, updated_at FROM households LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    async fn create(&self, id: Uuid, name: &str) -> anyhow::Result<Household> {
        let household = sqlx::query_as!(
            Household,
            "INSERT INTO households (id, name) VALUES ($1, $2) RETURNING id, name, created_at, updated_at",
            id,
            name
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(household)
    }
}
```

- [ ] **Step 3: Create `infrastructure/repositories/person_repository.rs`**

```rust
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::person::{CreatePerson, Person, UpdatePerson};
use crate::domain::repositories::person_repository::IPersonRepository;

pub struct PersonRepository {
    pool: PgPool,
}

impl PersonRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IPersonRepository for PersonRepository {
    async fn find_by_household(&self, household_id: Uuid) -> anyhow::Result<Vec<Person>> {
        let people = sqlx::query_as!(
            Person,
            r#"SELECT id, household_id, name, color, avatar_url, sort_order, is_active, created_at, updated_at
               FROM people WHERE household_id = $1 AND is_active = TRUE ORDER BY sort_order, name"#,
            household_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(people)
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Person>> {
        let person = sqlx::query_as!(
            Person,
            "SELECT id, household_id, name, color, avatar_url, sort_order, is_active, created_at, updated_at FROM people WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(person)
    }

    async fn create(&self, household_id: Uuid, input: &CreatePerson) -> anyhow::Result<Person> {
        let person = sqlx::query_as!(
            Person,
            r#"INSERT INTO people (id, household_id, name, color, avatar_url, sort_order)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id, household_id, name, color, avatar_url, sort_order, is_active, created_at, updated_at"#,
            Uuid::new_v4(),
            household_id,
            input.name,
            input.color,
            input.avatar_url,
            input.sort_order.unwrap_or(0)
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(person)
    }

    async fn update(&self, id: Uuid, existing: &Person, input: &UpdatePerson) -> anyhow::Result<Person> {
        let person = sqlx::query_as!(
            Person,
            r#"UPDATE people SET
                name = $2, color = $3, avatar_url = $4, sort_order = $5, is_active = $6, updated_at = NOW()
               WHERE id = $1
               RETURNING id, household_id, name, color, avatar_url, sort_order, is_active, created_at, updated_at"#,
            id,
            input.name.as_deref().unwrap_or(&existing.name),
            input.color.as_deref().unwrap_or(&existing.color),
            input.avatar_url.clone().unwrap_or(existing.avatar_url.clone()),
            input.sort_order.unwrap_or(existing.sort_order),
            input.is_active.unwrap_or(existing.is_active),
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(person)
    }
}
```

- [ ] **Step 4: Create remaining repository implementations**

Follow the same pattern for each repository. Each struct holds `pool: PgPool`, constructed via `new(pool: PgPool)`. Move the corresponding `sqlx::query!` / `sqlx::query_as!` calls from route handlers into these methods.

For `google_account_repository.rs` — move queries from `routes/google.rs` (mock account upsert, list accounts) and `infrastructure/google/client.rs` (the upsert after OAuth exchange).

For `calendar_source_repository.rs` — move queries from `routes/google.rs` (list calendars, mock calendar creation, select calendars) and the sync join query from `domain/sync/mod.rs`.

For `source_event_repository.rs` — move the upsert query from `domain/sync/mod.rs` and the household-scoped query from `domain/dedupe/mod.rs`.

For `merged_event_repository.rs` — move queries from `routes/schedule.rs` (find groups + sources in range) and `domain/dedupe/mod.rs` (delete + insert groups/sources).

For `local_activity_repository.rs` — move queries from `routes/activities.rs` (CRUD) and `routes/schedule.rs` (find in range with recurrence).

For `settings_repository.rs` — move queries from `routes/settings.rs`.

For `lane_rule_repository.rs` — move the query from `domain/dedupe/mod.rs`.

- [ ] **Step 5: Update `infrastructure/mod.rs`**

```rust
pub mod auth;
pub mod db;
pub mod error;
pub mod google;
pub mod repositories;
```

- [ ] **Step 6: Verify it compiles**

Run: `cd /Users/fidalgo/personal/repos/family-center/apps/server && cargo check 2>&1`
Expected: No errors. Repository implementations compile against the trait definitions.

- [ ] **Step 7: Commit**

```bash
cd /Users/fidalgo/personal/repos/family-center
git add apps/server/src/infrastructure/repositories/
git add apps/server/src/infrastructure/mod.rs
git commit -m "feat: implement repository traits in infrastructure layer with SQLx"
```

---

## Task 4: Create IAppConfig, IAppContext, and Bootstrapper

Define the DI container traits and the bootstrapper that wires everything at startup.

**Files:**
- Modify: `src/configuration/config.rs`
- Create: `src/configuration/app_context.rs`
- Create: `src/configuration/bootstrapper.rs`
- Modify: `src/configuration/mod.rs`

- [ ] **Step 1: Add `IAppConfig` trait to `configuration/config.rs`**

Add this trait above the `Config` struct. The trait exposes config values without exposing the concrete struct:

```rust
pub trait IAppConfig: Send + Sync {
    fn database_url(&self) -> &str;
    fn server_host(&self) -> &str;
    fn server_port(&self) -> u16;
    fn jwt_secret(&self) -> &str;
    fn google_client_id(&self) -> &str;
    fn google_client_secret(&self) -> &str;
    fn google_redirect_uri(&self) -> &str;
    fn mock_calendar(&self) -> bool;
}

impl IAppConfig for Config {
    fn database_url(&self) -> &str { &self.database_url }
    fn server_host(&self) -> &str { &self.server_host }
    fn server_port(&self) -> u16 { self.server_port }
    fn jwt_secret(&self) -> &str { &self.jwt_secret }
    fn google_client_id(&self) -> &str { &self.google_client_id }
    fn google_client_secret(&self) -> &str { &self.google_client_secret }
    fn google_redirect_uri(&self) -> &str { &self.google_redirect_uri }
    fn mock_calendar(&self) -> bool { self.mock_calendar }
}
```

- [ ] **Step 2: Create `configuration/app_context.rs`**

This is the composition root — the single place where concrete types are wired together:

```rust
use std::sync::Arc;

use crate::domain::repositories::{
    calendar_source_repository::ICalendarSourceRepository,
    google_account_repository::IGoogleAccountRepository,
    household_repository::IHouseholdRepository,
    lane_rule_repository::ILaneRuleRepository,
    local_activity_repository::ILocalActivityRepository,
    merged_event_repository::IMergedEventRepository,
    person_repository::IPersonRepository,
    settings_repository::ISettingsRepository,
    source_event_repository::ISourceEventRepository,
};
use super::config::IAppConfig;

pub trait IAppContext: Send + Sync {
    fn config(&self) -> &dyn IAppConfig;
    fn household_repository(&self) -> &dyn IHouseholdRepository;
    fn person_repository(&self) -> &dyn IPersonRepository;
    fn google_account_repository(&self) -> &dyn IGoogleAccountRepository;
    fn calendar_source_repository(&self) -> &dyn ICalendarSourceRepository;
    fn source_event_repository(&self) -> &dyn ISourceEventRepository;
    fn merged_event_repository(&self) -> &dyn IMergedEventRepository;
    fn local_activity_repository(&self) -> &dyn ILocalActivityRepository;
    fn settings_repository(&self) -> &dyn ISettingsRepository;
    fn lane_rule_repository(&self) -> &dyn ILaneRuleRepository;
}

pub struct AppContext {
    config: Arc<dyn IAppConfig>,
    household_repository: Arc<dyn IHouseholdRepository>,
    person_repository: Arc<dyn IPersonRepository>,
    google_account_repository: Arc<dyn IGoogleAccountRepository>,
    calendar_source_repository: Arc<dyn ICalendarSourceRepository>,
    source_event_repository: Arc<dyn ISourceEventRepository>,
    merged_event_repository: Arc<dyn IMergedEventRepository>,
    local_activity_repository: Arc<dyn ILocalActivityRepository>,
    settings_repository: Arc<dyn ISettingsRepository>,
    lane_rule_repository: Arc<dyn ILaneRuleRepository>,
}

impl AppContext {
    pub fn new(
        config: Arc<dyn IAppConfig>,
        household_repository: Arc<dyn IHouseholdRepository>,
        person_repository: Arc<dyn IPersonRepository>,
        google_account_repository: Arc<dyn IGoogleAccountRepository>,
        calendar_source_repository: Arc<dyn ICalendarSourceRepository>,
        source_event_repository: Arc<dyn ISourceEventRepository>,
        merged_event_repository: Arc<dyn IMergedEventRepository>,
        local_activity_repository: Arc<dyn ILocalActivityRepository>,
        settings_repository: Arc<dyn ISettingsRepository>,
        lane_rule_repository: Arc<dyn ILaneRuleRepository>,
    ) -> Self {
        Self {
            config,
            household_repository,
            person_repository,
            google_account_repository,
            calendar_source_repository,
            source_event_repository,
            merged_event_repository,
            local_activity_repository,
            settings_repository,
            lane_rule_repository,
        }
    }
}

impl IAppContext for AppContext {
    fn config(&self) -> &dyn IAppConfig { self.config.as_ref() }
    fn household_repository(&self) -> &dyn IHouseholdRepository { self.household_repository.as_ref() }
    fn person_repository(&self) -> &dyn IPersonRepository { self.person_repository.as_ref() }
    fn google_account_repository(&self) -> &dyn IGoogleAccountRepository { self.google_account_repository.as_ref() }
    fn calendar_source_repository(&self) -> &dyn ICalendarSourceRepository { self.calendar_source_repository.as_ref() }
    fn source_event_repository(&self) -> &dyn ISourceEventRepository { self.source_event_repository.as_ref() }
    fn merged_event_repository(&self) -> &dyn IMergedEventRepository { self.merged_event_repository.as_ref() }
    fn local_activity_repository(&self) -> &dyn ILocalActivityRepository { self.local_activity_repository.as_ref() }
    fn settings_repository(&self) -> &dyn ISettingsRepository { self.settings_repository.as_ref() }
    fn lane_rule_repository(&self) -> &dyn ILaneRuleRepository { self.lane_rule_repository.as_ref() }
}
```

- [ ] **Step 3: Create `configuration/bootstrapper.rs`**

```rust
use std::sync::Arc;

use crate::configuration::{
    app_context::AppContext,
    config::Config,
    service_setup,
};
use crate::infrastructure::{
    db,
    repositories::{
        calendar_source_repository::CalendarSourceRepository,
        google_account_repository::GoogleAccountRepository,
        household_repository::HouseholdRepository,
        lane_rule_repository::LaneRuleRepository,
        local_activity_repository::LocalActivityRepository,
        merged_event_repository::MergedEventRepository,
        person_repository::PersonRepository,
        settings_repository::SettingsRepository,
        source_event_repository::SourceEventRepository,
    },
};
use crate::application::routes;

pub async fn bootstrap() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "family_center_server=debug,poem=debug".into()),
        )
        .init();

    // 1. Load config
    let config = Arc::new(Config::from_env()?);

    // 2. Connect to database
    let pool = db::connect(&config.database_url).await?;

    // 3. Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    // 4. Wire repositories
    let household_repo = Arc::new(HouseholdRepository::new(pool.clone()));
    let person_repo = Arc::new(PersonRepository::new(pool.clone()));
    let google_account_repo = Arc::new(GoogleAccountRepository::new(pool.clone()));
    let calendar_source_repo = Arc::new(CalendarSourceRepository::new(pool.clone()));
    let source_event_repo = Arc::new(SourceEventRepository::new(pool.clone()));
    let merged_event_repo = Arc::new(MergedEventRepository::new(pool.clone()));
    let local_activity_repo = Arc::new(LocalActivityRepository::new(pool.clone()));
    let settings_repo = Arc::new(SettingsRepository::new(pool.clone()));
    let lane_rule_repo = Arc::new(LaneRuleRepository::new(pool.clone()));

    // 5. Build app context
    let context: Arc<AppContext> = Arc::new(AppContext::new(
        config.clone(),
        household_repo,
        person_repo,
        google_account_repo,
        calendar_source_repo,
        source_event_repo,
        merged_event_repo,
        local_activity_repo,
        settings_repo,
        lane_rule_repo,
    ));

    // 6. Build routes
    let app_routes = routes::build_routes(context);

    // 7. Start server
    service_setup::serve(config.server_host(), config.server_port(), app_routes)
        .await
        .map_err(anyhow::Error::from)
}
```

- [ ] **Step 4: Update `configuration/mod.rs`**

```rust
pub mod app_context;
pub mod bootstrapper;
pub mod config;
pub mod service_setup;
```

- [ ] **Step 5: Update `db::connect` to return `PgPool` (not `Arc`)**

Modify `infrastructure/db.rs` so `connect()` returns `PgPool` directly (not wrapped in Arc). The bootstrapper handles ownership:

```rust
pub async fn connect(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                conn.execute("SET search_path TO family_center, public")
                    .await?;
                Ok(())
            })
        })
        .connect(database_url)
        .await?;
    Ok(pool)
}
```

- [ ] **Step 6: Verify it compiles**

Run: `cd /Users/fidalgo/personal/repos/family-center/apps/server && cargo check 2>&1`
Expected: No errors.

- [ ] **Step 7: Commit**

```bash
cd /Users/fidalgo/personal/repos/family-center
git add apps/server/src/configuration/
git commit -m "feat: add IAppConfig, IAppContext DI container, and bootstrapper"
```

---

## Task 5: Refactor Route Handlers to Use IAppContext

Change every API struct from holding `Arc<PgPool>` + `Arc<Config>` to holding `Arc<dyn IAppContext>`. Replace all inline `sqlx::query!` calls with repository method calls via the context.

**Files:**
- Modify: `src/application/routes/mod.rs`
- Modify: `src/application/routes/security.rs`
- Modify: `src/application/routes/auth.rs`
- Modify: `src/application/routes/google.rs`
- Modify: `src/application/routes/sync.rs`
- Modify: `src/application/routes/schedule.rs`
- Modify: `src/application/routes/people.rs`
- Modify: `src/application/routes/activities.rs`
- Modify: `src/application/routes/settings.rs`

- [ ] **Step 1: Update `routes/security.rs`**

Remove `get_household_id` helper (it used `&PgPool` directly). Add `From<DomainError>` for `ApiError`. Remove `sqlx` import:

```rust
use poem_openapi::{
    ApiResponse, Object, SecurityScheme, Tags,
    auth::Bearer,
    payload::Json,
};
use serde::{Deserialize, Serialize};

use crate::domain::error::DomainError;

#[derive(Tags)]
pub enum ApiTags {
    Auth,
    Google,
    Sync,
    Schedule,
    People,
    Activities,
    Settings,
}

#[derive(SecurityScheme)]
#[oai(ty = "bearer")]
pub struct BearerAuth(pub Bearer);

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

#[derive(ApiResponse)]
pub enum ApiError {
    #[oai(status = 400)]
    BadRequest(Json<ErrorBody>),
    #[oai(status = 401)]
    Unauthorized(Json<ErrorBody>),
    #[oai(status = 404)]
    NotFound(Json<ErrorBody>),
    #[oai(status = 500)]
    Internal(Json<ErrorBody>),
}

impl ApiError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        ApiError::BadRequest(Json(ErrorBody {
            code: "BAD_REQUEST".to_string(),
            message: msg.into(),
        }))
    }

    pub fn unauthorized() -> Self {
        ApiError::Unauthorized(Json(ErrorBody {
            code: "UNAUTHORIZED".to_string(),
            message: "Unauthorized".to_string(),
        }))
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        ApiError::NotFound(Json(ErrorBody {
            code: "NOT_FOUND".to_string(),
            message: msg.into(),
        }))
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        ApiError::Internal(Json(ErrorBody {
            code: "INTERNAL_ERROR".to_string(),
            message: msg.into(),
        }))
    }
}

impl From<DomainError> for ApiError {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::NotFound(msg) => ApiError::not_found(msg),
            DomainError::Validation(msg) => ApiError::bad_request(msg),
            DomainError::Unauthorized => ApiError::unauthorized(),
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        tracing::error!("Internal error: {e}");
        ApiError::internal("Internal server error")
    }
}
```

- [ ] **Step 2: Update `routes/mod.rs`**

```rust
pub mod activities;
pub mod auth;
pub mod google;
pub mod people;
pub mod schedule;
pub mod security;
pub mod settings;
pub mod sync;

use std::sync::Arc;
use poem::{Route, IntoEndpoint};
use poem_openapi::OpenApiService;

use crate::configuration::app_context::IAppContext;

use activities::ActivitiesApi;
use auth::AuthApi;
use google::GoogleApi;
use people::PeopleApi;
use schedule::ScheduleApi;
use settings::SettingsApi;
use sync::SyncApi;

pub fn build_routes(context: Arc<dyn IAppContext>) -> Route {
    let auth_api = AuthApi { context: context.clone() };
    let google_api = GoogleApi { context: context.clone() };
    let sync_api = SyncApi { context: context.clone() };
    let schedule_api = ScheduleApi { context: context.clone() };
    let people_api = PeopleApi { context: context.clone() };
    let activities_api = ActivitiesApi { context: context.clone() };
    let settings_api = SettingsApi { context: context.clone() };

    let api_service = OpenApiService::new(
        (
            auth_api,
            google_api,
            sync_api,
            schedule_api,
            people_api,
            activities_api,
            settings_api,
        ),
        "Family Center API",
        "1.0.0",
    )
    .server("/api");

    Route::new()
        .nest("/docs", api_service.scalar())
        .nest("/openapi", api_service.spec_endpoint())
        .nest("/api", api_service.into_endpoint())
}
```

- [ ] **Step 3: Update `routes/auth.rs`**

```rust
use std::sync::Arc;
use poem_openapi::{OpenApi, Object, payload::Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::configuration::app_context::IAppContext;
use crate::infrastructure::auth;
use super::security::{ApiError, ApiTags};

#[derive(Debug, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapRequest {
    pub household_name: Option<String>,
}

#[derive(Debug, Serialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapResponse {
    pub household_id: Uuid,
    pub token: String,
    pub is_new: bool,
}

pub struct AuthApi {
    pub context: Arc<dyn IAppContext>,
}

#[OpenApi(tag = "ApiTags::Auth")]
impl AuthApi {
    #[oai(path = "/auth/bootstrap", method = "post")]
    pub async fn bootstrap(
        &self,
        body: Json<BootstrapRequest>,
    ) -> Result<Json<BootstrapResponse>, ApiError> {
        let repo = self.context.household_repository();
        let settings_repo = self.context.settings_repository();

        let existing = repo.find_first().await.map_err(ApiError::from)?;

        let (household, is_new) = if let Some(h) = existing {
            (h, false)
        } else {
            let name = body.0.household_name.unwrap_or_else(|| "My Family".to_string());
            let h = repo.create(Uuid::new_v4(), &name).await.map_err(ApiError::from)?;
            settings_repo.create_default(h.id).await.map_err(ApiError::from)?;
            (h, true)
        };

        let token = auth::create_token(household.id, self.context.config().jwt_secret())
            .map_err(|e| ApiError::from(anyhow::Error::from(e)))?;

        Ok(Json(BootstrapResponse {
            household_id: household.id,
            token,
            is_new,
        }))
    }
}
```

- [ ] **Step 4: Update `routes/people.rs`**

```rust
use std::sync::Arc;
use poem_openapi::{OpenApi, param::Path, payload::Json};
use uuid::Uuid;

use crate::configuration::app_context::IAppContext;
use crate::domain::entities::person::{CreatePerson, Person, UpdatePerson};
use crate::infrastructure::auth;
use super::security::{ApiError, ApiTags, BearerAuth};

pub struct PeopleApi {
    pub context: Arc<dyn IAppContext>,
}

impl PeopleApi {
    fn verify(&self, auth: &BearerAuth) -> Result<(), ApiError> {
        auth::verify_token(&auth.0.token, self.context.config().jwt_secret())
            .map_err(|_| ApiError::unauthorized())?;
        Ok(())
    }

    async fn household_id(&self) -> Result<Uuid, ApiError> {
        self.context
            .household_repository()
            .find_first()
            .await
            .map_err(ApiError::from)?
            .map(|h| h.id)
            .ok_or_else(|| ApiError::bad_request("No household configured. Call /auth/bootstrap first."))
    }
}

#[OpenApi(tag = "ApiTags::People")]
impl PeopleApi {
    #[oai(path = "/people", method = "get")]
    pub async fn list_people(&self, auth: BearerAuth) -> Result<Json<Vec<Person>>, ApiError> {
        self.verify(&auth)?;
        let hid = self.household_id().await?;
        let people = self.context.person_repository().find_by_household(hid).await.map_err(ApiError::from)?;
        Ok(Json(people))
    }

    #[oai(path = "/people", method = "post")]
    pub async fn create_person(&self, auth: BearerAuth, body: Json<CreatePerson>) -> Result<Json<Person>, ApiError> {
        self.verify(&auth)?;
        let hid = self.household_id().await?;
        let person = self.context.person_repository().create(hid, &body.0).await.map_err(ApiError::from)?;
        Ok(Json(person))
    }

    #[oai(path = "/people/:id", method = "patch")]
    pub async fn update_person(&self, auth: BearerAuth, id: Path<Uuid>, body: Json<UpdatePerson>) -> Result<Json<Person>, ApiError> {
        self.verify(&auth)?;
        let existing = self.context.person_repository().find_by_id(id.0).await.map_err(ApiError::from)?
            .ok_or_else(|| ApiError::not_found(format!("Person {} not found", id.0)))?;
        let person = self.context.person_repository().update(id.0, &existing, &body.0).await.map_err(ApiError::from)?;
        Ok(Json(person))
    }
}
```

- [ ] **Step 5: Update remaining route handlers**

Apply the same pattern to `google.rs`, `sync.rs`, `schedule.rs`, `activities.rs`, `settings.rs`:

- Replace `pool: Arc<PgPool>` + `config: Arc<Config>` with `context: Arc<dyn IAppContext>`
- Replace `self.pool.as_ref()` SQL calls with `self.context.<repo>().<method>()` calls
- Replace `self.config` references with `self.context.config()`
- Replace `get_household_id(pool)` with the `household_id()` helper pattern shown above
- For `sync.rs`, the handler calls `domain::sync::run_sync()` — this will be updated in Task 6

- [ ] **Step 6: Verify it compiles**

Run: `cd /Users/fidalgo/personal/repos/family-center/apps/server && cargo check 2>&1`
Expected: No errors. All routes use repositories via context.

- [ ] **Step 7: Run tests**

Run: `cd /Users/fidalgo/personal/repos/family-center/apps/server && cargo test 2>&1`
Expected: All tests pass.

- [ ] **Step 8: Commit**

```bash
cd /Users/fidalgo/personal/repos/family-center
git add -u apps/server/src/
git commit -m "refactor: route handlers use IAppContext instead of direct PgPool"
```

---

## Task 6: Refactor Domain Sync and Dedupe to Use Repository Traits

Remove all `sqlx` and `PgPool` usage from the domain layer. The sync module becomes a trait + implementation, and the dedupe rebuild function takes repository references.

**Files:**
- Modify: `src/domain/sync/mod.rs`
- Modify: `src/domain/dedupe/mod.rs`
- Create: `src/infrastructure/services/mod.rs`
- Create: `src/infrastructure/services/sync_service.rs`

- [ ] **Step 1: Define `ISyncService` trait in `domain/sync/mod.rs`**

Replace the current file contents. Keep only the trait definition and the Google API fetch function (which is pure HTTP, no DB):

```rust
use async_trait::async_trait;

use crate::domain::entities::sync::{SyncRunRequest, SyncRunResponse};

#[async_trait]
pub trait ISyncService: Send + Sync {
    async fn run_sync(&self, req: SyncRunRequest) -> anyhow::Result<SyncRunResponse>;
}
```

- [ ] **Step 2: Create `infrastructure/services/mod.rs`**

```rust
pub mod sync_service;
```

- [ ] **Step 3: Create `infrastructure/services/sync_service.rs`**

Move all sync logic here. This implementation uses repositories:

```rust
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::configuration::app_context::IAppContext;
use crate::domain::entities::source_event::SourceEvent;
use crate::domain::entities::sync::{SyncError, SyncRunRequest, SyncRunResponse};
use crate::domain::sync::ISyncService;

pub struct SyncService {
    context: Arc<dyn IAppContext>,
}

impl SyncService {
    pub fn new(context: Arc<dyn IAppContext>) -> Self {
        Self { context }
    }
}

#[async_trait]
impl ISyncService for SyncService {
    async fn run_sync(&self, req: SyncRunRequest) -> anyhow::Result<SyncRunResponse> {
        let mut response = SyncRunResponse {
            synced: 0,
            created: 0,
            updated: 0,
            errors: Vec::new(),
        };

        let sources = if let Some(ids) = req.calendar_source_ids {
            self.context.calendar_source_repository().find_selected_with_tokens_by_ids(&ids).await?
        } else {
            self.context.calendar_source_repository().find_selected_with_tokens().await?
        };

        for source in sources {
            let result = self.sync_one_calendar(
                source.id,
                source.access_token.as_deref(),
                req.force_full_sync.unwrap_or(false),
            ).await;
            match result {
                Ok((created, updated)) => {
                    response.synced += 1;
                    response.created += created;
                    response.updated += updated;
                }
                Err(e) => {
                    response.errors.push(SyncError {
                        calendar_source_id: source.id,
                        error: e.to_string(),
                    });
                }
            }
        }

        if response.errors.is_empty() || response.synced > 0 {
            if let Err(e) = self.rebuild_merged_events().await {
                tracing::warn!("Failed to rebuild merged events: {e}");
            }
        }

        Ok(response)
    }
}

impl SyncService {
    async fn sync_one_calendar(
        &self,
        calendar_source_id: Uuid,
        access_token: Option<&str>,
        _force_full: bool,
    ) -> anyhow::Result<(u32, u32)> {
        let events = if self.context.config().mock_calendar() || access_token.is_none() {
            let start = chrono::Utc::now() - chrono::Duration::days(7);
            let end = chrono::Utc::now() + chrono::Duration::days(30);
            crate::infrastructure::google::mock::mock_events_for_range(calendar_source_id, start, end)
        } else {
            let token = access_token.unwrap();
            fetch_google_events(calendar_source_id, token).await?
        };

        let mut created = 0u32;
        let mut updated = 0u32;

        for event in events {
            let is_new = self.context.source_event_repository().upsert(&event).await?;
            if is_new {
                created += 1;
            } else {
                updated += 1;
            }
        }

        Ok((created, updated))
    }

    async fn rebuild_merged_events(&self) -> anyhow::Result<()> {
        let households = self.context.household_repository().find_first().await?;
        if let Some(household) = households {
            self.rebuild_for_household(household.id).await?;
        }
        Ok(())
    }

    async fn rebuild_for_household(&self, household_id: Uuid) -> anyhow::Result<()> {
        let events = self.context.source_event_repository()
            .find_by_household_selected(household_id).await?;
        let rules = self.context.lane_rule_repository()
            .find_by_household(household_id).await?;

        let groups = crate::domain::dedupe::group_events(&events);

        self.context.merged_event_repository()
            .delete_by_household(household_id).await?;

        for group in &groups {
            if group.members.is_empty() { continue; }

            let primary_id = group.members.iter()
                .find(|(_, p)| *p)
                .map(|(id, _)| *id)
                .unwrap_or(group.members[0].0);

            let person_id = events.iter()
                .find(|e| e.id == primary_id)
                .and_then(|e| crate::domain::dedupe::apply_lane_rules(e, &rules));

            self.context.merged_event_repository()
                .insert_group(household_id, group, person_id).await?;
        }

        Ok(())
    }
}

async fn fetch_google_events(
    calendar_source_id: Uuid,
    access_token: &str,
) -> anyhow::Result<Vec<SourceEvent>> {
    // ... (move the existing fetch_google_events function here unchanged)
    let http = reqwest::Client::new();
    let start = chrono::Utc::now() - chrono::Duration::days(7);
    let end = chrono::Utc::now() + chrono::Duration::days(30);

    let url = format!(
        "https://www.googleapis.com/calendar/v3/calendars/primary/events?timeMin={}&timeMax={}&singleEvents=true&maxResults=250",
        start.to_rfc3339(),
        end.to_rfc3339()
    );

    let resp: serde_json::Value = http
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await?
        .json()
        .await?;

    let now = chrono::Utc::now();
    let mut events = Vec::new();

    if let Some(items) = resp["items"].as_array() {
        for item in items {
            let start_str = item["start"]["dateTime"]
                .as_str()
                .or_else(|| item["start"]["date"].as_str())
                .unwrap_or("");
            let end_str = item["end"]["dateTime"]
                .as_str()
                .or_else(|| item["end"]["date"].as_str())
                .unwrap_or("");

            let is_all_day = item["start"]["dateTime"].is_null();

            let start_at = chrono::DateTime::parse_from_rfc3339(start_str)
                .map(|d| d.with_timezone(&chrono::Utc))
                .unwrap_or(now);
            let end_at = chrono::DateTime::parse_from_rfc3339(end_str)
                .map(|d| d.with_timezone(&chrono::Utc))
                .unwrap_or(now);

            let attendees: Option<Vec<String>> = item["attendees"]
                .as_array()
                .map(|arr| arr.iter()
                    .filter_map(|a| a["email"].as_str().map(|s| s.to_string()))
                    .collect());

            events.push(SourceEvent {
                id: uuid::Uuid::new_v4(),
                calendar_source_id,
                google_event_id: item["id"].as_str().unwrap_or("").to_string(),
                ical_uid: item["iCalUID"].as_str().map(|s| s.to_string()),
                title: item["summary"].as_str().unwrap_or("(no title)").to_string(),
                description: item["description"].as_str().map(|s| s.to_string()),
                location: item["location"].as_str().map(|s| s.to_string()),
                start_at,
                end_at,
                is_all_day,
                recurrence_rule: item["recurrence"].as_array()
                    .and_then(|r| r.first())
                    .and_then(|r| r.as_str())
                    .map(|s| s.to_string()),
                recurring_event_id: item["recurringEventId"].as_str().map(|s| s.to_string()),
                organizer: item["organizer"]["email"].as_str().map(|s| s.to_string()),
                attendees,
                raw_json: item.clone(),
                synced_at: now,
                created_at: now,
                updated_at: now,
            });
        }
    }

    Ok(events)
}
```

- [ ] **Step 4: Update `domain/dedupe/mod.rs`**

Remove all `sqlx` usage and the `rebuild_merged_events()` function (it moved to `SyncService`). Keep only the pure logic functions and make `apply_lane_rules` public:

```rust
//! Deduplication and event merging — pure domain logic.
//!
//! Three tiers:
//!   Tier 1 exact:   same iCalUID + normalized start/end
//!   Tier 2 strong:  same normalized title + similar start/end (±5 min) + same organizer
//!   Tier 3 probable: same normalized title + same day/hour bucket + overlapping attendees

use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration, Timelike};

use crate::domain::entities::source_event::SourceEvent;
use crate::domain::entities::lane::LaneAssignmentRule;

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
    pub members: Vec<(Uuid, bool)>,
}

pub fn normalize_title(title: &str) -> String {
    // ... (unchanged)
}

pub fn group_events(events: &[SourceEvent]) -> Vec<EventGroup> {
    // ... (unchanged — the entire existing function)
}

fn glob_match(text: &str, pattern: &str) -> bool {
    // ... (unchanged)
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
```

Note: The `apply_lane_rules` function now takes `&[LaneAssignmentRule]` (domain entity) instead of the private `LaneRule` struct. The private `LaneRule` struct is deleted.

- [ ] **Step 5: Update dedupe tests**

The tests in `domain/dedupe/tests.rs` should still work since `group_events` is unchanged and `SourceEvent` is now imported from `domain::entities`. If the tests use `LaneRule` directly, update the imports.

- [ ] **Step 6: Add ISyncService to IAppContext**

Add to `configuration/app_context.rs`:

```rust
use crate::domain::sync::ISyncService;
```

Add to `IAppContext` trait:
```rust
fn sync_service(&self) -> &dyn ISyncService;
```

Add `sync_service: Arc<dyn ISyncService>` field to `AppContext` struct and implement the getter.

- [ ] **Step 7: Wire SyncService in bootstrapper**

In `configuration/bootstrapper.rs`, after building the context, create the `SyncService`. Note: `SyncService` needs `Arc<dyn IAppContext>`, but we're building it during context construction. The cleanest approach is to build `SyncService` after `AppContext` is created, then use a two-phase init or store it separately.

Simpler approach: make `SyncService` take individual repository references instead of `IAppContext`. Or, accept that `SyncService` is wired at the bootstrapper level and passed to the route that needs it.

The simplest refactor: keep `ISyncService` in the context by using `OnceLock` or by passing repositories directly to `SyncService` instead of `IAppContext`. Use repository refs:

```rust
pub struct SyncService {
    config: Arc<dyn IAppConfig>,
    calendar_source_repo: Arc<dyn ICalendarSourceRepository>,
    source_event_repo: Arc<dyn ISourceEventRepository>,
    household_repo: Arc<dyn IHouseholdRepository>,
    merged_event_repo: Arc<dyn IMergedEventRepository>,
    lane_rule_repo: Arc<dyn ILaneRuleRepository>,
}
```

Wire in bootstrapper:
```rust
let sync_service = Arc::new(SyncService::new(
    config.clone(),
    calendar_source_repo.clone(),
    source_event_repo.clone(),
    household_repo.clone(),
    merged_event_repo.clone(),
    lane_rule_repo.clone(),
));
```

Pass to `AppContext::new()`.

- [ ] **Step 8: Update `routes/sync.rs` to use ISyncService**

```rust
use std::sync::Arc;
use poem_openapi::{OpenApi, payload::Json};

use crate::configuration::app_context::IAppContext;
use crate::domain::entities::sync::{SyncRunRequest, SyncRunResponse};
use crate::infrastructure::auth;
use super::security::{ApiError, ApiTags, BearerAuth};

pub struct SyncApi {
    pub context: Arc<dyn IAppContext>,
}

impl SyncApi {
    fn verify(&self, auth: &BearerAuth) -> Result<(), ApiError> {
        auth::verify_token(&auth.0.token, self.context.config().jwt_secret())
            .map_err(|_| ApiError::unauthorized())?;
        Ok(())
    }
}

#[OpenApi(tag = "ApiTags::Sync")]
impl SyncApi {
    #[oai(path = "/sync/run", method = "post")]
    pub async fn run_sync(
        &self,
        auth: BearerAuth,
        body: Json<SyncRunRequest>,
    ) -> Result<Json<SyncRunResponse>, ApiError> {
        self.verify(&auth)?;
        let response = self.context.sync_service()
            .run_sync(body.0)
            .await
            .map_err(ApiError::from)?;
        Ok(Json(response))
    }
}
```

- [ ] **Step 9: Update `infrastructure/mod.rs`**

```rust
pub mod auth;
pub mod db;
pub mod error;
pub mod google;
pub mod repositories;
pub mod services;
```

- [ ] **Step 10: Verify it compiles**

Run: `cd /Users/fidalgo/personal/repos/family-center/apps/server && cargo check 2>&1`
Expected: No errors.

- [ ] **Step 11: Run tests**

Run: `cd /Users/fidalgo/personal/repos/family-center/apps/server && cargo test 2>&1`
Expected: All dedupe and recurrence tests pass. The domain tests don't need DB.

- [ ] **Step 12: Commit**

```bash
cd /Users/fidalgo/personal/repos/family-center
git add -u apps/server/src/
git add apps/server/src/infrastructure/services/
git commit -m "refactor: extract sync/dedupe DB access to repositories, domain is now pure"
```

---

## Task 7: Update main.rs to Use Bootstrapper

Replace the manual startup in `main.rs` with the bootstrapper.

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Replace `main.rs` contents**

```rust
mod application;
mod configuration;
mod domain;
mod infrastructure;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    configuration::bootstrapper::bootstrap().await
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cd /Users/fidalgo/personal/repos/family-center/apps/server && cargo check 2>&1`
Expected: No errors.

- [ ] **Step 3: Run tests**

Run: `cd /Users/fidalgo/personal/repos/family-center/apps/server && cargo test 2>&1`
Expected: All tests pass.

- [ ] **Step 4: Commit**

```bash
cd /Users/fidalgo/personal/repos/family-center
git add apps/server/src/main.rs
git commit -m "refactor: main.rs uses bootstrapper for startup"
```

---

## Task 8: Clean Up — Remove Dead Code and Verify

Final cleanup pass: remove `infrastructure/models/` directory (if not done in Task 1), remove orphaned imports, ensure the domain module is re-enabled in `main.rs`.

**Files:**
- Verify: All files in `src/domain/` have no `sqlx::PgPool` or `sqlx::query` usage
- Verify: All files in `src/application/routes/` use `Arc<dyn IAppContext>`, not `Arc<PgPool>`
- Verify: `infrastructure/models/` is deleted
- Modify: Any remaining broken imports

- [ ] **Step 1: Grep for violations**

Run: `cd /Users/fidalgo/personal/repos/family-center/apps/server && grep -rn "PgPool" src/domain/ && grep -rn "sqlx::query" src/domain/`
Expected: No matches. Domain layer is pure.

Run: `grep -rn "infrastructure::models" src/`
Expected: No matches. All imports point to `domain::entities`.

- [ ] **Step 2: Full cargo check**

Run: `cd /Users/fidalgo/personal/repos/family-center/apps/server && cargo check 2>&1`
Expected: No errors, no warnings about unused imports.

- [ ] **Step 3: Full test run**

Run: `cd /Users/fidalgo/personal/repos/family-center/apps/server && cargo test 2>&1`
Expected: All tests pass.

- [ ] **Step 4: Final commit**

```bash
cd /Users/fidalgo/personal/repos/family-center
git add -u apps/server/src/
git commit -m "refactor: clean up — remove dead infrastructure/models, verify hexagonal boundaries"
```

---

## Architecture Validation Checklist

After all tasks are complete, verify these invariants:

1. **Domain has zero infrastructure imports**: No `sqlx`, no `PgPool`, no `reqwest` in `src/domain/`
2. **Domain owns all traits**: `I*Repository` traits live in `domain/repositories/`, `ISyncService` in `domain/sync/`
3. **Infrastructure implements traits**: Every `infrastructure/repositories/*.rs` file implements a domain trait
4. **AppContext is the composition root**: Single struct wires all concrete implementations
5. **Bootstrapper handles startup**: `main.rs` is a one-liner calling `bootstrap()`
6. **Routes use `Arc<dyn IAppContext>`**: No route handler holds `PgPool` or `Config` directly
7. **Entities live in domain**: All entity structs are in `domain/entities/`, not `infrastructure/models/`
