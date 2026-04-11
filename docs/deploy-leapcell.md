# Deploying to Leapcell

Leapcell is a serverless Rust deployment platform. The server is designed to run there.

## Requirements

- All config from environment variables (no hardcoded secrets)
- Stateless — no assumptions about long-lived VM state
- PostgreSQL connection via `DATABASE_URL`

## Steps

### 1. Build and push

```bash
cd apps/server
cargo build --release
```

The binary is at `apps/server/target/release/server`.

### 2. Configure Leapcell service

In the Leapcell dashboard:

- **Build command**: `cargo build --release`
- **Start command**: `./target/release/server`
- **Port**: `3000`

### 3. Set environment variables

In Leapcell's environment configuration, set:

```
DATABASE_URL=postgres://user:pass@your-pg-host:5432/family_center
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
JWT_SECRET=<strong-random-secret>
GOOGLE_CLIENT_ID=<your-google-client-id>
GOOGLE_CLIENT_SECRET=<your-google-client-secret>
GOOGLE_REDIRECT_URI=https://<your-leapcell-domain>/auth/google/callback
MOCK_CALENDAR=false
RUST_LOG=family_center_server=info,tower_http=warn
```

### 4. Database

Use any managed PostgreSQL (Supabase, Neon, Railway, etc.).

Run migrations on first deploy:
```bash
DATABASE_URL=<your-prod-url> cargo sqlx migrate run
```

Or add to your deploy script to auto-migrate on startup — the server already calls `sqlx::migrate!().run(&pool).await?` on startup.

### 5. Google OAuth redirect URI

In Google Cloud Console, add your Leapcell domain as an authorized redirect URI:
```
https://<your-leapcell-domain>/auth/google/callback
```

Update `GOOGLE_REDIRECT_URI` in Leapcell env vars to match.

### 6. Mobile app configuration

Build the mobile app pointing at your deployed server:
```bash
cd apps/mobile
VITE_API_BASE_URL=https://<your-leapcell-domain> npm run build
npm run cap:sync
```

## Notes

- **Cold starts**: Leapcell may have cold starts. The Rust server starts in ~100ms so this is not an issue.
- **Stateless design**: the server holds no in-memory state between requests. All state is in PostgreSQL.
- **Migrations**: run automatically on server startup via `sqlx::migrate!()`.
- **Logs**: use `RUST_LOG=info` or `debug` for diagnostics.

## Leapcell config file

Create `leapcell.yaml` (optional, if Leapcell supports it):
```yaml
name: family-center-server
build:
  command: cargo build --release
  output: target/release/server
run:
  command: ./target/release/server
  port: 3000
env:
  - DATABASE_URL
  - SERVER_PORT
  - JWT_SECRET
  - GOOGLE_CLIENT_ID
  - GOOGLE_CLIENT_SECRET
  - GOOGLE_REDIRECT_URI
  - MOCK_CALENDAR
  - RUST_LOG
```
