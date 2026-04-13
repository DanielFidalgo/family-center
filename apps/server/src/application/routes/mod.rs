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
<title>Your Profile!</title>
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Baloo+2:wght@500;600;700;800&family=Nunito:wght@500;600;700;800&display=swap" rel="stylesheet">
<style>
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:'Nunito',sans-serif;background:#FFF8F0;color:#2D1B4E;min-height:100dvh;padding:20px 16px 100px;font-weight:600;overflow-x:hidden}

/* floating shapes background */
body::before{content:'';position:fixed;top:-60px;right:-60px;width:200px;height:200px;border-radius:50%;background:radial-gradient(circle,rgba(244,63,138,0.1),transparent 70%);z-index:0;animation:float1 8s ease-in-out infinite}
body::after{content:'';position:fixed;bottom:-80px;left:-80px;width:250px;height:250px;border-radius:50%;background:radial-gradient(circle,rgba(59,139,255,0.08),transparent 70%);z-index:0;animation:float2 10s ease-in-out infinite}

@keyframes float1{0%,100%{transform:translate(0,0) scale(1)}50%{transform:translate(-30px,40px) scale(1.1)}}
@keyframes float2{0%,100%{transform:translate(0,0) scale(1)}50%{transform:translate(30px,-30px) scale(1.15)}}
@keyframes pop-in{0%{opacity:0;transform:scale(0.8) translateY(20px)}60%{transform:scale(1.03) translateY(-2px)}100%{opacity:1;transform:scale(1) translateY(0)}}
@keyframes wiggle{0%,100%{transform:rotate(0)}20%{transform:rotate(-3deg)}40%{transform:rotate(3deg)}60%{transform:rotate(-2deg)}80%{transform:rotate(1deg)}}
@keyframes glow-pulse{0%,100%{box-shadow:0 4px 20px rgba(255,140,66,0.25)}50%{box-shadow:0 4px 30px rgba(255,140,66,0.45)}}
@keyframes shimmer{0%{background-position:-200% center}100%{background-position:200% center}}
@keyframes confetti{0%{transform:translateY(0) rotate(0) scale(1);opacity:1}100%{transform:translateY(-120px) rotate(720deg) scale(0);opacity:0}}

#app{position:relative;z-index:1}

.card{background:#fff;border:2px solid #E8DDF5;border-radius:20px;padding:20px;margin-bottom:14px;animation:pop-in 0.4s cubic-bezier(0.34,1.56,0.64,1) both;position:relative;overflow:hidden;box-shadow:0 2px 12px rgba(45,27,78,0.06)}
.card::before{content:'';position:absolute;top:0;left:0;right:0;height:3px;background:linear-gradient(90deg,#F43F8A,#FF8C42,#3B8BFF,#10B981);border-radius:20px 20px 0 0}
.card:nth-child(1){animation-delay:0.05s}
.card:nth-child(2){animation-delay:0.1s}
.card:nth-child(3){animation-delay:0.15s}
.card:nth-child(4){animation-delay:0.2s}

h1{font-family:'Baloo 2',cursive;font-size:28px;font-weight:800;margin-bottom:2px;background:linear-gradient(135deg,#FF8C42,#F43F8A);-webkit-background-clip:text;-webkit-text-fill-color:transparent;background-clip:text;animation:pop-in 0.5s cubic-bezier(0.34,1.56,0.64,1) both}
.sub{font-size:15px;color:#6B5A8A;margin-bottom:20px;animation:pop-in 0.5s 0.1s cubic-bezier(0.34,1.56,0.64,1) both}
.sub .wave{display:inline-block;animation:wiggle 1s 0.5s ease-in-out}

label{display:block;font-family:'Baloo 2',cursive;font-size:14px;font-weight:700;color:#6B5A8A;margin-bottom:8px;letter-spacing:0.02em}

input[type=text]{width:100%;background:#FFF8F0;border:2px solid #E8DDF5;border-radius:14px;padding:14px 16px;color:#2D1B4E;font-size:18px;font-family:'Nunito',sans-serif;font-weight:700;outline:none;transition:border-color 0.2s,box-shadow 0.2s}
input[type=text]:focus{border-color:#3B8BFF;box-shadow:0 0 16px rgba(59,139,255,0.2)}

.colors{display:flex;flex-wrap:wrap;gap:12px;margin-top:6px;justify-content:center}
.color-btn{width:48px;height:48px;border-radius:14px;border:3px solid transparent;cursor:pointer;transition:transform 0.2s cubic-bezier(0.34,1.56,0.64,1),border-color 0.2s,box-shadow 0.2s;box-shadow:0 3px 10px rgba(45,27,78,0.12)}
.color-btn:active{transform:scale(0.9)!important}
.color-btn.active{transform:scale(1.2);border-color:#2D1B4E;box-shadow:0 0 20px rgba(45,27,78,0.2)}

.avatar-section{text-align:center;margin:12px 0}
.avatar-preview{width:100px;height:100px;border-radius:50%;object-fit:cover;margin:0 auto 14px;display:block;border:4px solid #FF8C42;box-shadow:0 4px 20px rgba(255,140,66,0.2);animation:pop-in 0.5s cubic-bezier(0.34,1.56,0.64,1) both}
.avatar-initials{width:100px;height:100px;border-radius:50%;display:flex;align-items:center;justify-content:center;font-family:'Baloo 2',cursive;font-size:36px;font-weight:800;color:#fff;margin:0 auto 14px;border:4px solid rgba(255,255,255,0.5);box-shadow:0 4px 20px rgba(45,27,78,0.15);animation:pop-in 0.5s cubic-bezier(0.34,1.56,0.64,1) both}

.btn-row{display:flex;gap:10px;justify-content:center}
.btn{padding:12px 20px;border-radius:14px;border:2px solid #E8DDF5;background:#fff;color:#2D1B4E;font-size:14px;font-weight:700;font-family:'Nunito',sans-serif;cursor:pointer;transition:transform 0.15s cubic-bezier(0.34,1.56,0.64,1),background 0.15s}
.btn:active{transform:scale(0.92);background:#FFF0E8}
.btn-icon{font-size:16px;margin-right:4px}

.btn-primary{background:linear-gradient(135deg,#FF8C42,#F43F8A);border:none;color:#fff;font-size:17px;font-weight:800;font-family:'Baloo 2',cursive;animation:glow-pulse 3s ease-in-out infinite}
.btn-primary:active{transform:scale(0.94)}
.btn-google{background:linear-gradient(135deg,#3B8BFF,#A855F7);border:none;color:#fff}

.btn-full{width:100%;padding:16px;font-size:16px;margin-top:8px;min-height:52px}

.google-linked{display:flex;align-items:center;gap:10px;padding:12px 16px;background:linear-gradient(135deg,rgba(16,185,129,0.1),rgba(16,185,129,0.04));border:2px solid rgba(16,185,129,0.3);border-radius:14px;font-size:14px;font-weight:700;color:#059669;animation:pop-in 0.4s cubic-bezier(0.34,1.56,0.64,1) both}
.google-linked svg{flex-shrink:0}

.expired{text-align:center;padding:60px 20px;color:#6B5A8A;animation:pop-in 0.5s cubic-bezier(0.34,1.56,0.64,1) both}
.expired h2{font-family:'Baloo 2',cursive;font-size:24px;color:#F43F8A;margin-bottom:10px}
.expired p{font-size:15px;line-height:1.6}

.saving{opacity:.5;pointer-events:none}

.toast{position:fixed;bottom:28px;left:50%;transform:translateX(-50%) scale(0.8);background:linear-gradient(135deg,#10B981,#3B8BFF);color:#fff;padding:12px 24px;border-radius:16px;font-size:15px;font-weight:800;font-family:'Baloo 2',cursive;z-index:99;opacity:0;transition:opacity 0.3s,transform 0.3s cubic-bezier(0.34,1.56,0.64,1);box-shadow:0 6px 24px rgba(16,185,129,0.25)}
.toast.show{opacity:1;transform:translateX(-50%) scale(1)}

input[type=file]{display:none}

/* Confetti container */
#confetti{position:fixed;top:0;left:0;width:100%;height:100%;pointer-events:none;z-index:100;overflow:hidden}
.confetti-piece{position:absolute;width:10px;height:10px;border-radius:3px;animation:confetti 1s ease-out forwards}
</style>
</head>
<body>

<div id="app"></div>
<div class="toast" id="toast"></div>
<div id="confetti"></div>

<script>
const TOKEN = '{{TOKEN}}';
const API = window.location.origin + '/api';
const COLORS = ['#FF6BCA','#44BBFF','#3DE8A0','#FFD23F','#C084FC','#FF8C42','#5E81F4','#FF6B6B','#34D399','#FBBF24','#06B6D4','#A78BFA'];

let state = { person: null, linkedEmails: [], saving: false };

function $(id) { return document.getElementById(id); }

function showConfetti() {
  const c = $('confetti');
  const colors = ['#FF6BCA','#44BBFF','#3DE8A0','#FFD23F','#C084FC','#FF8C42'];
  for (let i = 0; i < 30; i++) {
    const piece = document.createElement('div');
    piece.className = 'confetti-piece';
    piece.style.left = Math.random() * 100 + '%';
    piece.style.top = Math.random() * 40 + 20 + '%';
    piece.style.background = colors[Math.floor(Math.random() * colors.length)];
    piece.style.animationDelay = Math.random() * 0.5 + 's';
    piece.style.animationDuration = 0.8 + Math.random() * 0.6 + 's';
    piece.style.width = 6 + Math.random() * 8 + 'px';
    piece.style.height = 6 + Math.random() * 8 + 'px';
    piece.style.borderRadius = Math.random() > 0.5 ? '50%' : '3px';
    c.appendChild(piece);
  }
  setTimeout(() => c.innerHTML = '', 2000);
}

function toast(msg) {
  const t = $('toast');
  t.textContent = msg;
  t.classList.add('show');
  showConfetti();
  setTimeout(() => t.classList.remove('show'), 2500);
}

function initials(name) {
  return (name || '?').split(' ').map(w => w[0]).join('').toUpperCase().slice(0,2);
}

function render() {
  const app = $('app');
  if (!state.person) {
    app.innerHTML = '<div class="expired"><h2>Oops, link expired!</h2><p>This link isn\'t working anymore.<br>Ask for a new QR code from the family display!</p></div>';
    return;
  }
  const p = state.person;
  const hasAvatar = p.avatarUrl && !p.avatarUrl.includes('undefined');
  const avatarHtml = hasAvatar
    ? `<img class="avatar-preview" src="${p.avatarUrl}" alt="avatar" />`
    : `<div class="avatar-initials" style="background:${p.color}">${initials(p.name)}</div>`;

  const googleHtml = state.linkedEmails.length > 0
    ? state.linkedEmails.map(e => `<div class="google-linked"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>${e}</div>`).join('')
    : `<button class="btn btn-google btn-full" onclick="startGoogle()"><span class="btn-icon">G</span> Connect Google Account</button>`;

  app.innerHTML = `
    <h1>Hey ${p.name}!</h1>
    <p class="sub">Make this profile yours <span class="wave">&#128075;</span></p>

    <div class="card">
      <div class="avatar-section">
        ${avatarHtml}
        <div class="btn-row">
          <button class="btn" onclick="pickPhoto('camera')"><span class="btn-icon">&#128247;</span> Camera</button>
          <button class="btn" onclick="pickPhoto('gallery')"><span class="btn-icon">&#128444;</span> Gallery</button>
        </div>
        <input type="file" id="camera-input" accept="image/*" capture="environment" onchange="uploadPhoto(this)" />
        <input type="file" id="gallery-input" accept="image/*" onchange="uploadPhoto(this)" />
      </div>
    </div>

    <div class="card">
      <label>Your Name</label>
      <input type="text" id="name-input" value="${p.name}" />
    </div>

    <div class="card">
      <label>Pick Your Color!</label>
      <div class="colors">
        ${COLORS.map(c => `<button class="color-btn${p.color===c?' active':''}" style="background:${c}" onclick="pickColor('${c}')"></button>`).join('')}
      </div>
    </div>

    <div class="card">
      <label>Google Calendar</label>
      ${googleHtml}
    </div>

    <button class="btn btn-primary btn-full${state.saving?' saving':''}" onclick="saveProfile()">
      ${state.saving ? 'Saving...' : 'Save My Profile!'}
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
    toast('Looking great!');
  } catch (e) {
    toast('Oops, try again!');
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
    toast('Awesome, saved!');
  } catch (e) {
    toast('Oops, try again!');
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
    toast('Something went wrong!');
  }
}

if (new URLSearchParams(window.location.search).get('google') === 'success') {
  history.replaceState(null, '', window.location.pathname);
  setTimeout(() => toast('Google connected!'), 500);
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
