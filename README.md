# Family Center

A wall-mounted family/team scheduler for Android touchscreen displays.

## What it is

- **Ionic React + Capacitor** Android app — landscape kiosk UI
- **Rust/Axum backend** — deployable on Leapcell, runs on your server/cloud
- **PostgreSQL** — server-side data store
- **SQLite** — local cache in the app for offline reading
- Aggregates events from multiple **Google Calendars**
- Swim-lane layout: one lane per person + shared lane
- Day view (time-based grid) and Week view
- Local recurring activities (chores, routines, swim practice, etc.)
- Three-tier duplicate detection

## Quick start

```bash
# Prerequisites: Node.js 20+, Rust toolchain, PostgreSQL

# Clone and install
git clone <repo-url>
cd family-center
npm install

# Configure
cp .env.example apps/server/.env
# Edit apps/server/.env (DATABASE_URL etc.)

# Initialize database
bash scripts/init-db.sh

# Start everything
npm run dev
```

See `docs/local-dev.md` for detailed instructions.

## Structure

```
apps/mobile/       Ionic React + Capacitor Android app
apps/server/       Rust Axum backend
packages/contracts/ Shared TypeScript types
docs/              Architecture, deploy, local dev notes
infra/             Deployment config
scripts/           Database init, seed, helpers
```

## Technology choices

| Concern | Choice | Reason |
|---------|--------|--------|
| Backend framework | Axum | Tower middleware ecosystem, simpler than Actix |
| Database ORM | SQLx | Async-first, compile-time queries, clean migrations |
| Mobile framework | Ionic React | Capacitor Android target, Ionic components |
| State | Zustand | Minimal, no boilerplate |
| Data fetching | TanStack Query | Caching, background sync, optimistic updates |
| Local storage | SQLite via Capacitor | Offline-first cache |

See `docs/architecture.md` for full decisions.
