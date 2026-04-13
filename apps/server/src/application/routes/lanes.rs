use std::sync::Arc;
use poem_openapi::{Object, OpenApi, param::Path, payload::Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::routes::security::{ApiError, ApiTags, BearerAuth};
use crate::configuration::app_context::IAppContext;
use crate::domain::entities::lane::LaneAssignmentRule;
use crate::infrastructure::auth;

#[derive(Debug, Deserialize, Serialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct LinkAccountRequest {
    /// The Google account to link.
    pub google_account_id: Uuid,
    /// The person to assign events to. Null for shared lane.
    pub person_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct UnlinkAccountRequest {
    /// The Google account to unlink.
    pub google_account_id: Uuid,
    /// The person to unlink from. Null for shared lane.
    pub person_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct LinkResult {
    pub rules_created: usize,
}

#[derive(Debug, Serialize, Object)]
#[serde(rename_all = "camelCase")]
pub struct UnlinkResult {
    pub rules_deleted: u64,
}

pub struct LanesApi {
    pub context: Arc<dyn IAppContext>,
}

impl LanesApi {
    fn verify(&self, auth: &BearerAuth) -> Result<(), ApiError> {
        auth::verify_token(&auth.0.token, self.context.config().jwt_secret())
            .map_err(|_| ApiError::unauthorized())?;
        Ok(())
    }

    async fn household_id(&self) -> Result<Uuid, ApiError> {
        let h = self
            .context
            .household_repository()
            .find_first()
            .await
            .map_err(ApiError::from)?
            .ok_or_else(|| {
                ApiError::bad_request("No household configured. Call /auth/bootstrap first.")
            })?;
        Ok(h.id)
    }
}

#[OpenApi(tag = "ApiTags::Lanes")]
impl LanesApi {
    /// List all lane assignment rules for the household.
    #[oai(path = "/lane-rules", method = "get")]
    pub async fn list_rules(
        &self,
        auth: BearerAuth,
    ) -> Result<Json<Vec<LaneAssignmentRule>>, ApiError> {
        self.verify(&auth)?;
        let household_id = self.household_id().await?;
        let rules = self
            .context
            .lane_rule_repository()
            .find_by_household(household_id)
            .await
            .map_err(ApiError::from)?;
        Ok(Json(rules))
    }

    /// Link a Google account to a person (or shared lane).
    ///
    /// Creates lane assignment rules for every selected calendar
    /// under the given Google account, pointing to the specified person.
    #[oai(path = "/lane-rules/link-account", method = "post")]
    pub async fn link_account(
        &self,
        auth: BearerAuth,
        body: Json<LinkAccountRequest>,
    ) -> Result<Json<LinkResult>, ApiError> {
        self.verify(&auth)?;
        let household_id = self.household_id().await?;

        let calendars = self
            .context
            .calendar_source_repository()
            .find_by_account(body.google_account_id)
            .await
            .map_err(ApiError::from)?;

        let selected: Vec<_> = calendars.into_iter().filter(|c| c.is_selected).collect();
        let lane_target = if body.person_id.is_some() {
            "person"
        } else {
            "shared"
        };

        let mut created = 0usize;
        for cal in &selected {
            self.context
                .lane_rule_repository()
                .create(
                    household_id,
                    Some(cal.id),
                    None,
                    body.person_id,
                    lane_target,
                    100,
                )
                .await
                .map_err(ApiError::from)?;
            created += 1;
        }

        Ok(Json(LinkResult {
            rules_created: created,
        }))
    }

    /// Unlink a Google account from a person (or shared lane).
    ///
    /// Removes all lane assignment rules that match the Google account's
    /// calendars and the specified person.
    #[oai(path = "/lane-rules/unlink-account", method = "post")]
    pub async fn unlink_account(
        &self,
        auth: BearerAuth,
        body: Json<UnlinkAccountRequest>,
    ) -> Result<Json<UnlinkResult>, ApiError> {
        self.verify(&auth)?;
        let household_id = self.household_id().await?;

        let calendars = self
            .context
            .calendar_source_repository()
            .find_by_account(body.google_account_id)
            .await
            .map_err(ApiError::from)?;

        let mut total_deleted = 0u64;
        for cal in &calendars {
            let deleted = self
                .context
                .lane_rule_repository()
                .delete_by_calendar_source_and_person(household_id, cal.id, body.person_id)
                .await
                .map_err(ApiError::from)?;
            total_deleted += deleted;
        }

        Ok(Json(UnlinkResult {
            rules_deleted: total_deleted,
        }))
    }

    /// Delete a single lane assignment rule by ID.
    #[oai(path = "/lane-rules/:id", method = "delete")]
    pub async fn delete_rule(
        &self,
        auth: BearerAuth,
        id: Path<Uuid>,
    ) -> Result<Json<UnlinkResult>, ApiError> {
        self.verify(&auth)?;
        self.context
            .lane_rule_repository()
            .delete_by_id(id.0)
            .await
            .map_err(ApiError::from)?;
        Ok(Json(UnlinkResult { rules_deleted: 1 }))
    }
}
