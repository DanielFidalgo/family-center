pub mod activities;
pub mod auth;
pub mod claim;
pub mod google;
pub mod lanes;
pub mod people;
pub mod schedule;
pub mod security;
pub mod settings;
pub mod sync;

use crate::configuration::app_context::IAppContext;
use poem::{handler, IntoEndpoint, Route};
use poem_openapi::OpenApiService;
use std::sync::Arc;

#[handler]
fn health() -> &'static str {
    "ok"
}

#[handler]
fn claim_page(req: &poem::Request) -> poem::Response {
    let token = req
        .uri()
        .path()
        .trim_start_matches("/claim/")
        .split('?')
        .next()
        .unwrap_or("");
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

use activities::ActivitiesApi;
use auth::AuthApi;
use claim::ClaimApi;
use google::GoogleApi;
use lanes::LanesApi;
use people::PeopleApi;
use schedule::ScheduleApi;
use settings::SettingsApi;
use sync::SyncApi;

pub fn build_routes(context: Arc<dyn IAppContext>) -> Route {
    let auth_api = AuthApi {
        context: context.clone(),
    };
    let google_api = GoogleApi {
        context: context.clone(),
    };
    let sync_api = SyncApi {
        context: context.clone(),
    };
    let schedule_api = ScheduleApi {
        context: context.clone(),
    };
    let people_api = PeopleApi {
        context: context.clone(),
    };
    let activities_api = ActivitiesApi {
        context: context.clone(),
    };
    let settings_api = SettingsApi {
        context: context.clone(),
    };
    let lanes_api = LanesApi {
        context: context.clone(),
    };
    let claim_api = ClaimApi {
        context: context.clone(),
    };

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

    Route::new()
        .at("/health", poem::get(health))
        .at("/kaithheathcheck", poem::get(health))
        .at("/claim/:token", poem::get(claim_page))
        .nest("/docs", api_service.scalar())
        .nest("/openapi", api_service.spec_endpoint())
        .nest("/api", api_service.into_endpoint())
}
