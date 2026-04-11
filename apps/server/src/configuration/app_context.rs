use std::sync::Arc;

use crate::configuration::config::IAppConfig;
use crate::domain::repositories::{
    calendar_source_repository::ICalendarSourceRepository,
    google_account_repository::IGoogleAccountRepository,
    household_repository::IHouseholdRepository,
    lane_rule_repository::ILaneRuleRepository,
    local_activity_repository::ILocalActivityRepository,
    merged_event_repository::IMergedEventRepository,
    person_repository::IPersonRepository,
    settings_repository::ISettingsRepository,
    source_event_repository::ISourceEventRepository,
};
use crate::domain::sync::ISyncService;

pub trait IAppContext: Send + Sync {
    fn config(&self) -> &dyn IAppConfig;
    fn household_repository(&self) -> &dyn IHouseholdRepository;
    fn person_repository(&self) -> &dyn IPersonRepository;
    fn google_account_repository(&self) -> &dyn IGoogleAccountRepository;
    fn calendar_source_repository(&self) -> &dyn ICalendarSourceRepository;
    fn source_event_repository(&self) -> &dyn ISourceEventRepository;
    fn merged_event_repository(&self) -> &dyn IMergedEventRepository;
    fn local_activity_repository(&self) -> &dyn ILocalActivityRepository;
    fn settings_repository(&self) -> &dyn ISettingsRepository;
    fn lane_rule_repository(&self) -> &dyn ILaneRuleRepository;
    fn sync_service(&self) -> &dyn ISyncService;
}

pub struct AppContext {
    pub config: Arc<dyn IAppConfig>,
    pub household_repository: Arc<dyn IHouseholdRepository>,
    pub person_repository: Arc<dyn IPersonRepository>,
    pub google_account_repository: Arc<dyn IGoogleAccountRepository>,
    pub calendar_source_repository: Arc<dyn ICalendarSourceRepository>,
    pub source_event_repository: Arc<dyn ISourceEventRepository>,
    pub merged_event_repository: Arc<dyn IMergedEventRepository>,
    pub local_activity_repository: Arc<dyn ILocalActivityRepository>,
    pub settings_repository: Arc<dyn ISettingsRepository>,
    pub lane_rule_repository: Arc<dyn ILaneRuleRepository>,
    pub sync_service: Arc<dyn ISyncService>,
}

impl IAppContext for AppContext {
    fn config(&self) -> &dyn IAppConfig { self.config.as_ref() }
    fn household_repository(&self) -> &dyn IHouseholdRepository { self.household_repository.as_ref() }
    fn person_repository(&self) -> &dyn IPersonRepository { self.person_repository.as_ref() }
    fn google_account_repository(&self) -> &dyn IGoogleAccountRepository { self.google_account_repository.as_ref() }
    fn calendar_source_repository(&self) -> &dyn ICalendarSourceRepository { self.calendar_source_repository.as_ref() }
    fn source_event_repository(&self) -> &dyn ISourceEventRepository { self.source_event_repository.as_ref() }
    fn merged_event_repository(&self) -> &dyn IMergedEventRepository { self.merged_event_repository.as_ref() }
    fn local_activity_repository(&self) -> &dyn ILocalActivityRepository { self.local_activity_repository.as_ref() }
    fn settings_repository(&self) -> &dyn ISettingsRepository { self.settings_repository.as_ref() }
    fn lane_rule_repository(&self) -> &dyn ILaneRuleRepository { self.lane_rule_repository.as_ref() }
    fn sync_service(&self) -> &dyn ISyncService { self.sync_service.as_ref() }
}
