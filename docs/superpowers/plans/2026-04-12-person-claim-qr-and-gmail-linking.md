# Person Claim QR Code & Gmail Linking Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable family members to self-service their profile (photo, name, Google account) by scanning a QR code, and fix Gmail linking visibility in the person creation flow.

**Architecture:** Admin generates a short-lived claim token per person, rendered as QR code. Family member scans it, lands on a standalone HTML page served by the Rust server. The page calls unauthenticated API endpoints scoped to the claim token. Photos upload to S3-compatible storage (Leapcell). Gmail linking is also surfaced in the person creation modal.

**Tech Stack:** Rust/Poem (server), `aws-sdk-s3` + `aws-config` crates (Leapcell S3), vanilla HTML/JS (claim page), `qrcode.react` (QR rendering in mobile app)

**Spec:** `docs/superpowers/specs/2026-04-12-person-claim-qr-and-gmail-linking-design.md`

---

### Task 1: Database migration for claim tokens

**Files:**
- Create: `apps/server/migrations/20260412100000_person_claim_tokens.up.sql`
- Create: `apps/server/migrations/20260412100000_person_claim_tokens.down.sql`

- [ ] **Step 1: Write the up migration**

```sql
-- apps/server/migrations/20260412100000_person_claim_tokens.up.sql
CREATE TABLE IF NOT EXISTS family_center.person_claim_tokens (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id  UUID NOT NULL REFERENCES family_center.people(id) ON DELETE CASCADE UNIQUE,
    token      TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_claim_tokens_token
    ON family_center.person_claim_tokens(token);
```

- [ ] **Step 2: Write the down migration**

```sql
-- apps/server/migrations/20260412100000_person_claim_tokens.down.sql
DROP TABLE IF EXISTS family_center.person_claim_tokens;
```

- [ ] **Step 3: Commit**

```bash
git add apps/server/migrations/20260412100000_person_claim_tokens.*
git commit -m "feat(server): add person_claim_tokens migration"
```

---

### Task 2: Add S3 and hex crate dependencies + config

**Files:**
- Modify: `apps/server/Cargo.toml`
- Modify: `apps/server/src/configuration/config.rs`

- [ ] **Step 1: Add `aws-sdk-s3`, `aws-config`, `hex`, and `rand` to Cargo.toml**

Add after the `url = "2"` line in `apps/server/Cargo.toml`:

```toml
# S3-compatible storage (Leapcell)
aws-sdk-s3 = "1"
aws-config = "1"
aws-types = "1"

# Hex encoding for token generation
hex = "0.4"

# Random bytes
rand = "0.8"
```

- [ ] **Step 2: Add S3 fields to IAppConfig trait**

In `apps/server/src/configuration/config.rs`, add to the `IAppConfig` trait (after `mock_calendar`):

```rust
    fn s3_endpoint(&self) -> &str;
    fn s3_bucket(&self) -> &str;
    fn s3_access_key(&self) -> &str;
    fn s3_secret_key(&self) -> &str;
    fn s3_region(&self) -> &str;
    fn public_url(&self) -> &str;
```

- [ ] **Step 3: Add S3 fields to Config struct**

In `apps/server/src/configuration/config.rs`, add to the `Config` struct:

```rust
    pub s3_endpoint: String,
    pub s3_bucket: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub s3_region: String,
    pub public_url: String,
```

- [ ] **Step 4: Load S3 env vars in `Config::from_env()`**

Add after `mock_calendar` in `from_env()`:

```rust
            s3_endpoint: std::env::var("S3_ENDPOINT").unwrap_or_default(),
            s3_bucket: std::env::var("S3_BUCKET").unwrap_or_else(|_| "family-center".to_string()),
            s3_access_key: std::env::var("S3_ACCESS_KEY").unwrap_or_default(),
            s3_secret_key: std::env::var("S3_SECRET_KEY").unwrap_or_default(),
            s3_region: std::env::var("S3_REGION").unwrap_or_else(|_| "auto".to_string()),
            public_url: std::env::var("PUBLIC_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()),
```

- [ ] **Step 5: Implement IAppConfig for new fields**

Add to the `impl IAppConfig for Config` block:

```rust
    fn s3_endpoint(&self) -> &str { &self.s3_endpoint }
    fn s3_bucket(&self) -> &str { &self.s3_bucket }
    fn s3_access_key(&self) -> &str { &self.s3_access_key }
    fn s3_secret_key(&self) -> &str { &self.s3_secret_key }
    fn s3_region(&self) -> &str { &self.s3_region }
    fn public_url(&self) -> &str { &self.public_url }
```

- [ ] **Step 6: Verify it compiles**

Run: `cargo check -p family-center-server 2>&1 | grep "^error"`
Expected: no errors

- [ ] **Step 7: Commit**

```bash
git add apps/server/Cargo.toml apps/server/src/configuration/config.rs
git commit -m "feat(server): add S3, hex, rand deps and config fields"
```

---

### Task 3: S3 upload module

**Files:**
- Create: `apps/server/src/infrastructure/s3.rs`
- Modify: `apps/server/src/infrastructure/mod.rs`

- [ ] **Step 1: Create the S3 upload module**

```rust
// apps/server/src/infrastructure/s3.rs
use anyhow::Result;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::Region;
use aws_sdk_s3::primitives::ByteStream;
use aws_types::credentials::Credentials;

use crate::configuration::config::IAppConfig;

async fn build_client(config: &dyn IAppConfig) -> Client {
    let creds = Credentials::new(
        config.s3_access_key(),
        config.s3_secret_key(),
        None,
        None,
        "leapcell",
    );

    let sdk_config = aws_config::from_env()
        .region(Region::new(config.s3_region().to_string()))
        .credentials_provider(creds)
        .endpoint_url(config.s3_endpoint())
        .load()
        .await;

    Client::new(&sdk_config)
}

pub async fn upload_avatar(
    config: &dyn IAppConfig,
    person_id: &str,
    content_type: &str,
    data: &[u8],
) -> Result<String> {
    let client = build_client(config).await;
    let key = format!("avatars/{person_id}.jpg");

    client
        .put_object()
        .bucket(config.s3_bucket())
        .key(&key)
        .content_type(content_type)
        .body(ByteStream::from(data.to_vec()))
        .send()
        .await?;

    // Return the public URL
    let url = format!("{}/{}/{}", config.s3_endpoint(), config.s3_bucket(), key);
    Ok(url)
}
```

- [ ] **Step 2: Register the module**

Add to `apps/server/src/infrastructure/mod.rs`:

```rust
pub mod s3;
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p family-center-server 2>&1 | grep "^error"`
Expected: no errors

- [ ] **Step 4: Commit**

```bash
git add apps/server/src/infrastructure/s3.rs apps/server/src/infrastructure/mod.rs
git commit -m "feat(server): add S3 avatar upload module"
```

---

### Task 4: Claim token entity and repository

**Files:**
- Create: `apps/server/src/domain/entities/claim_token.rs`
- Modify: `apps/server/src/domain/entities/mod.rs`
- Create: `apps/server/src/domain/repositories/claim_token_repository.rs`
- Modify: `apps/server/src/domain/repositories/mod.rs`
- Create: `apps/server/src/infrastructure/repositories/claim_token_repository.rs`
- Modify: `apps/server/src/infrastructure/repositories/mod.rs`

- [ ] **Step 1: Create the entity**

```rust
// apps/server/src/domain/entities/claim_token.rs
use chrono::{DateTime, Utc};
use poem_openapi::Object;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct ClaimToken {
    pub id: Uuid,
    pub person_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct ClaimTokenResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub claim_url: String,
}
```

- [ ] **Step 2: Register entity module**

Add to `apps/server/src/domain/entities/mod.rs`:

```rust
pub mod claim_token;
```

- [ ] **Step 3: Create the repository trait**

```rust
// apps/server/src/domain/repositories/claim_token_repository.rs
use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::claim_token::ClaimToken;

#[async_trait]
pub trait IClaimTokenRepository: Send + Sync {
    /// Generate a new claim token for a person. Replaces any existing token.
    async fn generate(&self, person_id: Uuid) -> anyhow::Result<ClaimToken>;

    /// Find a valid (non-expired) token by its string value.
    async fn find_valid(&self, token: &str) -> anyhow::Result<Option<ClaimToken>>;

    /// Delete expired tokens (cleanup).
    async fn delete_expired(&self) -> anyhow::Result<u64>;
}
```

- [ ] **Step 4: Register repository trait module**

Add to `apps/server/src/domain/repositories/mod.rs`:

```rust
pub mod claim_token_repository;
```

- [ ] **Step 5: Create the repository implementation**

```rust
// apps/server/src/infrastructure/repositories/claim_token_repository.rs
use async_trait::async_trait;
use chrono::{Duration, Utc};
use rand::RngCore;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::entities::claim_token::ClaimToken;
use crate::domain::repositories::claim_token_repository::IClaimTokenRepository;

pub struct ClaimTokenRepository {
    pool: PgPool,
}

impl ClaimTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl IClaimTokenRepository for ClaimTokenRepository {
    async fn generate(&self, person_id: Uuid) -> anyhow::Result<ClaimToken> {
        let mut bytes = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut bytes);
        let token = hex::encode(bytes);
        let expires_at = Utc::now() + Duration::minutes(30);

        let row = sqlx::query(
            r#"INSERT INTO family_center.person_claim_tokens (person_id, token, expires_at)
               VALUES ($1, $2, $3)
               ON CONFLICT (person_id) DO UPDATE SET
                   token = EXCLUDED.token,
                   expires_at = EXCLUDED.expires_at,
                   created_at = NOW()
               RETURNING id, person_id, token, expires_at, created_at"#,
        )
        .bind(person_id)
        .bind(&token)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(ClaimToken {
            id: row.get("id"),
            person_id: row.get("person_id"),
            token: row.get("token"),
            expires_at: row.get("expires_at"),
            created_at: row.get("created_at"),
        })
    }

    async fn find_valid(&self, token: &str) -> anyhow::Result<Option<ClaimToken>> {
        let row = sqlx::query(
            r#"SELECT id, person_id, token, expires_at, created_at
               FROM family_center.person_claim_tokens
               WHERE token = $1 AND expires_at > NOW()"#,
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| ClaimToken {
            id: r.get("id"),
            person_id: r.get("person_id"),
            token: r.get("token"),
            expires_at: r.get("expires_at"),
            created_at: r.get("created_at"),
        }))
    }

    async fn delete_expired(&self) -> anyhow::Result<u64> {
        let result = sqlx::query(
            "DELETE FROM family_center.person_claim_tokens WHERE expires_at <= NOW()",
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}
```

- [ ] **Step 6: Register infra repository module**

Add to `apps/server/src/infrastructure/repositories/mod.rs`:

```rust
pub mod claim_token_repository;
```

- [ ] **Step 7: Verify it compiles**

Run: `cargo check -p family-center-server 2>&1 | grep "^error"`
Expected: no errors

- [ ] **Step 8: Commit**

```bash
git add apps/server/src/domain/entities/claim_token.rs \
       apps/server/src/domain/entities/mod.rs \
       apps/server/src/domain/repositories/claim_token_repository.rs \
       apps/server/src/domain/repositories/mod.rs \
       apps/server/src/infrastructure/repositories/claim_token_repository.rs \
       apps/server/src/infrastructure/repositories/mod.rs
git commit -m "feat(server): add ClaimToken entity and repository"
```

---

### Task 5: Wire claim token repository into AppContext

**Files:**
- Modify: `apps/server/src/configuration/app_context.rs`
- Modify: `apps/server/src/configuration/bootstrapper.rs`

- [ ] **Step 1: Add to IAppContext trait**

In `apps/server/src/configuration/app_context.rs`, add the import:

```rust
use crate::domain::repositories::claim_token_repository::IClaimTokenRepository;
```

Add to the `IAppContext` trait:

```rust
    fn claim_token_repository(&self) -> &dyn IClaimTokenRepository;
```

- [ ] **Step 2: Add to AppContext struct**

Add the field:

```rust
    pub claim_token_repository: Arc<dyn IClaimTokenRepository>,
```

- [ ] **Step 3: Add to IAppContext impl**

```rust
    fn claim_token_repository(&self) -> &dyn IClaimTokenRepository { self.claim_token_repository.as_ref() }
```

- [ ] **Step 4: Construct in bootstrapper**

In `apps/server/src/configuration/bootstrapper.rs`, add the import:

```rust
use crate::infrastructure::repositories::claim_token_repository::ClaimTokenRepository;
```

After `let lane_rule_repo = ...` (line 53), add:

```rust
    let claim_token_repo = Arc::new(ClaimTokenRepository::new(pool.clone()));
```

Add to the `AppContext` construction (after `lane_rule_repository: lane_rule_repo,`):

```rust
        claim_token_repository: claim_token_repo,
```

- [ ] **Step 5: Verify it compiles**

Run: `cargo check -p family-center-server 2>&1 | grep "^error"`
Expected: no errors

- [ ] **Step 6: Commit**

```bash
git add apps/server/src/configuration/app_context.rs apps/server/src/configuration/bootstrapper.rs
git commit -m "feat(server): wire ClaimTokenRepository into AppContext"
```

---

### Task 6: Claim API routes — token generation + profile CRUD + avatar upload

**Files:**
- Create: `apps/server/src/application/routes/claim.rs`
- Modify: `apps/server/src/application/routes/mod.rs`
- Modify: `apps/server/src/application/routes/security.rs`
- Modify: `apps/server/src/application/routes/people.rs` (add claim-token endpoint)

- [ ] **Step 1: Add ApiTags::Claim variant**

In `apps/server/src/application/routes/security.rs`, add to the `ApiTags` enum:

```rust
    /// Person claim (self-service profile setup)
    Claim,
```

Also add a `Gone` variant to `ApiError`:

```rust
    /// 410 Gone
    #[oai(status = 410)]
    Gone(Json<ErrorBody>),
```

And a constructor:

```rust
    pub fn gone(msg: impl Into<String>) -> Self {
        ApiError::Gone(Json(ErrorBody {
            code: "GONE".to_string(),
            message: msg.into(),
        }))
    }
```

- [ ] **Step 2: Add claim token generation to PeopleApi**

In `apps/server/src/application/routes/people.rs`, add this endpoint to the `#[OpenApi]` impl block:

```rust
    /// Generate a claim token for a person. Returns token + claim URL.
    #[oai(path = "/people/:id/claim-token", method = "post")]
    pub async fn create_claim_token(
        &self,
        auth: BearerAuth,
        id: Path<Uuid>,
    ) -> Result<Json<crate::domain::entities::claim_token::ClaimTokenResponse>, ApiError> {
        self.verify(&auth)?;

        // Verify person exists
        let _person = self.context.person_repository().find_by_id(id.0)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::not_found(format!("Person {} not found", id.0)))?;

        let claim = self.context.claim_token_repository().generate(id.0)
            .await
            .map_err(ApiError::from)?;

        let claim_url = format!("{}/claim/{}", self.context.config().public_url(), claim.token);

        Ok(Json(crate::domain::entities::claim_token::ClaimTokenResponse {
            token: claim.token,
            expires_at: claim.expires_at,
            claim_url,
        }))
    }
```

Check that `people.rs` imports `Path` from `poem_openapi::param`. It likely already does — verify.

- [ ] **Step 3: Create the claim routes file**

```rust
// apps/server/src/application/routes/claim.rs
use std::sync::Arc;
use poem::web::{Multipart, Path as PoemPath};
use poem::{handler, IntoResponse, Response};
use poem::http::StatusCode;
use poem_openapi::{OpenApi, Object, param::Path, payload::Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::routes::security::{ApiError, ApiTags};
use crate::configuration::app_context::IAppContext;
use crate::domain::entities::person::Person;

#[derive(Debug, Serialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct ClaimProfileResponse {
    pub person: Person,
    pub linked_google_emails: Vec<String>,
}

#[derive(Debug, Deserialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct UpdateClaimProfileRequest {
    pub name: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct AvatarUploadResponse {
    pub avatar_url: String,
}

#[derive(Debug, Serialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct ClaimGoogleStartResponse {
    pub auth_url: String,
}

pub struct ClaimApi {
    pub context: Arc<dyn IAppContext>,
}

impl ClaimApi {
    async fn validate_token(&self, token: &str) -> Result<crate::domain::entities::claim_token::ClaimToken, ApiError> {
        self.context.claim_token_repository()
            .find_valid(token)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::gone("This link has expired. Ask for a new QR code."))
    }
}

#[OpenApi(tag = "ApiTags::Claim")]
impl ClaimApi {
    /// Get the person profile for a claim token.
    #[oai(path = "/claim/:token", method = "get")]
    pub async fn get_profile(
        &self,
        token: Path<String>,
    ) -> Result<Json<ClaimProfileResponse>, ApiError> {
        let claim = self.validate_token(&token.0).await?;

        let person = self.context.person_repository()
            .find_by_id(claim.person_id)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::not_found("Person not found"))?;

        // Get linked google account emails via lane rules
        let household = self.context.household_repository().find_first()
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::internal("No household"))?;

        let rules = self.context.lane_rule_repository()
            .find_by_household(household.id)
            .await
            .map_err(ApiError::from)?;

        let person_rules = rules.iter().filter(|r| r.person_id.as_ref() == Some(&person.id));
        let accounts = self.context.google_account_repository()
            .find_by_household(household.id)
            .await
            .map_err(ApiError::from)?;

        // Simplified: if there are any lane rules for this person, list the household's google emails
        let linked_emails: Vec<String> = if person_rules.count() > 0 {
            accounts.iter().map(|a| a.email.clone()).collect()
        } else {
            vec![]
        };

        Ok(Json(ClaimProfileResponse {
            person,
            linked_google_emails: linked_emails,
        }))
    }

    /// Update person profile via claim token.
    #[oai(path = "/claim/:token", method = "patch")]
    pub async fn update_profile(
        &self,
        token: Path<String>,
        body: Json<UpdateClaimProfileRequest>,
    ) -> Result<Json<Person>, ApiError> {
        let claim = self.validate_token(&token.0).await?;

        let mut person = self.context.person_repository()
            .find_by_id(claim.person_id)
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::not_found("Person not found"))?;

        // Build update
        let update = crate::domain::entities::person::UpdatePersonRequest {
            name: body.0.name,
            color: body.0.color,
            avatar_url: None,
            sort_order: None,
        };

        let updated = self.context.person_repository()
            .update(claim.person_id, update)
            .await
            .map_err(ApiError::from)?;

        Ok(Json(updated))
    }

    /// Upload avatar via claim token. Accepts multipart form with an `image` field.
    #[oai(path = "/claim/:token/avatar", method = "post")]
    pub async fn upload_avatar(
        &self,
        token: Path<String>,
        upload: poem_openapi::payload::Binary<Vec<u8>>,
    ) -> Result<Json<AvatarUploadResponse>, ApiError> {
        let claim = self.validate_token(&token.0).await?;

        let data = upload.0;
        if data.len() > 5 * 1024 * 1024 {
            return Err(ApiError::bad_request("Image too large (max 5MB)"));
        }

        let content_type = "image/jpeg";
        let url = crate::infrastructure::s3::upload_avatar(
            self.context.config(),
            &claim.person_id.to_string(),
            content_type,
            &data,
        )
        .await
        .map_err(|e| ApiError::internal(format!("S3 upload failed: {e}")))?;

        // Update person's avatar_url
        let update = crate::domain::entities::person::UpdatePersonRequest {
            name: None,
            color: None,
            avatar_url: Some(url.clone()),
            sort_order: None,
        };
        self.context.person_repository()
            .update(claim.person_id, update)
            .await
            .map_err(ApiError::from)?;

        Ok(Json(AvatarUploadResponse { avatar_url: url }))
    }

    /// Start Google OAuth flow from claim page.
    #[oai(path = "/claim/:token/google/start", method = "post")]
    pub async fn google_start(
        &self,
        token: Path<String>,
    ) -> Result<Json<ClaimGoogleStartResponse>, ApiError> {
        let claim = self.validate_token(&token.0).await?;

        if self.context.config().mock_calendar() {
            return Ok(Json(ClaimGoogleStartResponse {
                auth_url: format!(
                    "{}/api/claim/google/callback?code=mock&state={}",
                    self.context.config().public_url(),
                    claim.token
                ),
            }));
        }

        let client = crate::infrastructure::google::client::build_oauth_client_from_config(
            self.context.config(),
        )
        .map_err(|e| ApiError::from(anyhow::Error::from(e)))?;

        // Override redirect URI for claim flow
        let claim_redirect = format!("{}/api/claim/google/callback", self.context.config().public_url());
        let client = client.set_redirect_uri(
            oauth2::RedirectUrl::new(claim_redirect).map_err(|e| ApiError::internal(e.to_string()))?,
        );

        use oauth2::{CsrfToken, Scope};
        // Encode claim token in the CSRF state so callback can look up the person
        let (auth_url, _) = client
            .authorize_url(|| CsrfToken::new(claim.token.clone()))
            .add_scope(Scope::new("https://www.googleapis.com/auth/calendar.readonly".to_string()))
            .url();

        Ok(Json(ClaimGoogleStartResponse {
            auth_url: auth_url.to_string(),
        }))
    }

    /// Google OAuth callback for claim flow.
    #[oai(path = "/claim/google/callback", method = "get")]
    pub async fn google_callback(
        &self,
        code: poem_openapi::param::Query<Option<String>>,
        state: poem_openapi::param::Query<Option<String>>,
        error: poem_openapi::param::Query<Option<String>>,
    ) -> Result<poem_openapi::payload::PlainText<String>, ApiError> {
        if let Some(err) = error.0 {
            return Err(ApiError::bad_request(format!("OAuth error: {err}")));
        }

        let claim_token = state.0.ok_or_else(|| ApiError::bad_request("Missing state"))?;
        let claim = self.validate_token(&claim_token).await?;

        let household = self.context.household_repository().find_first()
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::internal("No household"))?;

        let code_str = code.0.ok_or_else(|| ApiError::bad_request("Missing code"))?;

        let account = if self.context.config().mock_calendar() || code_str == "mock" {
            let acct = self.context.google_account_repository()
                .upsert_mock(household.id)
                .await
                .map_err(ApiError::from)?;
            self.context.calendar_source_repository()
                .create_mock_calendars(acct.id)
                .await
                .map_err(ApiError::from)?;
            acct
        } else {
            crate::infrastructure::google::client::exchange_code_and_persist(
                self.context.config(),
                &code_str,
                household.id,
                self.context.google_account_repository(),
            )
            .await
            .map_err(|e| ApiError::from(anyhow::Error::from(e)))?
        };

        // Auto-link: create lane rules for this person's calendars
        let calendars = self.context.calendar_source_repository()
            .find_by_account(account.id)
            .await
            .map_err(ApiError::from)?;

        for cal in &calendars {
            // Create a lane rule linking this calendar to the person
            let _ = self.context.lane_rule_repository()
                .create(
                    household.id,
                    Some(cal.id),
                    None, // no email pattern
                    Some(claim.person_id),
                    "person",
                    100,
                )
                .await;
        }

        // Redirect back to claim page with success flag
        let redirect_url = format!(
            "{}/claim/{}?google=success",
            self.context.config().public_url(),
            claim_token
        );

        Ok(poem_openapi::payload::PlainText(format!(
            "<html><head><meta http-equiv='refresh' content='0;url={redirect_url}' /></head><body>Redirecting...</body></html>"
        )))
    }
}
```

- [ ] **Step 4: Register claim routes module**

Add to `apps/server/src/application/routes/mod.rs`, in the module declarations:

```rust
pub mod claim;
```

Add the import:

```rust
use claim::ClaimApi;
```

In `build_routes()`, after `let lanes_api = ...`:

```rust
    let claim_api = ClaimApi {
        context: context.clone(),
    };
```

Update the `OpenApiService::new(...)` tuple to include `claim_api`:

```rust
    let api_service = OpenApiService::new(
        (
            auth_api,
            google_api,
            sync_api,
            schedule_api,
            people_api,
            activities_api,
            settings_api,
            lanes_api,
            claim_api,
        ),
        "Family Center API",
        "1.0.0",
    )
    .server("/api");
```

- [ ] **Step 5: Verify it compiles**

Run: `cargo check -p family-center-server 2>&1 | grep "^error"`
Expected: no errors. If there are import issues (e.g. `Person` not being public, `UpdatePersonRequest` missing fields), fix them. The `Person` entity and `UpdatePersonRequest` should already exist — check `apps/server/src/domain/entities/person.rs` for the exact field names and adjust accordingly.

- [ ] **Step 6: Commit**

```bash
git add apps/server/src/application/routes/claim.rs \
       apps/server/src/application/routes/mod.rs \
       apps/server/src/application/routes/security.rs \
       apps/server/src/application/routes/people.rs
git commit -m "feat(server): add claim API routes (profile, avatar, Google OAuth)"
```

---

### Task 7: Serve standalone claim HTML page

**Files:**
- Modify: `apps/server/src/application/routes/mod.rs`

The claim page is a standalone HTML file served at `GET /claim/:token`. This is outside the OpenAPI routes — it's a plain Poem handler that returns HTML.

- [ ] **Step 1: Add the claim page handler**

In `apps/server/src/application/routes/mod.rs`, add a new handler function:

```rust
#[handler]
fn claim_page(req: &poem::Request) -> poem::Response {
    let token = req.uri().path().trim_start_matches("/claim/").split('?').next().unwrap_or("");
    if token.is_empty() {
        return poem::Response::builder()
            .status(poem::http::StatusCode::NOT_FOUND)
            .body("Not found");
    }

    let html = CLAIM_HTML.replace("{{TOKEN}}", token);
    poem::Response::builder()
        .header("Content-Type", "text/html; charset=utf-8")
        .body(html)
}

const CLAIM_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
<title>Set Up Your Profile</title>
<style>
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:#0f0f0f;color:#e8e8e8;min-height:100dvh;padding:24px 16px}
.card{background:#1a1a1a;border:1px solid #2a2a2a;border-radius:16px;padding:24px;margin-bottom:16px}
h1{font-size:22px;font-weight:700;margin-bottom:4px}
.sub{font-size:13px;color:#888;margin-bottom:24px}
label{display:block;font-size:11px;font-weight:600;text-transform:uppercase;letter-spacing:.06em;color:#888;margin-bottom:8px}
input[type=text]{width:100%;background:#111;border:1px solid #333;border-radius:10px;padding:13px 14px;color:#e8e8e8;font-size:16px;outline:none}
input[type=text]:focus{border-color:#5E81F4}
.colors{display:flex;flex-wrap:wrap;gap:10px;margin-top:4px}
.color-btn{width:40px;height:40px;border-radius:10px;border:3px solid transparent;cursor:pointer;transition:transform .15s}
.color-btn.active{transform:scale(1.15);border-color:#fff}
.avatar-section{text-align:center;margin:16px 0}
.avatar-preview{width:80px;height:80px;border-radius:50%;object-fit:cover;margin:0 auto 12px;display:block;background:#333}
.avatar-initials{width:80px;height:80px;border-radius:50%;display:flex;align-items:center;justify-content:center;font-size:28px;font-weight:700;color:#fff;margin:0 auto 12px}
.btn-row{display:flex;gap:8px;justify-content:center}
.btn{padding:10px 18px;border-radius:10px;border:1.5px solid #333;background:transparent;color:#e8e8e8;font-size:13px;font-weight:600;cursor:pointer}
.btn:active{opacity:.7}
.btn-primary{background:#5E81F4;border-color:#5E81F4;color:#fff}
.btn-green{background:#22c55e;border-color:#22c55e;color:#fff}
.btn-full{width:100%;padding:14px;font-size:15px;font-weight:700;margin-top:8px}
.google-linked{display:flex;align-items:center;gap:8px;padding:10px 14px;background:#1e3a1e;border:1px solid #22c55e40;border-radius:10px;font-size:13px;color:#4ade80}
.google-linked svg{flex-shrink:0}
.expired{text-align:center;padding:60px 20px;color:#888}
.expired h2{font-size:20px;color:#e8e8e8;margin-bottom:8px}
.saving{opacity:.5;pointer-events:none}
.toast{position:fixed;bottom:24px;left:50%;transform:translateX(-50%);background:#22c55e;color:#fff;padding:10px 20px;border-radius:10px;font-size:13px;font-weight:600;z-index:99;opacity:0;transition:opacity .3s}
.toast.show{opacity:1}
input[type=file]{display:none}
</style>
</head>
<body>

<div id="app"></div>
<div class="toast" id="toast"></div>

<script>
const TOKEN = '{{TOKEN}}';
const API = window.location.origin + '/api';
const COLORS = ['#5E81F4','#FF6B6B','#4FCB8A','#F5A623','#A78BFA','#06B6D4','#F472B6','#FB923C','#8B78FF','#34D399','#FBBF24','#60A5FA'];

let state = { person: null, linkedEmails: [], saving: false };

function $(id) { return document.getElementById(id); }
function toast(msg) {
  const t = $('toast');
  t.textContent = msg;
  t.classList.add('show');
  setTimeout(() => t.classList.remove('show'), 2000);
}

function initials(name) {
  return (name || '?').split(' ').map(w => w[0]).join('').toUpperCase().slice(0,2);
}

function render() {
  const app = $('app');
  if (!state.person) {
    app.innerHTML = '<div class="expired"><h2>Link expired</h2><p>This link has expired. Ask for a new QR code from the family display.</p></div>';
    return;
  }
  const p = state.person;
  const hasAvatar = p.avatarUrl && !p.avatarUrl.includes('undefined');
  const avatarHtml = hasAvatar
    ? `<img class="avatar-preview" src="${p.avatarUrl}" alt="avatar" />`
    : `<div class="avatar-initials" style="background:${p.color}">${initials(p.name)}</div>`;

  const googleHtml = state.linkedEmails.length > 0
    ? state.linkedEmails.map(e => `<div class="google-linked"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>${e}</div>`).join('')
    : `<button class="btn btn-full" onclick="startGoogle()">Connect Google Account</button>`;

  app.innerHTML = `
    <h1>Set up your profile</h1>
    <p class="sub">Scan complete! Customize your profile below.</p>

    <div class="card">
      <div class="avatar-section">
        ${avatarHtml}
        <div class="btn-row">
          <button class="btn" onclick="pickPhoto('camera')">Take photo</button>
          <button class="btn" onclick="pickPhoto('gallery')">Choose photo</button>
        </div>
        <input type="file" id="camera-input" accept="image/*" capture="environment" onchange="uploadPhoto(this)" />
        <input type="file" id="gallery-input" accept="image/*" onchange="uploadPhoto(this)" />
      </div>
    </div>

    <div class="card">
      <label>Name</label>
      <input type="text" id="name-input" value="${p.name}" />
    </div>

    <div class="card">
      <label>Lane color</label>
      <div class="colors">
        ${COLORS.map(c => `<button class="color-btn${p.color===c?' active':''}" style="background:${c}" onclick="pickColor('${c}')"></button>`).join('')}
      </div>
    </div>

    <div class="card">
      <label>Google account</label>
      ${googleHtml}
    </div>

    <button class="btn btn-primary btn-full${state.saving?' saving':''}" onclick="saveProfile()">
      ${state.saving ? 'Saving...' : 'Save changes'}
    </button>
  `;
}

async function load() {
  try {
    const res = await fetch(API + '/claim/' + TOKEN);
    if (res.status === 410 || res.status === 404) {
      state.person = null;
      render();
      return;
    }
    const data = await res.json();
    state.person = data.person;
    state.linkedEmails = data.linkedGoogleEmails || [];
    render();
  } catch (e) {
    state.person = null;
    render();
  }
}

function pickPhoto(mode) {
  if (mode === 'camera') $('camera-input').click();
  else $('gallery-input').click();
}

async function uploadPhoto(input) {
  const file = input.files[0];
  if (!file) return;

  // Resize client-side
  const resized = await resizeImage(file, 512);
  state.saving = true;
  render();

  try {
    const res = await fetch(API + '/claim/' + TOKEN + '/avatar', {
      method: 'POST',
      headers: { 'Content-Type': 'application/octet-stream' },
      body: resized,
    });
    const data = await res.json();
    state.person.avatarUrl = data.avatarUrl;
    toast('Photo updated!');
  } catch (e) {
    toast('Upload failed');
  }
  state.saving = false;
  input.value = '';
  render();
}

function resizeImage(file, maxSize) {
  return new Promise((resolve) => {
    const img = new Image();
    img.onload = () => {
      const canvas = document.createElement('canvas');
      let w = img.width, h = img.height;
      if (w > h) { if (w > maxSize) { h = h * maxSize / w; w = maxSize; } }
      else { if (h > maxSize) { w = w * maxSize / h; h = maxSize; } }
      canvas.width = w;
      canvas.height = h;
      canvas.getContext('2d').drawImage(img, 0, 0, w, h);
      canvas.toBlob(resolve, 'image/jpeg', 0.85);
    };
    img.src = URL.createObjectURL(file);
  });
}

function pickColor(c) {
  state.person.color = c;
  render();
}

async function saveProfile() {
  const name = $('name-input')?.value?.trim();
  if (!name) return;
  state.saving = true;
  render();

  try {
    const res = await fetch(API + '/claim/' + TOKEN, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name, color: state.person.color }),
    });
    const data = await res.json();
    state.person = data;
    toast('Profile saved!');
  } catch (e) {
    toast('Save failed');
  }
  state.saving = false;
  render();
}

async function startGoogle() {
  try {
    const res = await fetch(API + '/claim/' + TOKEN + '/google/start', { method: 'POST' });
    const data = await res.json();
    window.location.href = data.authUrl;
  } catch (e) {
    toast('Failed to start Google connect');
  }
}

// Check for google=success in URL
if (new URLSearchParams(window.location.search).get('google') === 'success') {
  // Clean URL
  history.replaceState(null, '', window.location.pathname);
}

load();
</script>
</body>
</html>"##;
```

- [ ] **Step 2: Register the claim page route**

In `build_routes()`, add the claim page route before the API nest. Update the `Route::new()` block:

```rust
    Route::new()
        .at("/health", poem::get(health))
        .at("/kaithheathcheck", poem::get(health))
        .nest("/docs", api_service.scalar())
        .nest("/openapi", api_service.spec_endpoint())
        .at("/claim/:token", poem::get(claim_page))
        .nest("/api", api_service.into_endpoint())
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p family-center-server 2>&1 | grep "^error"`
Expected: no errors

- [ ] **Step 4: Commit**

```bash
git add apps/server/src/application/routes/mod.rs
git commit -m "feat(server): serve standalone claim HTML page at /claim/:token"
```

---

### Task 8: Fix Gmail linking in person creation modal

**Files:**
- Modify: `apps/mobile/src/screens/PeopleManagement.tsx`

- [ ] **Step 1: Add GmailPicker to the create/edit modal**

In `apps/mobile/src/screens/PeopleManagement.tsx`, inside the `<IonModal>` content (after the "Profile picture URL" field and before the preview card), add:

```tsx
              {/* Gmail linking */}
              <div style={s.field}>
                <label style={s.fieldLabel}>Google account</label>
                {editing ? (
                  <GmailPicker
                    personId={editing.id}
                    accounts={accounts}
                    rules={rules}
                    onLink={(gid) => handleLink(editing.id, gid)}
                    onUnlink={(gid) => handleUnlink(editing.id, gid)}
                    isLinking={isLinking}
                  />
                ) : (
                  <span style={{ fontSize: '12px', color: 'var(--fc-text-muted)' }}>
                    Save the person first, then link a Google account.
                  </span>
                )}
              </div>
```

This shows the GmailPicker when editing an existing person, and an informational message when creating (since we need a person ID to link). The GmailPicker is already visible on the main cards outside the modal — this adds it inside the modal too.

- [ ] **Step 2: Verify TypeScript compiles**

Run: `cd apps/mobile && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 3: Commit**

```bash
git add apps/mobile/src/screens/PeopleManagement.tsx
git commit -m "feat(mobile): add GmailPicker to person edit modal"
```

---

### Task 9: QR code generation in PeopleManagement

**Files:**
- Modify: `apps/mobile/package.json` (add qrcode dependency)
- Modify: `apps/mobile/src/api/hooks.ts` (add claim token hook)
- Modify: `apps/mobile/src/screens/PeopleManagement.tsx` (add QR modal)

- [ ] **Step 1: Install QR code library**

```bash
cd apps/mobile && npm install qrcode.react
```

- [ ] **Step 2: Add claim token hook**

In `apps/mobile/src/api/hooks.ts`, add a new hook (after the people hooks section):

```typescript
// ---- Claim Tokens ----
export function useCreateClaimToken() {
  return useMutation({
    mutationFn: (personId: string) =>
      api.post<{ token: string; expiresAt: string; claimUrl: string }>(`/people/${personId}/claim-token`, {}),
  });
}
```

- [ ] **Step 3: Add QR modal to PeopleManagement**

In `apps/mobile/src/screens/PeopleManagement.tsx`:

Add imports at the top:

```typescript
import { QRCodeSVG } from 'qrcode.react';
import { useCreateClaimToken } from '../api/hooks';
```

Add state for the QR modal (after existing state declarations):

```typescript
  const [qrPerson, setQrPerson] = useState<Person | null>(null);
  const [qrUrl, setQrUrl] = useState('');
  const [qrExpiry, setQrExpiry] = useState('');
  const claimTokenMutation = useCreateClaimToken();
```

Add the QR generation handler:

```typescript
  const openQr = async (person: Person) => {
    try {
      const result = await claimTokenMutation.mutateAsync(person.id);
      setQrPerson(person);
      setQrUrl(result.claimUrl);
      setQrExpiry(new Date(result.expiresAt).toLocaleTimeString());
    } catch (e) {
      console.error('Failed to generate QR code', e);
    }
  };
```

Add a QR button to each person card. In the person card header (the `<button style={s.laneCardHeader} ...>` element), add a QR button before the edit icon SVG:

```tsx
                    <button
                      style={{
                        background: 'none',
                        border: '1px solid var(--fc-border)',
                        borderRadius: '8px',
                        padding: '6px 8px',
                        cursor: 'pointer',
                        display: 'flex',
                        alignItems: 'center',
                        marginRight: '4px',
                      }}
                      onClick={(e) => {
                        e.stopPropagation();
                        openQr(person);
                      }}
                    >
                      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="var(--fc-text-muted)" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                        <rect x="2" y="2" width="8" height="8" rx="1"/>
                        <rect x="14" y="2" width="8" height="8" rx="1"/>
                        <rect x="2" y="14" width="8" height="8" rx="1"/>
                        <rect x="14" y="14" width="4" height="4"/>
                        <line x1="22" y1="14" x2="22" y2="14.01"/>
                        <line x1="22" y1="22" x2="22" y2="22.01"/>
                        <line x1="18" y1="18" x2="18" y2="18.01"/>
                      </svg>
                    </button>
```

Add the QR modal at the bottom of the component (before the closing `</IonContent>`):

```tsx
        {/* ── QR Code modal ── */}
        <IonModal isOpen={!!qrPerson} onDidDismiss={() => setQrPerson(null)}>
          <IonHeader>
            <IonToolbar>
              <IonTitle style={{ fontFamily: 'var(--fc-font-display)' }}>
                QR Code
              </IonTitle>
              <button slot="end" style={s.modalCancel} onClick={() => setQrPerson(null)}>
                Close
              </button>
            </IonToolbar>
          </IonHeader>
          <IonContent>
            <div style={{
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              padding: '32px 24px',
              gap: '20px',
              textAlign: 'center',
            }}>
              <div style={{
                fontFamily: 'var(--fc-font-display)',
                fontSize: '18px',
                fontWeight: 700,
                color: 'var(--fc-text-primary)',
              }}>
                Set up {qrPerson?.name}'s profile
              </div>
              <div style={{
                background: '#fff',
                padding: '20px',
                borderRadius: '16px',
              }}>
                {qrUrl && <QRCodeSVG value={qrUrl} size={220} />}
              </div>
              <div style={{
                fontSize: '13px',
                color: 'var(--fc-text-secondary)',
                lineHeight: '1.5',
              }}>
                Scan this QR code with {qrPerson?.name}'s phone to set up their
                photo and Google account.
                <br />
                <span style={{ fontSize: '11px', color: 'var(--fc-text-muted)' }}>
                  Expires at {qrExpiry}
                </span>
              </div>
              <button
                style={{
                  ...s.addBtn,
                  marginTop: '8px',
                  borderStyle: 'solid',
                  maxWidth: '240px',
                }}
                onClick={() => qrPerson && openQr(qrPerson)}
                disabled={claimTokenMutation.isPending}
              >
                {claimTokenMutation.isPending ? 'Generating...' : 'Regenerate QR'}
              </button>
            </div>
          </IonContent>
        </IonModal>
```

- [ ] **Step 4: Verify TypeScript compiles**

Run: `cd apps/mobile && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 5: Commit**

```bash
git add apps/mobile/package.json apps/mobile/package-lock.json \
       apps/mobile/src/api/hooks.ts \
       apps/mobile/src/screens/PeopleManagement.tsx
git commit -m "feat(mobile): add QR code generation for person claim flow"
```

---

### Task 10: Update .env.example with new variables

**Files:**
- Modify: `apps/server/.env.example` (or root `.env.example`)

- [ ] **Step 1: Add new env vars to .env.example**

Append:

```
# S3-compatible storage (Leapcell built-in)
S3_ENDPOINT=
S3_BUCKET=family-center
S3_ACCESS_KEY=
S3_SECRET_KEY=
S3_REGION=auto

# Public URL of the server (used for claim URLs, OAuth callbacks)
PUBLIC_URL=http://localhost:8080
```

- [ ] **Step 2: Commit**

```bash
git add apps/server/.env.example .env.example
git commit -m "docs: add S3 and PUBLIC_URL env vars to .env.example"
```
