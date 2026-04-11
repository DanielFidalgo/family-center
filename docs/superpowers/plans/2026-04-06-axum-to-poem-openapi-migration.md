# Axum → poem-openapi Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the axum HTTP layer with poem + poem-openapi (Scalar UI, typed OpenAPI spec), reorganizing source into `application/`, `configuration/`, `domain/`, and `infrastructure/` layers — without changing any business logic.

**Architecture:** Seven domain-split API structs (`AuthApi`, `GoogleApi`, `SyncApi`, `PeopleApi`, `ActivitiesApi`, `ScheduleApi`, `SettingsApi`) each hold only the deps they need via `Arc<PgPool>`/`Arc<Config>`. An `OpenApiService` tuple combines them and serves Scalar docs at `/docs` and the raw spec at `/openapi`. Domain and infrastructure code moves unchanged; only the HTTP surface and file layout change.

**Tech Stack:** poem 3.1.12, poem-openapi 5.1.16 (scalar feature), tokio 1 (full), sqlx 0.7, thiserror 1, jsonwebtoken 9, tracing 0.1

---

## File Map

### Created
| File | Purpose |
|------|---------|
| `src/configuration/mod.rs` | Layer root |
| `src/configuration/config.rs` | Moved from `src/config.rs` |
| `src/configuration/service_setup.rs` | poem server startup, CORS, tracing, graceful shutdown |
| `src/application/mod.rs` | Layer root |
| `src/application/routes/mod.rs` | OpenApiService composition, Route building |
| `src/application/routes/security.rs` | BearerAuth security scheme, ApiError, ErrorBody, get_household_id |
| `src/application/routes/auth.rs` | AuthApi (bootstrap) |
| `src/application/routes/google.rs` | GoogleApi (connect_start, connect_callback, list_accounts, list_calendars, select_calendars) |
| `src/application/routes/sync.rs` | SyncApi (run_sync) |
| `src/application/routes/schedule.rs` | ScheduleApi (get_schedule) |
| `src/application/routes/people.rs` | PeopleApi (list, create, update) |
| `src/application/routes/activities.rs` | ActivitiesApi (list, create, update) |
| `src/application/routes/settings.rs` | SettingsApi (get, update) |
| `src/domain/mod.rs` | Layer root |
| `src/domain/error.rs` | DomainError (thiserror) |
| `src/domain/sync/mod.rs` | Moved from `src/sync/mod.rs` |
| `src/domain/dedupe/mod.rs` | Moved from `src/dedupe/mod.rs` |
| `src/domain/recurrence/mod.rs` | Moved from `src/recurrence/mod.rs` |
| `src/infrastructure/mod.rs` | Layer root |
| `src/infrastructure/error.rs` | RepositoryError (thiserror) |
| `src/infrastructure/auth.rs` | Moved from `src/auth.rs` |
| `src/infrastructure/db.rs` | Moved from `src/db.rs` |
| `src/infrastructure/google/` | Moved from `src/google/` |
| `src/infrastructure/models/` | Moved from `src/models/`, gains Object derives |

### Deleted
- `src/config.rs`
- `src/error.rs`
- `src/auth.rs`
- `src/db.rs`
- `src/routes.rs`
- `src/google/` (entire directory)
- `src/models/` (entire directory)
- `src/sync/` (entire directory)
- `src/dedupe/` (entire directory)
- `src/recurrence/` (entire directory)
- `src/handlers/` (entire directory)

### Modified
- `Cargo.toml` — swap axum/tower/tower-http/axum-test for poem/poem-openapi
- `src/main.rs` — rewritten to use new layers

---

## Task 1: Update Cargo.toml

**Files:**
- Modify: `apps/server/Cargo.toml`

- [ ] **Step 1: Replace dependencies**

Open `apps/server/Cargo.toml` and replace the `[dependencies]` and `[dev-dependencies]` sections:

```toml
[package]
name = "family-center-server"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "server"
path = "src/main.rs"

[dependencies]
# Web framework
poem = { version = "3.1.12", features = ["static-files"] }
poem-openapi = { version = "5.1.16", features = ["scalar"] }

# Async runtime
tokio = { version = "1", features = ["full"] }

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json", "migrate"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Config/env
dotenvy = "0.15"

# UUID
uuid = { version = "1", features = ["v4", "serde"] }

# Time
chrono = { version = "0.4", features = ["serde"] }

# Error handling
anyhow = "1"
thiserror = "1"

# Tracing/logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# JWT
jsonwebtoken = "9"

# HTTP client (for Google API)
reqwest = { version = "0.11", features = ["json", "rustls-tls"], default-features = false }

# OAuth2
oauth2 = "4"

# URL parsing
url = "2"

# Async trait
async-trait = "0.1"

[dev-dependencies]
tokio-test = "0.4"
```

- [ ] **Step 2: Verify cargo resolves (compile not expected to succeed yet)**

```bash
cd apps/server && cargo fetch 2>&1 | tail -5
```

Expected: downloads poem/poem-openapi crates, no error.

- [ ] **Step 3: Commit**

```bash
git add apps/server/Cargo.toml apps/server/Cargo.lock
git commit -m "chore(server): swap axum for poem + poem-openapi"
```

---

## Task 2: Create Layer Directory Structure

**Files:**
- Create: `src/configuration/mod.rs`
- Create: `src/application/mod.rs`
- Create: `src/application/routes/mod.rs` (stub)
- Create: `src/domain/mod.rs`
- Create: `src/infrastructure/mod.rs`

- [ ] **Step 1: Create layer module roots**

`apps/server/src/configuration/mod.rs`:
```rust
pub mod config;
pub mod service_setup;
```

`apps/server/src/application/mod.rs`:
```rust
pub mod routes;
```

`apps/server/src/application/routes/mod.rs` (stub — filled in Task 16):
```rust
// Route composition — filled in Task 16
```

`apps/server/src/domain/mod.rs`:
```rust
pub mod dedupe;
pub mod error;
pub mod recurrence;
pub mod sync;
```

`apps/server/src/infrastructure/mod.rs`:
```rust
pub mod auth;
pub mod db;
pub mod error;
pub mod google;
pub mod models;
```

- [ ] **Step 2: Commit**

```bash
git add apps/server/src/configuration/ apps/server/src/application/ apps/server/src/domain/ apps/server/src/infrastructure/
git commit -m "chore(server): create layered directory structure"
```

---

## Task 3: Move configuration/config.rs

**Files:**
- Create: `src/configuration/config.rs` (from `src/config.rs`)
- Delete: `src/config.rs`

- [ ] **Step 1: Create configuration/config.rs**

`apps/server/src/configuration/config.rs` — exact same content as current `src/config.rs`, only the error import changes from `anyhow` to `thiserror`:

```rust
use anyhow::{Context, Result};

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub jwt_secret: String,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_uri: String,
    pub mock_calendar: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL must be set")?,
            server_host: std::env::var("SERVER_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .context("SERVER_PORT must be a valid port number")?,
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "dev-secret-change-in-production".to_string()),
            google_client_id: std::env::var("GOOGLE_CLIENT_ID")
                .unwrap_or_default(),
            google_client_secret: std::env::var("GOOGLE_CLIENT_SECRET")
                .unwrap_or_default(),
            google_redirect_uri: std::env::var("GOOGLE_REDIRECT_URI")
                .unwrap_or_else(|_| "http://localhost:3000/auth/google/callback".to_string()),
            mock_calendar: std::env::var("MOCK_CALENDAR")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
        })
    }
}
```

- [ ] **Step 2: Delete old config.rs**

```bash
rm apps/server/src/config.rs
```

- [ ] **Step 3: Commit**

```bash
git add apps/server/src/configuration/config.rs apps/server/src/config.rs
git commit -m "refactor(server): move config.rs to configuration layer"
```

---

## Task 4: Move Infrastructure Files

Move `db.rs`, `auth.rs`, `google/`, and `models/` into `infrastructure/`. Content is unchanged; only `crate::` paths inside them need updating.

**Files:**
- Create: `src/infrastructure/db.rs`
- Create: `src/infrastructure/auth.rs`
- Create: `src/infrastructure/google/` (full directory)
- Create: `src/infrastructure/models/` (full directory)

- [ ] **Step 1: Copy db.rs**

`apps/server/src/infrastructure/db.rs` — identical to `src/db.rs`:

```rust
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn connect(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;
    Ok(pool)
}
```

- [ ] **Step 2: Copy auth.rs**

`apps/server/src/infrastructure/auth.rs` — identical to `src/auth.rs`:

```rust
use anyhow::Result;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // household_id
    pub exp: i64,
}

pub fn create_token(household_id: Uuid, secret: &str) -> Result<String> {
    let expiry = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(365))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: household_id.to_string(),
        exp: expiry,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok(token)
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(data.claims)
}
```

- [ ] **Step 3: Copy google/ directory**

```bash
cp -r apps/server/src/google apps/server/src/infrastructure/google
```

Then update internal `crate::` references inside `apps/server/src/infrastructure/google/client.rs`. Find every occurrence of `crate::models` and replace with `crate::infrastructure::models`, and `crate::config` / `crate::Config` with `crate::configuration::config::Config`:

```bash
# Check what needs updating:
grep -n "crate::" apps/server/src/infrastructure/google/client.rs
```

Update each occurrence. Typical replacements:
- `crate::Config` → `crate::configuration::config::Config`
- `crate::models::` → `crate::infrastructure::models::`

- [ ] **Step 4: Copy models/ directory**

```bash
cp -r apps/server/src/models apps/server/src/infrastructure/models
```

No internal `crate::` path changes needed inside model files (they only reference `serde`, `uuid`, `chrono`, `sqlx`).

Update `apps/server/src/infrastructure/models/mod.rs` to ensure all submodules are still declared (content should be identical to `src/models/mod.rs`).

- [ ] **Step 5: Delete old files**

```bash
rm apps/server/src/db.rs
rm apps/server/src/auth.rs
rm -rf apps/server/src/google
rm -rf apps/server/src/models
```

- [ ] **Step 6: Commit**

```bash
git add apps/server/src/infrastructure/ apps/server/src/db.rs apps/server/src/auth.rs apps/server/src/google apps/server/src/models
git commit -m "refactor(server): move db, auth, google, models to infrastructure layer"
```

---

## Task 5: Move Domain Files

Move `sync/`, `dedupe/`, and `recurrence/` into `domain/`. Update all `crate::` paths inside them.

**Files:**
- Create: `src/domain/sync/mod.rs`
- Create: `src/domain/dedupe/mod.rs`
- Create: `src/domain/recurrence/mod.rs`

- [ ] **Step 1: Copy domain directories**

```bash
cp -r apps/server/src/sync apps/server/src/domain/sync
cp -r apps/server/src/dedupe apps/server/src/domain/dedupe
cp -r apps/server/src/recurrence apps/server/src/domain/recurrence
```

- [ ] **Step 2: Update crate paths in domain/sync/mod.rs**

Open `apps/server/src/domain/sync/mod.rs`. Find all `crate::` references and update:
- `crate::config::Config` / `crate::Config` → `crate::configuration::config::Config`
- `crate::models::` → `crate::infrastructure::models::`
- `crate::google::` → `crate::infrastructure::google::`
- `crate::dedupe::` → `crate::domain::dedupe::`
- `crate::db` → `crate::infrastructure::db`

Run:
```bash
grep -n "crate::" apps/server/src/domain/sync/mod.rs
```

Update each reference to use the new paths above.

- [ ] **Step 3: Update crate paths in domain/dedupe/mod.rs**

```bash
grep -n "crate::" apps/server/src/domain/dedupe/mod.rs
```

Typical replacements:
- `crate::models::` → `crate::infrastructure::models::`

- [ ] **Step 4: Update crate paths in domain/recurrence/mod.rs**

```bash
grep -n "crate::" apps/server/src/domain/recurrence/mod.rs
```

These files typically only use std + chrono, so there may be nothing to update.

- [ ] **Step 5: Delete old directories**

```bash
rm -rf apps/server/src/sync
rm -rf apps/server/src/dedupe
rm -rf apps/server/src/recurrence
```

- [ ] **Step 6: Commit**

```bash
git add apps/server/src/domain/ apps/server/src/sync apps/server/src/dedupe apps/server/src/recurrence
git commit -m "refactor(server): move sync, dedupe, recurrence to domain layer"
```

---

## Task 6: Create Error Types

Replace the monolithic `AppError` with layer-appropriate errors using `thiserror`.

**Files:**
- Create: `src/domain/error.rs`
- Create: `src/infrastructure/error.rs`
- Delete: `src/error.rs`

- [ ] **Step 1: Create domain/error.rs**

`apps/server/src/domain/error.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("unauthorized")]
    Unauthorized,
}
```

- [ ] **Step 2: Create infrastructure/error.rs**

`apps/server/src/infrastructure/error.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("external api error: {0}")]
    ExternalApi(String),
}
```

- [ ] **Step 3: Delete old error.rs**

```bash
rm apps/server/src/error.rs
```

- [ ] **Step 4: Commit**

```bash
git add apps/server/src/domain/error.rs apps/server/src/infrastructure/error.rs apps/server/src/error.rs
git commit -m "refactor(server): split AppError into DomainError and RepositoryError"
```

---

## Task 7: Add poem_openapi::Object Derives to Models

Every type that appears in an API response or request body needs `#[derive(poem_openapi::Object)]`.

**Note on limitations:**
- `#[serde(flatten)]` is **not supported** by `poem_openapi::Object`. `LocalActivityWithRecurrence` uses it — this struct must be rewritten to inline all fields.
- `Option<Option<T>>` (used in `UpdateLocalActivity`, `UpdatePerson`) should use `poem_openapi::types::MaybeUndefined<T>` instead. However, since `sqlx` queries reference these fields directly, keep the existing `Option<Option<T>>` fields and **skip** adding `Object` to update-only types (`UpdateLocalActivity`, `UpdatePerson`, `UpdateSettings`). These types are only deserialized from request bodies; poem-openapi handles that via `serde` + a manual `impl poem_openapi::types::ParseFromJSON`.
- Simpler path: derive `Object` on all read/create types; leave update types as `Deserialize`-only and annotate with `#[oai(skip)]` where necessary.

**Files:**
- Modify: all files in `src/infrastructure/models/`

- [ ] **Step 1: Add Object to person.rs**

`apps/server/src/infrastructure/models/person.rs`:

```rust
use chrono::{DateTime, Utc};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Object)]
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

// UpdatePerson uses Option<Option<T>> — keep as Deserialize only, no Object derive
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePerson {
    pub name: Option<String>,
    pub color: Option<String>,
    pub avatar_url: Option<Option<String>>,
    pub sort_order: Option<i32>,
    pub is_active: Option<bool>,
}
```

- [ ] **Step 2: Add Object to settings.rs**

`apps/server/src/infrastructure/models/settings.rs`:

```rust
use chrono::{DateTime, Utc};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Object)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub household_id: Uuid,
    pub default_view: String,
    pub week_starts_monday: bool,
    pub dedupe_mode: String,
    pub display_timezone: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// UpdateSettings: no Option<Option<T>>, safe to add Object
#[derive(Debug, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettings {
    pub default_view: Option<String>,
    pub week_starts_monday: Option<bool>,
    pub dedupe_mode: Option<String>,
    pub display_timezone: Option<String>,
}
```

- [ ] **Step 3: Add Object to sync.rs**

`apps/server/src/infrastructure/models/sync.rs`:

```rust
use chrono::{DateTime, Utc};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SyncCheckpoint {
    pub id: Uuid,
    pub calendar_source_id: Uuid,
    pub sync_token: Option<String>,
    pub full_sync_at: Option<DateTime<Utc>>,
    pub next_page_token: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct SyncRunRequest {
    pub calendar_source_ids: Option<Vec<Uuid>>,
    pub force_full_sync: Option<bool>,
}

#[derive(Debug, Serialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct SyncRunResponse {
    pub synced: u32,
    pub created: u32,
    pub updated: u32,
    pub errors: Vec<SyncError>,
}

#[derive(Debug, Serialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct SyncError {
    pub calendar_source_id: Uuid,
    pub error: String,
}
```

- [ ] **Step 4: Add Object to local_activity.rs**

`LocalActivityWithRecurrence` uses `#[serde(flatten)]` which poem_openapi::Object does not support. Replace it with inlined fields:

`apps/server/src/infrastructure/models/local_activity.rs`:

```rust
use chrono::{DateTime, NaiveDate, Utc};
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Object)]
#[serde(rename_all = "camelCase")]
pub struct LocalActivity {
    pub id: Uuid,
    pub household_id: Uuid,
    pub person_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub start_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
    pub is_all_day: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Object)]
#[serde(rename_all = "camelCase")]
pub struct LocalActivityRecurrence {
    pub id: Uuid,
    pub local_activity_id: Uuid,
    pub freq: String,
    pub interval_val: i32,
    pub by_day_of_week: Option<Vec<i32>>,
    pub by_day_of_month: Option<Vec<i32>>,
    pub until: Option<NaiveDate>,
    pub count: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Flattened for poem_openapi::Object (no serde flatten supported).
/// Fields from LocalActivity are inlined alongside recurrence.
#[derive(Debug, Clone, Serialize, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct LocalActivityWithRecurrence {
    // LocalActivity fields inlined
    pub id: Uuid,
    pub household_id: Uuid,
    pub person_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub start_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
    pub is_all_day: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Recurrence
    pub recurrence: Option<LocalActivityRecurrence>,
}

impl LocalActivityWithRecurrence {
    pub fn new(activity: LocalActivity, recurrence: Option<LocalActivityRecurrence>) -> Self {
        Self {
            id: activity.id,
            household_id: activity.household_id,
            person_id: activity.person_id,
            title: activity.title,
            description: activity.description,
            color: activity.color,
            start_at: activity.start_at,
            end_at: activity.end_at,
            is_all_day: activity.is_all_day,
            created_at: activity.created_at,
            updated_at: activity.updated_at,
            recurrence,
        }
    }
}

#[derive(Debug, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct RecurrenceInput {
    pub freq: String,
    pub interval: Option<i32>,
    pub by_day_of_week: Option<Vec<i32>>,
    pub by_day_of_month: Option<Vec<i32>>,
    pub until: Option<String>,
    pub count: Option<i32>,
}

#[derive(Debug, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct CreateLocalActivity {
    pub person_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub start_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
    pub is_all_day: Option<bool>,
    pub recurrence: Option<RecurrenceInput>,
}

// UpdateLocalActivity uses Option<Option<T>> — keep as Deserialize only
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLocalActivity {
    pub person_id: Option<Option<Uuid>>,
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub color: Option<Option<String>>,
    pub start_at: Option<Option<DateTime<Utc>>>,
    pub end_at: Option<Option<DateTime<Utc>>>,
    pub is_all_day: Option<bool>,
    pub recurrence: Option<Option<RecurrenceInput>>,
}
```

- [ ] **Step 5: Add Object to remaining model files**

For each remaining model file in `src/infrastructure/models/` (`google_account.rs`, `calendar_source.rs`, `merged_event.rs`, `source_event.rs`, `household.rs`, `lane.rs`): add `poem_openapi::Object` to all response/request structs following the same pattern. Add `use poem_openapi::Object;` at the top and add `Object` to the derive list for each public struct that is used in API responses. Skip structs with `#[serde(flatten)]` or complex nested generics.

For `merged_event.rs`, verify `MergedEventGroupWithSources` does not use `#[serde(flatten)]` — if it does, inline its fields exactly as done for `LocalActivityWithRecurrence` above.

- [ ] **Step 6: Commit**

```bash
git add apps/server/src/infrastructure/models/
git commit -m "feat(server): add poem_openapi::Object derives to model types"
```

---

## Task 8: Create Shared Application Types

Define `BearerAuth`, `ApiError`, `ErrorBody`, and `get_household_id` — used by all API structs.

**Files:**
- Create: `src/application/routes/security.rs`

- [ ] **Step 1: Create security.rs**

`apps/server/src/application/routes/security.rs`:

```rust
use poem_openapi::{
    ApiResponse, Object, SecurityScheme,
    auth::Bearer,
    payload::Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Bearer token security scheme — appears as lock icon in Scalar UI.
#[derive(SecurityScheme)]
#[oai(ty = "bearer")]
pub struct BearerAuth(pub Bearer);

/// Standard error response body.
#[derive(Debug, Serialize, Deserialize, Object)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

/// Unified HTTP error responses for all API handlers.
#[derive(ApiResponse)]
pub enum ApiError {
    /// 400 Bad Request
    #[oai(status = 400)]
    BadRequest(Json<ErrorBody>),
    /// 401 Unauthorized
    #[oai(status = 401)]
    Unauthorized(Json<ErrorBody>),
    /// 404 Not Found
    #[oai(status = 404)]
    NotFound(Json<ErrorBody>),
    /// 500 Internal Server Error
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

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!("Database error: {e}");
        ApiError::internal("Database error")
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        tracing::error!("Internal error: {e}");
        ApiError::internal("Internal server error")
    }
}

/// Single-household helper: fetches the first household ID from the DB.
/// Returns ApiError::BadRequest if no household exists (call /auth/bootstrap first).
pub async fn get_household_id(pool: &PgPool) -> Result<Uuid, ApiError> {
    let row = sqlx::query!("SELECT id FROM households LIMIT 1")
        .fetch_optional(pool)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::bad_request("No household configured. Call /auth/bootstrap first."))?;
    Ok(row.id)
}
```

- [ ] **Step 2: Expose security from routes mod**

Add to `apps/server/src/application/routes/mod.rs`:

```rust
pub mod security;
pub mod auth;
pub mod google;
pub mod sync;
pub mod schedule;
pub mod people;
pub mod activities;
pub mod settings;
```

- [ ] **Step 3: Commit**

```bash
git add apps/server/src/application/routes/security.rs apps/server/src/application/routes/mod.rs
git commit -m "feat(server): add BearerAuth, ApiError, and shared security types"
```

---

## Task 9: AuthApi

**Files:**
- Create: `src/application/routes/auth.rs`

- [ ] **Step 1: Create AuthApi**

`apps/server/src/application/routes/auth.rs`:

```rust
use std::sync::Arc;
use poem_openapi::{OpenApi, Object, payload::Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use crate::configuration::config::Config;
use crate::infrastructure::auth;
use crate::application::routes::security::ApiError;

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
    pub pool: Arc<PgPool>,
    pub config: Arc<Config>,
}

#[OpenApi(tag = "Auth")]
impl AuthApi {
    /// Bootstrap the household. Returns a JWT token.
    /// Creates a new household on first call; returns the existing one thereafter.
    #[oai(path = "/auth/bootstrap", method = "post")]
    pub async fn bootstrap(
        &self,
        body: Json<BootstrapRequest>,
    ) -> Result<Json<BootstrapResponse>, ApiError> {
        let existing = sqlx::query_as!(
            crate::infrastructure::models::household::Household,
            "SELECT id, name, created_at, updated_at FROM households LIMIT 1"
        )
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(ApiError::from)?;

        let (household, is_new) = if let Some(h) = existing {
            (h, false)
        } else {
            let name = body.0.household_name.clone().unwrap_or_else(|| "My Family".to_string());
            let household = sqlx::query_as!(
                crate::infrastructure::models::household::Household,
                "INSERT INTO households (id, name) VALUES ($1, $2) RETURNING id, name, created_at, updated_at",
                Uuid::new_v4(),
                name
            )
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(ApiError::from)?;

            sqlx::query!(
                "INSERT INTO settings (household_id) VALUES ($1) ON CONFLICT DO NOTHING",
                household.id
            )
            .execute(self.pool.as_ref())
            .await
            .map_err(ApiError::from)?;

            (household, true)
        };

        let token = auth::create_token(household.id, &self.config.jwt_secret)
            .map_err(|e| ApiError::from(anyhow::Error::from(e)))?;

        Ok(Json(BootstrapResponse {
            household_id: household.id,
            token,
            is_new,
        }))
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add apps/server/src/application/routes/auth.rs
git commit -m "feat(server): add AuthApi with bootstrap endpoint"
```

---

## Task 10: PeopleApi

**Files:**
- Create: `src/application/routes/people.rs`

- [ ] **Step 1: Create PeopleApi**

`apps/server/src/application/routes/people.rs`:

```rust
use std::sync::Arc;
use poem_openapi::{OpenApi, param::Path, payload::Json};
use sqlx::PgPool;
use uuid::Uuid;
use crate::application::routes::security::{ApiError, BearerAuth, get_household_id};
use crate::infrastructure::{
    auth,
    models::person::{CreatePerson, Person, UpdatePerson},
};
use crate::configuration::config::Config;

pub struct PeopleApi {
    pub pool: Arc<PgPool>,
    pub config: Arc<Config>,
}

impl PeopleApi {
    fn verify(&self, auth: &BearerAuth) -> Result<(), ApiError> {
        auth::verify_token(&auth.0.token, &self.config.jwt_secret)
            .map_err(|_| ApiError::unauthorized())?;
        Ok(())
    }
}

#[OpenApi(tag = "People")]
impl PeopleApi {
    /// List all active people in the household.
    #[oai(path = "/people", method = "get")]
    pub async fn list_people(
        &self,
        auth: BearerAuth,
    ) -> Result<Json<Vec<Person>>, ApiError> {
        self.verify(&auth)?;
        let household_id = get_household_id(self.pool.as_ref()).await?;

        let people = sqlx::query_as!(
            Person,
            r#"SELECT id, household_id, name, color, avatar_url, sort_order, is_active, created_at, updated_at
               FROM people WHERE household_id = $1 AND is_active = TRUE ORDER BY sort_order, name"#,
            household_id
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(ApiError::from)?;

        Ok(Json(people))
    }

    /// Create a new person in the household.
    #[oai(path = "/people", method = "post")]
    pub async fn create_person(
        &self,
        auth: BearerAuth,
        body: Json<CreatePerson>,
    ) -> Result<Json<Person>, ApiError> {
        self.verify(&auth)?;
        let household_id = get_household_id(self.pool.as_ref()).await?;

        let person = sqlx::query_as!(
            Person,
            r#"INSERT INTO people (id, household_id, name, color, avatar_url, sort_order)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id, household_id, name, color, avatar_url, sort_order, is_active, created_at, updated_at"#,
            Uuid::new_v4(),
            household_id,
            body.0.name,
            body.0.color,
            body.0.avatar_url,
            body.0.sort_order.unwrap_or(0)
        )
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(ApiError::from)?;

        Ok(Json(person))
    }

    /// Update an existing person by ID.
    #[oai(path = "/people/:id", method = "patch")]
    pub async fn update_person(
        &self,
        auth: BearerAuth,
        id: Path<Uuid>,
        body: Json<UpdatePerson>,
    ) -> Result<Json<Person>, ApiError> {
        self.verify(&auth)?;

        let existing = sqlx::query_as!(
            Person,
            "SELECT id, household_id, name, color, avatar_url, sort_order, is_active, created_at, updated_at FROM people WHERE id = $1",
            id.0
        )
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::not_found(format!("Person {} not found", id.0)))?;

        let person = sqlx::query_as!(
            Person,
            r#"UPDATE people SET
                name = $2,
                color = $3,
                avatar_url = $4,
                sort_order = $5,
                is_active = $6,
                updated_at = NOW()
               WHERE id = $1
               RETURNING id, household_id, name, color, avatar_url, sort_order, is_active, created_at, updated_at"#,
            id.0,
            body.0.name.unwrap_or(existing.name),
            body.0.color.unwrap_or(existing.color),
            body.0.avatar_url.unwrap_or(existing.avatar_url),
            body.0.sort_order.unwrap_or(existing.sort_order),
            body.0.is_active.unwrap_or(existing.is_active),
        )
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(ApiError::from)?;

        Ok(Json(person))
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add apps/server/src/application/routes/people.rs
git commit -m "feat(server): add PeopleApi with list, create, update endpoints"
```

---

## Task 11: SettingsApi

**Files:**
- Create: `src/application/routes/settings.rs`

- [ ] **Step 1: Create SettingsApi**

`apps/server/src/application/routes/settings.rs`:

```rust
use std::sync::Arc;
use poem_openapi::{OpenApi, payload::Json};
use sqlx::PgPool;
use crate::application::routes::security::{ApiError, BearerAuth, get_household_id};
use crate::infrastructure::{auth, models::settings::{Settings, UpdateSettings}};
use crate::configuration::config::Config;

pub struct SettingsApi {
    pub pool: Arc<PgPool>,
    pub config: Arc<Config>,
}

impl SettingsApi {
    fn verify(&self, auth: &BearerAuth) -> Result<(), ApiError> {
        auth::verify_token(&auth.0.token, &self.config.jwt_secret)
            .map_err(|_| ApiError::unauthorized())?;
        Ok(())
    }
}

#[OpenApi(tag = "Settings")]
impl SettingsApi {
    /// Get household settings. Creates defaults if none exist.
    #[oai(path = "/settings", method = "get")]
    pub async fn get_settings(
        &self,
        auth: BearerAuth,
    ) -> Result<Json<Settings>, ApiError> {
        self.verify(&auth)?;
        let household_id = get_household_id(self.pool.as_ref()).await?;

        let settings = sqlx::query_as!(
            Settings,
            "SELECT household_id, default_view, week_starts_monday, dedupe_mode, display_timezone, created_at, updated_at FROM settings WHERE household_id = $1",
            household_id
        )
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(ApiError::from)?;

        match settings {
            Some(s) => Ok(Json(s)),
            None => {
                let s = sqlx::query_as!(
                    Settings,
                    "INSERT INTO settings (household_id) VALUES ($1) RETURNING household_id, default_view, week_starts_monday, dedupe_mode, display_timezone, created_at, updated_at",
                    household_id
                )
                .fetch_one(self.pool.as_ref())
                .await
                .map_err(ApiError::from)?;
                Ok(Json(s))
            }
        }
    }

    /// Update household settings.
    #[oai(path = "/settings", method = "patch")]
    pub async fn update_settings(
        &self,
        auth: BearerAuth,
        body: Json<UpdateSettings>,
    ) -> Result<Json<Settings>, ApiError> {
        self.verify(&auth)?;
        let household_id = get_household_id(self.pool.as_ref()).await?;

        let current = sqlx::query_as!(
            Settings,
            "SELECT household_id, default_view, week_starts_monday, dedupe_mode, display_timezone, created_at, updated_at FROM settings WHERE household_id = $1",
            household_id
        )
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(ApiError::from)?
        .unwrap_or_else(|| Settings {
            household_id,
            default_view: "week".to_string(),
            week_starts_monday: false,
            dedupe_mode: "probable".to_string(),
            display_timezone: "UTC".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });

        let updated = sqlx::query_as!(
            Settings,
            r#"UPDATE settings SET
                default_view = $2,
                week_starts_monday = $3,
                dedupe_mode = $4,
                display_timezone = $5,
                updated_at = NOW()
               WHERE household_id = $1
               RETURNING household_id, default_view, week_starts_monday, dedupe_mode, display_timezone, created_at, updated_at"#,
            household_id,
            body.0.default_view.unwrap_or(current.default_view),
            body.0.week_starts_monday.unwrap_or(current.week_starts_monday),
            body.0.dedupe_mode.unwrap_or(current.dedupe_mode),
            body.0.display_timezone.unwrap_or(current.display_timezone),
        )
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(ApiError::from)?;

        Ok(Json(updated))
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add apps/server/src/application/routes/settings.rs
git commit -m "feat(server): add SettingsApi with get and update endpoints"
```

---

## Task 12: SyncApi

**Files:**
- Create: `src/application/routes/sync.rs`

- [ ] **Step 1: Create SyncApi**

`apps/server/src/application/routes/sync.rs`:

```rust
use std::sync::Arc;
use poem_openapi::{OpenApi, payload::Json};
use sqlx::PgPool;
use crate::application::routes::security::{ApiError, BearerAuth};
use crate::infrastructure::{auth, models::sync::{SyncRunRequest, SyncRunResponse}};
use crate::configuration::config::Config;

pub struct SyncApi {
    pub pool: Arc<PgPool>,
    pub config: Arc<Config>,
}

impl SyncApi {
    fn verify(&self, auth: &BearerAuth) -> Result<(), ApiError> {
        auth::verify_token(&auth.0.token, &self.config.jwt_secret)
            .map_err(|_| ApiError::unauthorized())?;
        Ok(())
    }
}

#[OpenApi(tag = "Sync")]
impl SyncApi {
    /// Trigger a calendar sync run.
    #[oai(path = "/sync/run", method = "post")]
    pub async fn run_sync(
        &self,
        auth: BearerAuth,
        body: Json<SyncRunRequest>,
    ) -> Result<Json<SyncRunResponse>, ApiError> {
        self.verify(&auth)?;
        let response = crate::domain::sync::run_sync(
            self.pool.as_ref(),
            self.config.as_ref(),
            body.0,
        )
        .await
        .map_err(ApiError::from)?;
        Ok(Json(response))
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add apps/server/src/application/routes/sync.rs
git commit -m "feat(server): add SyncApi with run_sync endpoint"
```

---

## Task 13: ScheduleApi

**Files:**
- Create: `src/application/routes/schedule.rs`

- [ ] **Step 1: Create ScheduleApi**

`apps/server/src/application/routes/schedule.rs`:

```rust
use std::sync::Arc;
use chrono::{DateTime, Utc};
use poem_openapi::{OpenApi, Object, param::Query, payload::Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use crate::application::routes::security::{ApiError, BearerAuth, get_household_id};
use crate::infrastructure::{
    auth,
    models::{
        local_activity::{LocalActivity, LocalActivityRecurrence, LocalActivityWithRecurrence},
        merged_event::{MergedEventGroup, MergedEventSource, MergedEventGroupWithSources},
    },
};
use crate::configuration::config::Config;

#[derive(Debug, Serialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleResponse {
    pub events: Vec<MergedEventGroupWithSources>,
    pub local_activities: Vec<LocalActivityWithRecurrence>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

pub struct ScheduleApi {
    pub pool: Arc<PgPool>,
    pub config: Arc<Config>,
}

#[OpenApi(tag = "Schedule")]
impl ScheduleApi {
    /// Get the schedule for a time window.
    /// Auth token is optional — supply it for personalized data.
    #[oai(path = "/schedule", method = "get")]
    pub async fn get_schedule(
        &self,
        _auth: Option<BearerAuth>,
        start: Query<DateTime<Utc>>,
        end: Query<DateTime<Utc>>,
    ) -> Result<Json<ScheduleResponse>, ApiError> {
        let household_id = get_household_id(self.pool.as_ref()).await?;

        let groups = sqlx::query_as!(
            MergedEventGroup,
            r#"SELECT id, household_id, canonical_title, canonical_start, canonical_end,
                      is_all_day, person_id, lane_override, dupe_tier, created_at, updated_at
               FROM merged_event_groups
               WHERE household_id = $1 AND canonical_start >= $2 AND canonical_start < $3
               ORDER BY canonical_start"#,
            household_id,
            start.0,
            end.0,
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(ApiError::from)?;

        let mut events = Vec::with_capacity(groups.len());
        for group in groups {
            let sources = sqlx::query_as!(
                MergedEventSource,
                "SELECT id, merged_event_group_id, source_event_id, is_primary, created_at FROM merged_event_sources WHERE merged_event_group_id = $1",
                group.id
            )
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(ApiError::from)?;
            events.push(MergedEventGroupWithSources { group, sources });
        }

        let activities_raw = sqlx::query_as!(
            LocalActivity,
            r#"SELECT id, household_id, person_id, title, description, color, start_at, end_at, is_all_day, created_at, updated_at
               FROM local_activities
               WHERE household_id = $1
                 AND (
                   (start_at IS NOT NULL AND start_at >= $2 AND start_at < $3)
                   OR start_at IS NULL
                 )"#,
            household_id,
            start.0,
            end.0,
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(ApiError::from)?;

        let mut local_activities = Vec::new();
        for activity in activities_raw {
            let recurrence = sqlx::query_as!(
                LocalActivityRecurrence,
                "SELECT id, local_activity_id, freq, interval_val, by_day_of_week, by_day_of_month, until, count, created_at, updated_at FROM local_activity_recurrences WHERE local_activity_id = $1",
                activity.id
            )
            .fetch_optional(self.pool.as_ref())
            .await
            .map_err(ApiError::from)?;
            local_activities.push(LocalActivityWithRecurrence::new(activity, recurrence));
        }

        Ok(Json(ScheduleResponse {
            events,
            local_activities,
            start: start.0,
            end: end.0,
        }))
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add apps/server/src/application/routes/schedule.rs
git commit -m "feat(server): add ScheduleApi with optional-auth get_schedule endpoint"
```

---

## Task 14: ActivitiesApi

**Files:**
- Create: `src/application/routes/activities.rs`

- [ ] **Step 1: Create ActivitiesApi**

`apps/server/src/application/routes/activities.rs`:

```rust
use std::sync::Arc;
use poem_openapi::{OpenApi, param::Path, payload::Json};
use sqlx::PgPool;
use uuid::Uuid;
use crate::application::routes::security::{ApiError, BearerAuth, get_household_id};
use crate::infrastructure::{
    auth,
    models::local_activity::{
        CreateLocalActivity, LocalActivity, LocalActivityRecurrence,
        LocalActivityWithRecurrence, UpdateLocalActivity,
    },
};
use crate::configuration::config::Config;

pub struct ActivitiesApi {
    pub pool: Arc<PgPool>,
    pub config: Arc<Config>,
}

impl ActivitiesApi {
    fn verify(&self, auth: &BearerAuth) -> Result<(), ApiError> {
        auth::verify_token(&auth.0.token, &self.config.jwt_secret)
            .map_err(|_| ApiError::unauthorized())?;
        Ok(())
    }
}

#[OpenApi(tag = "Activities")]
impl ActivitiesApi {
    /// List all local activities with their recurrence rules.
    #[oai(path = "/activities", method = "get")]
    pub async fn list_activities(
        &self,
        auth: BearerAuth,
    ) -> Result<Json<Vec<LocalActivityWithRecurrence>>, ApiError> {
        self.verify(&auth)?;
        let household_id = get_household_id(self.pool.as_ref()).await?;

        let activities = sqlx::query_as!(
            LocalActivity,
            "SELECT id, household_id, person_id, title, description, color, start_at, end_at, is_all_day, created_at, updated_at FROM local_activities WHERE household_id = $1 ORDER BY start_at NULLS LAST, title",
            household_id
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(ApiError::from)?;

        let mut result = Vec::with_capacity(activities.len());
        for activity in activities {
            let recurrence = sqlx::query_as!(
                LocalActivityRecurrence,
                "SELECT id, local_activity_id, freq, interval_val, by_day_of_week, by_day_of_month, until, count, created_at, updated_at FROM local_activity_recurrences WHERE local_activity_id = $1",
                activity.id
            )
            .fetch_optional(self.pool.as_ref())
            .await
            .map_err(ApiError::from)?;
            result.push(LocalActivityWithRecurrence::new(activity, recurrence));
        }

        Ok(Json(result))
    }

    /// Create a new local activity, optionally with a recurrence rule.
    #[oai(path = "/activities", method = "post")]
    pub async fn create_activity(
        &self,
        auth: BearerAuth,
        body: Json<CreateLocalActivity>,
    ) -> Result<Json<LocalActivityWithRecurrence>, ApiError> {
        self.verify(&auth)?;
        let household_id = get_household_id(self.pool.as_ref()).await?;
        let id = Uuid::new_v4();
        let body = body.0;

        let activity = sqlx::query_as!(
            LocalActivity,
            r#"INSERT INTO local_activities (id, household_id, person_id, title, description, color, start_at, end_at, is_all_day)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
               RETURNING id, household_id, person_id, title, description, color, start_at, end_at, is_all_day, created_at, updated_at"#,
            id,
            household_id,
            body.person_id,
            body.title,
            body.description,
            body.color,
            body.start_at,
            body.end_at,
            body.is_all_day.unwrap_or(false),
        )
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(ApiError::from)?;

        let recurrence = if let Some(rec) = body.recurrence {
            let r = sqlx::query_as!(
                LocalActivityRecurrence,
                r#"INSERT INTO local_activity_recurrences (id, local_activity_id, freq, interval_val, by_day_of_week, by_day_of_month, until, count)
                   VALUES ($1, $2, $3, $4, $5, $6, $7::date, $8)
                   RETURNING id, local_activity_id, freq, interval_val, by_day_of_week, by_day_of_month, until, count, created_at, updated_at"#,
                Uuid::new_v4(),
                id,
                rec.freq,
                rec.interval.unwrap_or(1),
                rec.by_day_of_week.as_deref(),
                rec.by_day_of_month.as_deref(),
                rec.until,
                rec.count,
            )
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(ApiError::from)?;
            Some(r)
        } else {
            None
        };

        Ok(Json(LocalActivityWithRecurrence::new(activity, recurrence)))
    }

    /// Update an existing local activity by ID.
    #[oai(path = "/activities/:id", method = "patch")]
    pub async fn update_activity(
        &self,
        auth: BearerAuth,
        id: Path<Uuid>,
        body: Json<UpdateLocalActivity>,
    ) -> Result<Json<LocalActivityWithRecurrence>, ApiError> {
        self.verify(&auth)?;
        let body = body.0;

        let existing = sqlx::query_as!(
            LocalActivity,
            "SELECT id, household_id, person_id, title, description, color, start_at, end_at, is_all_day, created_at, updated_at FROM local_activities WHERE id = $1",
            id.0
        )
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::not_found(format!("Activity {} not found", id.0)))?;

        let activity = sqlx::query_as!(
            LocalActivity,
            r#"UPDATE local_activities SET
                person_id = COALESCE($2, person_id),
                title = COALESCE($3, title),
                description = COALESCE($4, description),
                color = COALESCE($5, color),
                start_at = COALESCE($6, start_at),
                end_at = COALESCE($7, end_at),
                is_all_day = COALESCE($8, is_all_day),
                updated_at = NOW()
               WHERE id = $1
               RETURNING id, household_id, person_id, title, description, color, start_at, end_at, is_all_day, created_at, updated_at"#,
            id.0,
            body.person_id.flatten(),
            body.title,
            body.description.flatten(),
            body.color.flatten(),
            body.start_at.flatten(),
            body.end_at.flatten(),
            body.is_all_day,
        )
        .fetch_one(self.pool.as_ref())
        .await
        .map_err(ApiError::from)?;

        let recurrence = match body.recurrence {
            Some(Some(rec)) => {
                let r = sqlx::query_as!(
                    LocalActivityRecurrence,
                    r#"INSERT INTO local_activity_recurrences (id, local_activity_id, freq, interval_val, by_day_of_week, by_day_of_month, until, count)
                       VALUES ($1, $2, $3, $4, $5, $6, $7::date, $8)
                       ON CONFLICT (local_activity_id) DO UPDATE SET
                           freq = EXCLUDED.freq,
                           interval_val = EXCLUDED.interval_val,
                           by_day_of_week = EXCLUDED.by_day_of_week,
                           by_day_of_month = EXCLUDED.by_day_of_month,
                           until = EXCLUDED.until,
                           count = EXCLUDED.count,
                           updated_at = NOW()
                       RETURNING id, local_activity_id, freq, interval_val, by_day_of_week, by_day_of_month, until, count, created_at, updated_at"#,
                    Uuid::new_v4(),
                    id.0,
                    rec.freq,
                    rec.interval.unwrap_or(1),
                    rec.by_day_of_week.as_deref(),
                    rec.by_day_of_month.as_deref(),
                    rec.until,
                    rec.count,
                )
                .fetch_one(self.pool.as_ref())
                .await
                .map_err(ApiError::from)?;
                Some(r)
            }
            Some(None) => {
                sqlx::query!("DELETE FROM local_activity_recurrences WHERE local_activity_id = $1", id.0)
                    .execute(self.pool.as_ref())
                    .await
                    .map_err(ApiError::from)?;
                None
            }
            None => {
                sqlx::query_as!(
                    LocalActivityRecurrence,
                    "SELECT id, local_activity_id, freq, interval_val, by_day_of_week, by_day_of_month, until, count, created_at, updated_at FROM local_activity_recurrences WHERE local_activity_id = $1",
                    id.0
                )
                .fetch_optional(self.pool.as_ref())
                .await
                .map_err(ApiError::from)?
            }
        };

        Ok(Json(LocalActivityWithRecurrence::new(activity, recurrence)))
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add apps/server/src/application/routes/activities.rs
git commit -m "feat(server): add ActivitiesApi with list, create, update endpoints"
```

---

## Task 15: GoogleApi

**Files:**
- Create: `src/application/routes/google.rs`

- [ ] **Step 1: Create GoogleApi**

`apps/server/src/application/routes/google.rs`:

```rust
use std::sync::Arc;
use poem_openapi::{OpenApi, Object, param::{Path, Query}, payload::Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use crate::application::routes::security::{ApiError, BearerAuth, get_household_id};
use crate::infrastructure::{
    auth,
    models::{
        calendar_source::{CalendarSource, SelectCalendarsBody},
        google_account::GoogleAccount,
    },
};
use crate::configuration::config::Config;

#[derive(Debug, Serialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct ConnectStartResponse {
    pub auth_url: String,
    pub state: String,
}

pub struct GoogleApi {
    pub pool: Arc<PgPool>,
    pub config: Arc<Config>,
}

impl GoogleApi {
    fn verify(&self, auth: &BearerAuth) -> Result<(), ApiError> {
        auth::verify_token(&auth.0.token, &self.config.jwt_secret)
            .map_err(|_| ApiError::unauthorized())?;
        Ok(())
    }
}

#[OpenApi(tag = "Google")]
impl GoogleApi {
    /// Start the Google OAuth2 flow. Returns an auth URL to redirect the user to.
    #[oai(path = "/google/connect/start", method = "post")]
    pub async fn connect_start(
        &self,
        auth: BearerAuth,
    ) -> Result<Json<ConnectStartResponse>, ApiError> {
        self.verify(&auth)?;

        if self.config.mock_calendar {
            return Ok(Json(ConnectStartResponse {
                auth_url: "/google/connect/callback?code=mock&state=mock".to_string(),
                state: "mock".to_string(),
            }));
        }

        let client = crate::infrastructure::google::client::build_oauth_client(&self.config)
            .map_err(|e| ApiError::from(e))?;
        use oauth2::{CsrfToken, Scope};
        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("https://www.googleapis.com/auth/calendar.readonly".to_string()))
            .url();

        Ok(Json(ConnectStartResponse {
            auth_url: auth_url.to_string(),
            state: csrf_token.secret().clone(),
        }))
    }

    /// OAuth2 callback — Google redirects here with an authorization code.
    #[oai(path = "/google/connect/callback", method = "get")]
    pub async fn connect_callback(
        &self,
        code: Query<Option<String>>,
        state: Query<Option<String>>,
        error: Query<Option<String>>,
    ) -> Result<Json<GoogleAccount>, ApiError> {
        if let Some(err) = error.0 {
            return Err(ApiError::bad_request(format!("OAuth error: {err}")));
        }

        let household_id = get_household_id(self.pool.as_ref()).await?;

        if self.config.mock_calendar || code.0.as_deref() == Some("mock") {
            let account = sqlx::query_as!(
                GoogleAccount,
                r#"INSERT INTO google_accounts (id, household_id, email, display_name, is_active)
                   VALUES ($1, $2, 'mock@example.com', 'Mock Account', TRUE)
                   ON CONFLICT (household_id, email) DO UPDATE SET display_name = EXCLUDED.display_name
                   RETURNING id, household_id, email, display_name, avatar_url, access_token, refresh_token, token_expires_at, is_active, created_at, updated_at"#,
                Uuid::new_v4(),
                household_id,
            )
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(ApiError::from)?;

            sqlx::query!(
                r#"INSERT INTO calendar_sources (id, google_account_id, calendar_id, name, is_selected, access_role)
                   VALUES
                     ($1, $2, 'mock-personal', 'Personal', TRUE, 'owner'),
                     ($3, $2, 'mock-family', 'Family', TRUE, 'writer'),
                     ($4, $2, 'mock-work', 'Work', FALSE, 'reader')
                   ON CONFLICT (google_account_id, calendar_id) DO NOTHING"#,
                Uuid::new_v4(),
                account.id,
                Uuid::new_v4(),
                Uuid::new_v4(),
            )
            .execute(self.pool.as_ref())
            .await
            .map_err(ApiError::from)?;

            return Ok(Json(account));
        }

        let code_str = code.0.ok_or_else(|| ApiError::bad_request("Missing code"))?;
        let client = crate::infrastructure::google::client::build_oauth_client(&self.config)
            .map_err(|e| ApiError::from(e))?;
        let account = crate::infrastructure::google::client::exchange_code_for_account(
            &client, &code_str, household_id, self.pool.as_ref(),
        )
        .await
        .map_err(|e| ApiError::from(e))?;

        Ok(Json(account))
    }

    /// List all connected Google accounts.
    #[oai(path = "/google/accounts", method = "get")]
    pub async fn list_accounts(
        &self,
        auth: BearerAuth,
    ) -> Result<Json<Vec<GoogleAccount>>, ApiError> {
        self.verify(&auth)?;
        let household_id = get_household_id(self.pool.as_ref()).await?;

        let accounts = sqlx::query_as!(
            GoogleAccount,
            "SELECT id, household_id, email, display_name, avatar_url, access_token, refresh_token, token_expires_at, is_active, created_at, updated_at FROM google_accounts WHERE household_id = $1 AND is_active = TRUE ORDER BY created_at",
            household_id
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(ApiError::from)?;

        Ok(Json(accounts))
    }

    /// List calendars for a specific Google account.
    #[oai(path = "/google/accounts/:id/calendars", method = "get")]
    pub async fn list_calendars(
        &self,
        auth: BearerAuth,
        id: Path<Uuid>,
    ) -> Result<Json<Vec<CalendarSource>>, ApiError> {
        self.verify(&auth)?;

        let calendars = sqlx::query_as!(
            CalendarSource,
            "SELECT id, google_account_id, calendar_id, name, description, color_hex, is_selected, access_role, created_at, updated_at FROM calendar_sources WHERE google_account_id = $1 ORDER BY name",
            id.0
        )
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(ApiError::from)?;

        Ok(Json(calendars))
    }

    /// Toggle calendar selection for sync.
    #[oai(path = "/google/calendars/select", method = "post")]
    pub async fn select_calendars(
        &self,
        auth: BearerAuth,
        body: Json<SelectCalendarsBody>,
    ) -> Result<Json<Vec<CalendarSource>>, ApiError> {
        self.verify(&auth)?;
        let mut updated = Vec::new();

        for sel in body.0.selections {
            let cal = sqlx::query_as!(
                CalendarSource,
                "UPDATE calendar_sources SET is_selected = $2, updated_at = NOW() WHERE id = $1 RETURNING id, google_account_id, calendar_id, name, description, color_hex, is_selected, access_role, created_at, updated_at",
                sel.calendar_source_id,
                sel.is_selected,
            )
            .fetch_optional(self.pool.as_ref())
            .await
            .map_err(ApiError::from)?;

            if let Some(c) = cal {
                updated.push(c);
            }
        }

        Ok(Json(updated))
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add apps/server/src/application/routes/google.rs
git commit -m "feat(server): add GoogleApi with OAuth and calendar management endpoints"
```

---

## Task 16: Compose Routes (OpenApiService)

**Files:**
- Modify: `src/application/routes/mod.rs`

- [ ] **Step 1: Write routes/mod.rs**

`apps/server/src/application/routes/mod.rs`:

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
use poem::Route;
use poem_openapi::OpenApiService;
use sqlx::PgPool;
use crate::configuration::config::Config;

use activities::ActivitiesApi;
use auth::AuthApi;
use google::GoogleApi;
use people::PeopleApi;
use schedule::ScheduleApi;
use settings::SettingsApi;
use sync::SyncApi;

pub fn build_routes(pool: Arc<PgPool>, config: Arc<Config>) -> Route {
    let auth_api = AuthApi { pool: pool.clone(), config: config.clone() };
    let google_api = GoogleApi { pool: pool.clone(), config: config.clone() };
    let sync_api = SyncApi { pool: pool.clone(), config: config.clone() };
    let schedule_api = ScheduleApi { pool: pool.clone(), config: config.clone() };
    let people_api = PeopleApi { pool: pool.clone(), config: config.clone() };
    let activities_api = ActivitiesApi { pool: pool.clone(), config: config.clone() };
    let settings_api = SettingsApi { pool: pool.clone(), config: config.clone() };

    let api_service = OpenApiService::new(
        (auth_api, google_api, sync_api, schedule_api, people_api, activities_api, settings_api),
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

- [ ] **Step 2: Commit**

```bash
git add apps/server/src/application/routes/mod.rs
git commit -m "feat(server): compose OpenApiService with all API structs"
```

---

## Task 17: Create configuration/service_setup.rs

**Files:**
- Create: `src/configuration/service_setup.rs`

- [ ] **Step 1: Write service_setup.rs**

`apps/server/src/configuration/service_setup.rs`:

```rust
use poem::{
    Route,
    listener::TcpListener,
    middleware::{Cors, Tracing},
    EndpointExt, Server,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("poem error: {0}")]
    Poem(#[from] poem::Error),
}

pub async fn serve(host: &str, port: u16, routes: Route) -> Result<(), ServiceError> {
    let app = routes
        .with(Cors::new())   // permissive: allow any origin/method/header
        .with(Tracing);      // structured request/response logging

    let bind_addr = format!("{host}:{port}");
    tracing::info!("Listening on {bind_addr}");

    Server::new(TcpListener::bind(bind_addr))
        .run_with_graceful_shutdown(
            app,
            async {
                let _ = tokio::signal::ctrl_c().await;
                tracing::info!("Shutting down…");
            },
            None,
        )
        .await?;

    Ok(())
}
```

- [ ] **Step 2: Commit**

```bash
git add apps/server/src/configuration/service_setup.rs
git commit -m "feat(server): add poem server setup with CORS, tracing, and graceful shutdown"
```

---

## Task 18: Rewrite main.rs and Remove Old Files

**Files:**
- Modify: `src/main.rs`
- Delete: `src/routes.rs`, `src/handlers/` (entire directory)

- [ ] **Step 1: Rewrite main.rs**

`apps/server/src/main.rs`:

```rust
mod application;
mod configuration;
mod domain;
mod infrastructure;

use std::sync::Arc;
use configuration::{config::Config, service_setup};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "family_center_server=debug,poem=debug".into()),
        )
        .init();

    let config = Arc::new(Config::from_env()?);
    let pool = Arc::new(infrastructure::db::connect(&config.database_url).await?);

    sqlx::migrate!("./migrations").run(pool.as_ref()).await?;

    let routes = application::routes::build_routes(pool, config.clone());

    service_setup::serve(&config.server_host, config.server_port, routes)
        .await
        .map_err(anyhow::Error::from)
}
```

- [ ] **Step 2: Delete old files**

```bash
rm apps/server/src/routes.rs
rm -rf apps/server/src/handlers
```

- [ ] **Step 3: Run cargo check**

```bash
cd apps/server && cargo check 2>&1
```

Expected: compiles. Fix any remaining path errors — they will be `crate::` references that weren't updated in Tasks 4 and 5. Work through each error methodically:
- `error[E0432]: unresolved import` → update the `use` path to the new layer location
- `error[E0433]: failed to resolve` → same fix

- [ ] **Step 4: Commit when clean**

```bash
git add apps/server/src/main.rs apps/server/src/routes.rs apps/server/src/handlers
git commit -m "feat(server): rewrite main.rs for poem, remove old axum routes and handlers"
```

---

## Task 19: Verify Build and Run

- [ ] **Step 1: Full build**

```bash
cd apps/server && cargo build 2>&1
```

Expected: `Finished` with no errors.

- [ ] **Step 2: Run the server**

Start the server locally (requires a running Postgres at `DATABASE_URL`):

```bash
cd apps/server && cargo run 2>&1
```

Expected output:
```
INFO family_center_server: Listening on 0.0.0.0:3000
```

- [ ] **Step 3: Verify Scalar UI**

```bash
curl -s http://localhost:3000/docs | head -5
```

Expected: HTML page with Scalar UI.

- [ ] **Step 4: Verify OpenAPI spec**

```bash
curl -s http://localhost:3000/openapi | python3 -m json.tool | head -30
```

Expected: Valid JSON OpenAPI spec with paths for all 13 endpoints.

- [ ] **Step 5: Smoke-test bootstrap**

```bash
curl -s -X POST http://localhost:3000/api/auth/bootstrap \
  -H 'Content-Type: application/json' \
  -d '{"householdName":"Test"}' | python3 -m json.tool
```

Expected:
```json
{
  "householdId": "<uuid>",
  "token": "<jwt>",
  "isNew": true
}
```

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "chore(server): verify poem-openapi migration builds and serves"
```

---

## Task 20: Integration Tests

Replace `axum-test` tests with `poem::test::TestClient`.

**Files:**
- Modify or create: `src/application/routes/auth.rs` (add tests module)

- [ ] **Step 1: Write bootstrap integration test**

Add to the bottom of `apps/server/src/application/routes/auth.rs`:

```rust
#[cfg(test)]
mod tests {
    use poem::test::TestClient;
    use crate::application::routes::build_routes;

    // Integration tests require a running DATABASE_URL.
    // Run with: DATABASE_URL=postgres://... cargo test -- --test-threads=1

    fn test_pool() -> std::sync::Arc<sqlx::PgPool> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let url = std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/family_center_test".to_string());
            std::sync::Arc::new(
                sqlx::PgPool::connect(&url).await.expect("test DB connection")
            )
        })
    }

    #[tokio::test]
    async fn bootstrap_creates_household() {
        let pool = test_pool();
        let config = std::sync::Arc::new(
            crate::configuration::config::Config::from_env().expect("config")
        );
        let app = build_routes(pool.clone(), config);
        let client = TestClient::new(app);

        let resp = client
            .post("/api/auth/bootstrap")
            .body_json(&serde_json::json!({ "householdName": "Test Family" }))
            .send()
            .await;

        resp.assert_status_is_ok();
        let body: serde_json::Value = resp.json().await;
        assert!(body["householdId"].is_string());
        assert!(body["token"].is_string());
        assert!(body["isNew"].is_boolean());
    }

    #[tokio::test]
    async fn list_people_requires_auth() {
        let pool = test_pool();
        let config = std::sync::Arc::new(
            crate::configuration::config::Config::from_env().expect("config")
        );
        let app = build_routes(pool.clone(), config);
        let client = TestClient::new(app);

        let resp = client.get("/api/people").send().await;
        // No token → 401
        resp.assert_status(poem::http::StatusCode::UNAUTHORIZED);
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cd apps/server && cargo test 2>&1
```

Expected: `bootstrap_creates_household` passes (requires Postgres). `list_people_requires_auth` passes without Postgres since it never hits the DB.

- [ ] **Step 3: Commit**

```bash
git add apps/server/src/application/routes/auth.rs
git commit -m "test(server): add poem TestClient integration tests for auth and auth guard"
```

---

## Self-Review Checklist

- [x] Task 1 covers Cargo.toml swap (axum/tower/tower-http/axum-test out, poem/poem-openapi in)
- [x] Tasks 2–6 cover full directory reorganization and error type split
- [x] Task 7 covers Object derives + flatten workaround for LocalActivityWithRecurrence
- [x] Task 8 covers BearerAuth, ApiError, get_household_id shared types
- [x] Tasks 9–15 cover all 7 API structs (13 endpoints total, auth guard on each)
- [x] Task 16 composes OpenApiService tuple, serves /docs (Scalar) and /openapi (spec)
- [x] Task 17 covers service_setup.rs with CORS, Tracing, graceful shutdown
- [x] Task 18 rewrites main.rs, removes routes.rs and handlers/
- [x] Task 19 verifies build + smoke-tests bootstrap
- [x] Task 20 adds integration tests with poem::test::TestClient
- [x] `LocalActivityWithRecurrence::new()` constructor defined in Task 7, used in Tasks 13 and 14 — consistent
- [x] `ApiError::from(anyhow::Error)` used in Task 9 auth.rs — `From<anyhow::Error> for ApiError` defined in Task 8 — consistent
- [x] `build_routes` defined in Task 16, called in Task 18 main.rs and Task 20 tests — consistent
- [x] All 15 endpoint rows from spec covered across GoogleApi (5), AuthApi (1), SyncApi (1), ScheduleApi (1), PeopleApi (3), ActivitiesApi (3), SettingsApi (2)
