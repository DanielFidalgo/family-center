use std::sync::Arc;

use crate::configuration::app_context::AppContext;
use crate::configuration::config::{Config, IAppConfig};
use crate::configuration::service_setup::{
    Config as ServiceConfig, HandlerFn, ServiceError, make_teardown, service_setup,
};
use crate::infrastructure::db;
use crate::infrastructure::repositories::{
    calendar_source_repository::CalendarSourceRepository,
    claim_token_repository::ClaimTokenRepository,
    google_account_repository::GoogleAccountRepository,
    household_repository::HouseholdRepository,
    lane_rule_repository::LaneRuleRepository,
    local_activity_repository::LocalActivityRepository,
    merged_event_repository::MergedEventRepository,
    person_repository::PersonRepository,
    settings_repository::SettingsRepository,
    source_event_repository::SourceEventRepository,
};
use crate::infrastructure::services::sync_service::SyncService;

pub async fn bootstrap() -> Result<(), ServiceError> {
    dotenvy::dotenv_override().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "family_center_server=debug,poem=debug".into()),
        )
        .init();

    let config = Config::from_env().expect("Failed to load config");
    let pool = db::connect(&config.database_url).await.expect("Failed to connect to database");

    if std::env::var("RUN_MIGRATIONS").unwrap_or_default() == "true" {
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");
    }

    let config: Arc<dyn IAppConfig> = Arc::new(config);

    // Create repositories
    let household_repo = Arc::new(HouseholdRepository::new(pool.clone()));
    let person_repo = Arc::new(PersonRepository::new(pool.clone()));
    let google_account_repo = Arc::new(GoogleAccountRepository::new(pool.clone()));
    let calendar_source_repo = Arc::new(CalendarSourceRepository::new(pool.clone()));
    let source_event_repo = Arc::new(SourceEventRepository::new(pool.clone()));
    let merged_event_repo = Arc::new(MergedEventRepository::new(pool.clone()));
    let local_activity_repo = Arc::new(LocalActivityRepository::new(pool.clone()));
    let settings_repo = Arc::new(SettingsRepository::new(pool.clone()));
    let lane_rule_repo = Arc::new(LaneRuleRepository::new(pool.clone()));
    let claim_token_repo = Arc::new(ClaimTokenRepository::new(pool.clone()));

    // Create sync service
    let sync_service = Arc::new(SyncService::new(
        config.clone(),
        calendar_source_repo.clone(),
        source_event_repo.clone(),
        household_repo.clone(),
        merged_event_repo.clone(),
        lane_rule_repo.clone(),
    ));

    // Build context
    let context = Arc::new(AppContext {
        config: config.clone(),
        household_repository: household_repo,
        person_repository: person_repo,
        google_account_repository: google_account_repo,
        calendar_source_repository: calendar_source_repo,
        source_event_repository: source_event_repo,
        merged_event_repository: merged_event_repo,
        local_activity_repository: local_activity_repo,
        settings_repository: settings_repo,
        lane_rule_repository: lane_rule_repo,
        claim_token_repository: claim_token_repo,
        sync_service,
    });

    let routes = crate::application::routes::build_routes(context);

    let service_config = ServiceConfig {
        service_url: format!("http://{}:{}", config.server_host(), config.server_port()),
        port: config.server_port(),
        routes,
    };

    let handlers: Vec<(String, HandlerFn)> = vec![];

    let teardown = vec![make_teardown(|| async move {
        tracing::info!("teardown done");
    })];

    service_setup(service_config, handlers, teardown).await
}
