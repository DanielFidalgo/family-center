# Local Development

## Prerequisites

- **Node.js** 20+
- **Rust** (stable, via rustup): https://rustup.rs
- **PostgreSQL** 14+
- **sqlx-cli**: `cargo install sqlx-cli --no-default-features --features postgres`

Optional (for Android build):
- Android Studio
- Java 17+

## Setup

### 1. Clone and install

```bash
git clone <repo-url>
cd family-center
npm install
```

### 2. Configure the server

```bash
cp .env.example apps/server/.env
```

Edit `apps/server/.env`:

```
DATABASE_URL=postgres://your_user:your_pass@localhost:5432/family_center
SERVER_PORT=3000
JWT_SECRET=dev-only-change-in-production
MOCK_CALENDAR=true
```

Set `MOCK_CALENDAR=true` to skip real Google OAuth during development.

### 3. Initialize the database

```bash
# Create DB and run migrations
bash scripts/init-db.sh

# Optional: seed with demo data
bash scripts/seed.sh
```

Or manually:
```bash
createdb family_center
cd apps/server && sqlx migrate run
```

### 4. Start the server

```bash
npm run dev:server
# Or: cd apps/server && cargo run
```

Server listens on `http://localhost:3000`.

### 5. Start the mobile app (web mode)

```bash
npm run dev:mobile
# Or: cd apps/mobile && npm run dev
```

App runs at `http://localhost:5173`. In a browser, it behaves as a kiosk web app.

### 6. Bootstrap

Open `http://localhost:5173`. The onboarding screen will call `POST /auth/bootstrap` automatically.

With `MOCK_CALENDAR=true`:
- Connect a Google Account → creates a mock account with 3 calendars
- Select calendars → enable "Personal" and "Family"
- Run sync → populates the schedule with mock events
- Open Week view → should show swim lanes with events

## Running tests

```bash
# All tests
npm test

# Backend only
npm run test:server
# Or: cd apps/server && cargo test

# Mobile only
npm run test:mobile
# Or: cd apps/mobile && npm test
```

## Database management

```bash
# Create a new migration
cd apps/server && sqlx migrate add <name>

# Run migrations
cd apps/server && sqlx migrate run

# Revert last migration
cd apps/server && sqlx migrate revert
```

## Environment variables reference

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | required | PostgreSQL connection string |
| `SERVER_PORT` | `3000` | HTTP port |
| `JWT_SECRET` | `dev-secret` | JWT signing key (change in prod!) |
| `MOCK_CALENDAR` | `true` | Skip real Google OAuth |
| `GOOGLE_CLIENT_ID` | — | Required if `MOCK_CALENDAR=false` |
| `GOOGLE_CLIENT_SECRET` | — | Required if `MOCK_CALENDAR=false` |
| `GOOGLE_REDIRECT_URI` | `http://localhost:3000/auth/google/callback` | OAuth callback URL |
| `VITE_API_BASE_URL` | `http://localhost:3000` | API URL for the mobile app |

## Real Google OAuth setup

1. Go to https://console.cloud.google.com
2. Create a project → Enable **Google Calendar API**
3. Create OAuth 2.0 credentials (Web application type)
4. Add `http://localhost:3000/auth/google/callback` as an authorized redirect URI
5. Copy client ID and secret to `apps/server/.env`
6. Set `MOCK_CALENDAR=false`

## Android build

```bash
cd apps/mobile
npm run build
npm run cap:sync
npm run cap:android
# Opens Android Studio → Run on device
```

For wall display: set device to kiosk mode / screen pinning after installing.
