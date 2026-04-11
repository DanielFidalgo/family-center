use std::sync::Arc;
use chrono::{DateTime, Utc};
use poem_openapi::{OpenApi, Object, param::Query, payload::Json};
use serde::Serialize;
use uuid::Uuid;
use crate::application::routes::security::{ApiError, ApiTags};
use crate::configuration::app_context::IAppContext;
use crate::domain::entities::{
    local_activity::LocalActivityWithRecurrence,
    merged_event::MergedEventGroupWithSources,
};

#[derive(Debug, Serialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleResponse {
    pub events: Vec<MergedEventGroupWithSources>,
    pub local_activities: Vec<LocalActivityWithRecurrence>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

pub struct ScheduleApi {
    pub context: Arc<dyn IAppContext>,
}

impl ScheduleApi {
    async fn household_id(&self) -> Result<Uuid, ApiError> {
        let h = self.context.household_repository().find_first()
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| ApiError::bad_request("No household configured. Call /auth/bootstrap first."))?;
        Ok(h.id)
    }
}

#[OpenApi(tag = "ApiTags::Schedule")]
impl ScheduleApi {
    /// Get the schedule for a time window.
    /// Auth token is optional — supply it for personalized data.
    #[oai(path = "/schedule", method = "get")]
    pub async fn get_schedule(
        &self,
        start: Query<DateTime<Utc>>,
        end: Query<DateTime<Utc>>,
    ) -> Result<Json<ScheduleResponse>, ApiError> {
        let household_id = self.household_id().await?;

        let events = self.context.merged_event_repository()
            .find_by_household_in_range(household_id, start.0, end.0)
            .await
            .map_err(ApiError::from)?;

        let local_activities = self.context.local_activity_repository()
            .find_by_household_in_range(household_id, start.0, end.0)
            .await
            .map_err(ApiError::from)?;

        Ok(Json(ScheduleResponse {
            events,
            local_activities,
            start: start.0,
            end: end.0,
        }))
    }
}
