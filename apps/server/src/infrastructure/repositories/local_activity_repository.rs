use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::local_activity::{
    CreateLocalActivity, LocalActivity, LocalActivityRecurrence, LocalActivityWithRecurrence,
    UpdateLocalActivity,
};
use crate::domain::repositories::local_activity_repository::ILocalActivityRepository;

pub struct LocalActivityRepository {
    pool: PgPool,
}

impl LocalActivityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl LocalActivityRepository {
    async fn fetch_recurrence(
        &self,
        activity_id: Uuid,
    ) -> anyhow::Result<Option<LocalActivityRecurrence>> {
        let recurrence = sqlx::query_as!(
            LocalActivityRecurrence,
            "SELECT id, local_activity_id, freq, interval_val, by_day_of_week, by_day_of_month, until, count, created_at, updated_at FROM family_center.local_activity_recurrences WHERE local_activity_id = $1",
            activity_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(recurrence)
    }
}

#[async_trait]
impl ILocalActivityRepository for LocalActivityRepository {
    async fn find_by_household(
        &self,
        household_id: Uuid,
    ) -> anyhow::Result<Vec<LocalActivityWithRecurrence>> {
        let activities = sqlx::query_as!(
            LocalActivity,
            "SELECT id, household_id, person_id, title, description, color, start_at, end_at, is_all_day, created_at, updated_at FROM family_center.local_activities WHERE household_id = $1 ORDER BY start_at NULLS LAST, title",
            household_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::with_capacity(activities.len());
        for activity in activities {
            let recurrence = self.fetch_recurrence(activity.id).await?;
            result.push(LocalActivityWithRecurrence::new(activity, recurrence));
        }
        Ok(result)
    }

    async fn find_by_household_in_range(
        &self,
        household_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<Vec<LocalActivityWithRecurrence>> {
        let activities = sqlx::query_as!(
            LocalActivity,
            r#"SELECT id, household_id, person_id, title, description, color, start_at, end_at, is_all_day, created_at, updated_at
               FROM family_center.local_activities
               WHERE household_id = $1
                 AND (
                   (start_at IS NOT NULL AND start_at >= $2 AND start_at < $3)
                   OR start_at IS NULL
                 )"#,
            household_id,
            start,
            end,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for activity in activities {
            let recurrence = self.fetch_recurrence(activity.id).await?;
            result.push(LocalActivityWithRecurrence::new(activity, recurrence));
        }
        Ok(result)
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<LocalActivity>> {
        let activity = sqlx::query_as!(
            LocalActivity,
            "SELECT id, household_id, person_id, title, description, color, start_at, end_at, is_all_day, created_at, updated_at FROM family_center.local_activities WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(activity)
    }

    async fn create(
        &self,
        household_id: Uuid,
        input: CreateLocalActivity,
    ) -> anyhow::Result<LocalActivityWithRecurrence> {
        let id = Uuid::new_v4();

        let activity = sqlx::query_as!(
            LocalActivity,
            r#"INSERT INTO family_center.local_activities (id, household_id, person_id, title, description, color, start_at, end_at, is_all_day)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
               RETURNING id, household_id, person_id, title, description, color, start_at, end_at, is_all_day, created_at, updated_at"#,
            id,
            household_id,
            input.person_id,
            input.title,
            input.description,
            input.color,
            input.start_at,
            input.end_at,
            input.is_all_day.unwrap_or(false),
        )
        .fetch_one(&self.pool)
        .await?;

        let recurrence = if let Some(rec) = input.recurrence {
            let r = sqlx::query_as!(
                LocalActivityRecurrence,
                r#"INSERT INTO family_center.local_activity_recurrences (id, local_activity_id, freq, interval_val, by_day_of_week, by_day_of_month, until, count)
                   VALUES ($1, $2, $3, $4, $5, $6, $7::date, $8)
                   RETURNING id, local_activity_id, freq, interval_val, by_day_of_week, by_day_of_month, until, count, created_at, updated_at"#,
                Uuid::new_v4(),
                id,
                rec.freq,
                rec.interval.unwrap_or(1),
                rec.by_day_of_week.as_deref(),
                rec.by_day_of_month.as_deref(),
                rec.until.as_deref().map(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d")).transpose()?,
                rec.count,
            )
            .fetch_one(&self.pool)
            .await?;
            Some(r)
        } else {
            None
        };

        Ok(LocalActivityWithRecurrence::new(activity, recurrence))
    }

    async fn update(
        &self,
        id: Uuid,
        input: UpdateLocalActivity,
    ) -> anyhow::Result<LocalActivityWithRecurrence> {
        let activity = sqlx::query_as!(
            LocalActivity,
            r#"UPDATE family_center.local_activities SET
                person_id = COALESCE($2, person_id),
                title = COALESCE($3, title),
                description = COALESCE($4, description),
                color = COALESCE($5, color),
                start_at = COALESCE($6, start_at),
                end_at = COALESCE($7, end_at),
                is_all_day = COALESCE($8, is_all_day),
                updated_at = NOW()
               WHERE id = $1
               RETURNING id, household_id, person_id, title, description, color, start_at, end_at, is_all_day, created_at, updated_at"#,
            id,
            input.person_id.flatten(),
            input.title,
            input.description.flatten(),
            input.color.flatten(),
            input.start_at.flatten(),
            input.end_at.flatten(),
            input.is_all_day,
        )
        .fetch_one(&self.pool)
        .await?;

        let recurrence = match input.recurrence {
            Some(Some(rec)) => {
                let r = sqlx::query_as!(
                    LocalActivityRecurrence,
                    r#"INSERT INTO family_center.local_activity_recurrences (id, local_activity_id, freq, interval_val, by_day_of_week, by_day_of_month, until, count)
                       VALUES ($1, $2, $3, $4, $5, $6, $7::date, $8)
                       ON CONFLICT (local_activity_id) DO UPDATE SET
                           freq = EXCLUDED.freq,
                           interval_val = EXCLUDED.interval_val,
                           by_day_of_week = EXCLUDED.by_day_of_week,
                           by_day_of_month = EXCLUDED.by_day_of_month,
                           until = EXCLUDED.until,
                           count = EXCLUDED.count,
                           updated_at = NOW()
                       RETURNING id, local_activity_id, freq, interval_val, by_day_of_week, by_day_of_month, until, count, created_at, updated_at"#,
                    Uuid::new_v4(),
                    id,
                    rec.freq,
                    rec.interval.unwrap_or(1),
                    rec.by_day_of_week.as_deref(),
                    rec.by_day_of_month.as_deref(),
                    rec.until.as_deref().map(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d")).transpose()?,
                    rec.count,
                )
                .fetch_one(&self.pool)
                .await?;
                Some(r)
            }
            Some(None) => {
                sqlx::query!(
                    "DELETE FROM family_center.local_activity_recurrences WHERE local_activity_id = $1",
                    id
                )
                .execute(&self.pool)
                .await?;
                None
            }
            None => self.fetch_recurrence(id).await?,
        };

        Ok(LocalActivityWithRecurrence::new(activity, recurrence))
    }
}
