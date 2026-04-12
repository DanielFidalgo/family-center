pub mod activities;
pub mod auth;
pub mod google;
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

use activities::ActivitiesApi;
use auth::AuthApi;
use google::GoogleApi;
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
        .at("/health", poem::get(health))
        .at("/kaithheathcheck", poem::get(health))
        .nest("/docs", api_service.scalar())
        .nest("/openapi", api_service.spec_endpoint())
        .nest("/api", api_service.into_endpoint())
}
