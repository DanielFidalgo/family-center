# Axum → poem-openapi Migration Design

**Date:** 2026-04-06
**Status:** Approved

## Overview

Migrate the family-center server from axum 0.7 to poem 3 + poem-openapi 5 with Scalar UI. All 13 existing endpoints are preserved. The HTTP surface is replaced; all business logic (sync, dedupe, recurrence, google client) is untouched. The module structure is reorganized into four explicit layers mirroring the argus reference project.

---

## Architecture

### Layer Boundaries

```
src/
├── main.rs                          # wires deps, calls service_setup
├── configuration/
│   ├── service_setup.rs             # server startup, tracing, signal handling
│   └── config.rs                    # environment variables (moved from src/config.rs)
├── application/
│   └── routes/
│       ├── mod.rs                   # composes OpenApiService + Route
│       ├── auth.rs                  # AuthApi struct + handlers
│       ├── google.rs                # GoogleApi struct + handlers
│       ├── sync.rs                  # SyncApi struct + handlers
│       ├── people.rs                # PeopleApi struct + handlers
│       ├── activities.rs            # ActivitiesApi struct + handlers
│       ├── schedule.rs              # ScheduleApi struct + handlers
│       └── settings.rs              # SettingsApi struct + handlers
├── domain/
│   ├── error.rs                     # DomainError (thiserror)
│   ├── sync/                        # moved from src/sync/
│   ├── dedupe/                      # moved from src/dedupe/
│   └── recurrence/                  # moved from src/recurrence/
└── infrastructure/
    ├── error.rs                     # RepositoryError (thiserror)
    ├── auth.rs                      # JWT create_token / verify_token
    ├── db.rs                        # PgPool setup (unchanged)
    ├── google/                      # moved from src/google/
    └── models/                      # moved from src/models/
```

### What moves, what stays

| Concern | From | To | Changes |
|---------|------|----|---------|
| Routes + handlers | `src/routes.rs`, `src/handlers/` | `src/application/routes/` | Rewritten for poem-openapi |
| Server setup | `src/main.rs` (inline) | `src/configuration/service_setup.rs` | Extracted, mirrors argus |
| Config | `src/config.rs` | `src/configuration/config.rs` | Moved only |
| Error types | `src/error.rs` | `src/domain/error.rs` + `src/infrastructure/error.rs` | Split by layer |
| Business logic | `src/sync/`, `src/dedupe/`, `src/recurrence/` | `src/domain/*/` | Moved only |
| Infrastructure | `src/google/`, `src/models/`, `src/db.rs`, `src/auth.rs` | `src/infrastructure/*/` | Moved only |

---

## Dependencies

Replace in `Cargo.toml`:

**Remove:**
- `axum`
- `tower`
- `tower-http`
- `axum-test`

**Add:**
```toml
poem = { version = "3.1.12", features = ["static-files"] }
poem-openapi = { version = "5.1.16", features = ["scalar"] }
```

**Keep:**
- `tokio` (already present)
- `serde`, `serde_json`, `sqlx`, `uuid`, `chrono`
- `jsonwebtoken`, `oauth2`, `reqwest`
- `tracing`, `tracing-subscriber`
- `anyhow`, `thiserror`, `dotenvy`

---

## API Structs

Each struct holds only the dependencies it needs, injected at construction via `Arc<>`. No global `AppState`.

```rust
pub struct AuthApi     { pool: Arc<PgPool>, config: Arc<Config> }
pub struct GoogleApi   { pool: Arc<PgPool>, config: Arc<Config> }
pub struct SyncApi     { pool: Arc<PgPool>, config: Arc<Config> }
pub struct PeopleApi     { pool: Arc<PgPool> }
pub struct ActivitiesApi { pool: Arc<PgPool> }
pub struct ScheduleApi   { pool: Arc<PgPool> }
pub struct SettingsApi   { pool: Arc<PgPool> }
```

All are passed as a tuple to `OpenApiService::new((auth_api, google_api, sync_api, people_api, activities_api, schedule_api, settings_api))`.

### Handler pattern

```rust
#[OpenApi]
impl PeopleApi {
    #[oai(path = "/people", method = "get")]
    async fn list_people(&self, auth: BearerAuth) -> poem_openapi::payload::Json<Vec<Person>> {
        // self.pool available
    }
}
```

---

## Error Handling

### Domain errors (`src/domain/error.rs`)

Business rule violations:

```rust
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("unauthorized")]
    Unauthorized,
}
```

### Repository errors (`src/infrastructure/error.rs`)

Persistence and external API failures:

```rust
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("external api error: {0}")]
    ExternalApi(String),
}
```

### HTTP mapping (application layer)

Each API struct converts errors to poem-openapi `ApiResponse` variants:

```rust
#[derive(ApiResponse)]
enum ApiError {
    #[oai(status = 400)] BadRequest(Json<ErrorBody>),
    #[oai(status = 401)] Unauthorized(Json<ErrorBody>),
    #[oai(status = 404)] NotFound(Json<ErrorBody>),
    #[oai(status = 500)] Internal(Json<ErrorBody>),
}

impl From<DomainError> for ApiError { ... }
impl From<RepositoryError> for ApiError { ... }
```

Response format matches current:
```json
{ "error": { "code": "ERROR_CODE", "message": "Human readable message" } }
```

---

## Auth — SecurityScheme

Defined in `application/routes/mod.rs` or a shared `application/security.rs`:

```rust
#[derive(SecurityScheme)]
#[oai(ty = "bearer")]
pub struct BearerAuth(Bearer);
```

Usage per endpoint:
- **Protected** (JWT required): `auth: BearerAuth` — rejects if absent
- **JWT-aware public** (token optional): `auth: Option<BearerAuth>` — serves public view if absent
- **Fully public** (`POST /auth/bootstrap`): no auth parameter

JWT verification (`infrastructure/auth.rs`) is called inside handlers using `auth.0.token`.

---

## Server Setup (`configuration/service_setup.rs`)

Mirrors argus pattern:

```rust
pub fn server_setup(config: Config, routes: Route) -> JoinHandle<Result<(), ServiceError>> {
    let app = routes
        .with(Cors::new())      // permissive: allow any origin/method/header
        .with(Tracing);         // request/response logging

    tokio::spawn(async move {
        poem::Server::new(TcpListener::bind(format!("{}:{}", config.host, config.port)))
            .run_with_graceful_shutdown(app, graceful_shutdown(), None)
            .await
    })
}
```

Fixes the current hardcoded port — `host` and `port` now come from `Config`.

---

## Routes (`application/routes/mod.rs`)

```rust
pub fn routes(/* all api structs */) -> Route {
    let api_service = OpenApiService::new(
        (auth_api, google_api, sync_api, people_api, activities_api, schedule_api, settings_api),
        "Family Center API",
        "1.0.0",
    ).server("/api");

    Route::new()
        .nest("/docs",    api_service.scalar())
        .nest("/openapi", api_service.spec_endpoint())
        .nest("/api",     api_service.into_endpoint())
}
```

Scalar UI available at `GET /docs`. Raw spec at `GET /openapi`.

---

## Endpoint Mapping

All 13 existing endpoints preserved:

| Method | Path | API Struct | Auth |
|--------|------|------------|------|
| POST | /auth/bootstrap | AuthApi | public |
| POST | /google/connect/start | GoogleApi | BearerAuth |
| GET | /google/connect/callback | GoogleApi | public (OAuth callback) |
| GET | /google/accounts | GoogleApi | BearerAuth |
| GET | /google/accounts/:id/calendars | GoogleApi | BearerAuth |
| POST | /google/calendars/select | GoogleApi | BearerAuth |
| POST | /sync/run | SyncApi | BearerAuth |
| GET | /schedule | ScheduleApi | Option\<BearerAuth\> |
| GET | /people | PeopleApi | BearerAuth |
| POST | /people | PeopleApi | BearerAuth |
| PATCH | /people/:id | PeopleApi | BearerAuth |
| GET | /activities | ActivitiesApi | BearerAuth |
| POST | /activities | ActivitiesApi | BearerAuth |
| PATCH | /activities/:id | ActivitiesApi | BearerAuth |
| GET | /settings | SettingsApi | BearerAuth |
| PATCH | /settings | SettingsApi | BearerAuth |

---

## Models

All types in `infrastructure/models/` gain `#[derive(poem_openapi::Object)]` alongside existing `serde` derives. This generates OpenAPI schemas automatically. No structural changes to the model structs.

---

## Testing

`axum-test` is removed. Integration tests use `poem::test::TestClient`:

```rust
let client = TestClient::new(routes());
let resp = client.get("/api/people").bearer_auth(token).send().await;
```

Existing test logic is preserved; only the test harness changes.

---

## Out of Scope

- No new endpoints
- No changes to sync, dedupe, recurrence, or Google OAuth logic
- No multi-tenant support
- No token refresh implementation
- No frontend changes
