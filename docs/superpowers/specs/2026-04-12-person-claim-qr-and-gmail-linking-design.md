# Person Claim QR Code & Gmail Linking

**Date:** 2026-04-12
**Status:** Approved

## Problem

1. The Gmail linking UI (GmailPicker) in PeopleManagement is not visible or accessible during person creation — users cannot link a Google account to a person.
2. There is no way for household members to self-service their own profile setup (photo, name, Google account) from their own phone.

## Solution

Two changes:

1. **Fix Gmail linking** — surface the GmailPicker in both the person creation form and the edit view. Show a prompt if no Google accounts are connected to the household yet.
2. **QR-based claim flow** — the admin generates a QR code per person. The family member scans it on their phone, lands on a standalone web page where they can edit their name, pick a color, upload a photo (camera or gallery), and connect their Google account. The QR encodes a short-lived token (30 min) so no persistent auth is needed.

## Approach: Short-Lived Token URL

A time-limited token per person is generated on demand, encoded in a QR code URL (`https://<host>/claim/<token>`). The token itself is the auth for the claim page — no login required. Tokens expire after 30 minutes. Generating a new token for a person replaces any existing one.

### Why this approach

- Simplest to implement — no new auth system, no PINs, no per-person JWTs
- Secure enough — 30-min expiry, random 32-char hex token, single use per person
- Works on any phone browser — no app install needed

## Data Model

### New table: `person_claim_tokens`

```sql
CREATE TABLE family_center.person_claim_tokens (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    person_id  UUID NOT NULL REFERENCES family_center.people(id) ON DELETE CASCADE UNIQUE,
    token      TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

- `person_id` is UNIQUE — one active token per person at a time
- Generating a new token upserts (replaces any existing)
- Token is a random 32-character hex string
- `expires_at` is set to `NOW() + 30 minutes`

### Photo storage

Avatars are uploaded to S3-compatible storage (Leapcell built-in).

- Bucket key: `avatars/<person_id>.jpg`
- Public URL stored in `people.avatar_url`
- Server-side S3 config via env vars: `S3_ENDPOINT`, `S3_BUCKET`, `S3_ACCESS_KEY`, `S3_SECRET_KEY`, `S3_REGION`

No new columns on the `people` table — `avatar_url` already exists.

## Server Endpoints

### Claim token management (authenticated)

| Method | Path | Auth | Purpose |
|--------|------|------|---------|
| `POST` | `/people/:id/claim-token` | Bearer | Generate 30-min claim token, return `{ token, expiresAt, claimUrl }` |

### Claim page endpoints (unauthenticated — token is the auth)

| Method | Path | Auth | Purpose |
|--------|------|------|---------|
| `GET` | `/claim/:token` | None | Validate token not expired, return person data (name, color, avatarUrl, linked Google accounts) |
| `PATCH` | `/claim/:token` | None | Update person name, color |
| `POST` | `/claim/:token/avatar` | None | Multipart image upload → S3 → update `avatar_url` |
| `POST` | `/claim/:token/google/start` | None | Start Google OAuth for this person, return `{ authUrl }` |
| `GET` | `/claim/google/callback` | None | Google OAuth callback, link account to person via lane rules |

All claim endpoints validate the token exists and `expires_at > NOW()`. Return 410 Gone if expired.

### Token validation behavior

- Token not found → 404
- Token expired → 410 Gone (with message "This link has expired. Ask for a new QR code.")
- Token valid → proceed

## Google OAuth via Claim

The claim page OAuth flow reuses the existing Google client infrastructure but with a different callback:

1. `POST /claim/:token/google/start` builds an OAuth URL with:
   - Same `GOOGLE_CLIENT_ID` and `GOOGLE_CLIENT_SECRET`
   - Redirect URI: `https://<host>/claim/google/callback`
   - State parameter encodes the claim token (so callback can look up the person)
   - Scope: `https://www.googleapis.com/auth/calendar.readonly`

2. `GET /claim/google/callback` receives the auth code:
   - Extracts claim token from state parameter
   - Validates token not expired
   - Exchanges code for tokens (reuses existing `exchange_code_and_persist`)
   - Creates lane assignment rules linking the Google account's selected calendars to the person
   - Redirects back to `https://<host>/claim/<token>?google=success`

**Note:** `https://<host>/claim/google/callback` must be added as an authorized redirect URI in Google Cloud Console.

## Mobile App Changes (Admin Side)

### PeopleManagement — Gmail linking fix

- Add the GmailPicker section to the person **creation** form (currently only in edit view)
- If no Google accounts are connected to the household, show: "No Google accounts connected. [Connect one](/settings)" or similar prompt
- Ensure the GmailPicker is visible and functional in the edit view (debug why it's not showing)

### PeopleManagement — QR code button

- Each person card gets a "QR Code" icon button
- Tapping calls `POST /people/:id/claim-token`
- Opens a modal showing:
  - QR code rendered from the returned `claimUrl`
  - Person's name: "Set up {name}'s profile"
  - Expiry notice: "Expires in 30 minutes"
  - "Regenerate" button to get a fresh token
- QR code rendered using a JS library (e.g. `qrcode.react` or `qrcode` package)

## Claim Page (Standalone Web)

A lightweight standalone HTML page served by the server. Not inside the Ionic app shell — works on any mobile browser without app install.

### Served at

`GET /claim/:token` serves the standalone HTML page (with token embedded as a JS variable). The page makes fetch calls to the JSON API endpoints listed above (which live under the existing `/api` prefix, e.g. `GET /api/claim/:token` for person data, `PATCH /api/claim/:token` for updates, etc.).

### Page contents

1. **Header**: "Set up your profile" with person's current name
2. **Avatar section**: 
   - Shows current avatar (or initials placeholder)
   - "Take photo" button (opens camera via `<input type="file" accept="image/*" capture="environment">`)
   - "Choose from gallery" button (opens file picker via `<input type="file" accept="image/*">`)
   - On selection: preview, then upload to `POST /claim/:token/avatar`
3. **Name field**: Editable text input, pre-filled with current name
4. **Color picker**: Same 12-color palette as the admin editor
5. **Google account section**:
   - If already linked: shows email with green checkmark
   - If not linked: "Connect Google Account" button → calls `POST /claim/:token/google/start` → redirects to Google
   - After OAuth return: shows success state with linked email
6. **Save button**: Calls `PATCH /claim/:token` with name + color changes
7. **Expired state**: If token is expired, show "This link has expired. Ask for a new QR code from the family display."

### Tech stack

- Plain HTML + vanilla JS + inline CSS (no build step, no framework)
- Served as a static asset or inline template from the Rust server
- Responsive, mobile-first design
- Image resized client-side before upload (max 512x512) to keep S3 storage small

## Environment Variables (New)

```
S3_ENDPOINT=https://<leapcell-s3-endpoint>
S3_BUCKET=family-center
S3_ACCESS_KEY=<access-key>
S3_SECRET_KEY=<secret-key>
S3_REGION=auto
```

## Security Considerations

- Claim tokens are 32-char random hex (128 bits of entropy) — not guessable
- 30-minute expiry limits window of exposure
- Generating a new token invalidates the old one (UNIQUE on person_id with upsert)
- Claim endpoints only allow editing the specific person the token was created for
- No household-level data is exposed through claim endpoints
- Image upload is validated server-side (content-type check, max size 5MB)

## Out of Scope

- Per-person login tokens / multi-user auth
- Claim page inside the Ionic mobile app
- Email/SMS delivery of the claim link (QR scan only)
- Token refresh / extend expiry
