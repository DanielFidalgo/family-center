use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::entities::local_activity::{
    ActivityCompletion, CreateLocalActivity, LocalActivity, LocalActivityRecurrence,
    LocalActivityWithRecurrence, UpdateLocalActivity,
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
        let rows = sqlx::query(
            "SELECT id, household_id, person_id, title, description, color, start_at, end_at, is_all_day, category, is_time_bound, created_at, updated_at FROM family_center.local_activities WHERE household_id = $1 ORDER BY start_at NULLS LAST, title"
        )
        .bind(household_id)
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            let activity = LocalActivity {
                id: row.get("id"),
                household_id: row.get("household_id"),
                person_id: row.get("person_id"),
                title: row.get("title"),
                description: row.get("description"),
                color: row.get("color"),
                start_at: row.get("start_at"),
                end_at: row.get("end_at"),
                is_all_day: row.get("is_all_day"),
                category: row.get("category"),
                is_time_bound: row.get("is_time_bound"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
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
        let rows = sqlx::query(
            r#"SELECT id, household_id, person_id, title, description, color, start_at, end_at, is_all_day, category, is_time_bound, created_at, updated_at
               FROM family_center.local_activities
               WHERE household_id = $1
                 AND (
                   (start_at IS NOT NULL AND start_at >= $2 AND start_at < $3)
                   OR start_at IS NULL
                 )"#,
        )
        .bind(household_id)
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for row in rows {
            let activity = LocalActivity {
                id: row.get("id"),
                household_id: row.get("household_id"),
                person_id: row.get("person_id"),
                title: row.get("title"),
                description: row.get("description"),
                color: row.get("color"),
                start_at: row.get("start_at"),
                end_at: row.get("end_at"),
                is_all_day: row.get("is_all_day"),
                category: row.get("category"),
                is_time_bound: row.get("is_time_bound"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            let recurrence = self.fetch_recurrence(activity.id).await?;
            result.push(LocalActivityWithRecurrence::new(activity, recurrence));
        }
        Ok(result)
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<LocalActivity>> {
        let row = sqlx::query(
            "SELECT id, household_id, person_id, title, description, color, start_at, end_at, is_all_day, category, is_time_bound, created_at, updated_at FROM family_center.local_activities WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| LocalActivity {
            id: r.get("id"),
            household_id: r.get("household_id"),
            person_id: r.get("person_id"),
            title: r.get("title"),
            description: r.get("description"),
            color: r.get("color"),
            start_at: r.get("start_at"),
            end_at: r.get("end_at"),
            is_all_day: r.get("is_all_day"),
            category: r.get("category"),
            is_time_bound: r.get("is_time_bound"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn create(
        &self,
        household_id: Uuid,
        input: CreateLocalActivity,
    ) -> anyhow::Result<LocalActivityWithRecurrence> {
        let id = Uuid::new_v4();

        let row = sqlx::query(
            r#"INSERT INTO family_center.local_activities (id, household_id, person_id, title, description, color, start_at, end_at, is_all_day, category, is_time_bound)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
               RETURNING id, household_id, person_id, title, description, color, start_at, end_at, is_all_day, category, is_time_bound, created_at, updated_at"#,
        )
        .bind(id)
        .bind(household_id)
        .bind(input.person_id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(&input.color)
        .bind(input.start_at)
        .bind(input.end_at)
        .bind(input.is_all_day.unwrap_or(false))
        .bind(&input.category)
        .bind(input.is_time_bound.unwrap_or(false))
        .fetch_one(&self.pool)
        .await?;

        let activity = LocalActivity {
            id: row.get("id"),
            household_id: row.get("household_id"),
            person_id: row.get("person_id"),
            title: row.get("title"),
            description: row.get("description"),
            color: row.get("color"),
            start_at: row.get("start_at"),
            end_at: row.get("end_at"),
            is_all_day: row.get("is_all_day"),
            category: row.get("category"),
            is_time_bound: row.get("is_time_bound"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

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
        let row = sqlx::query(
            r#"UPDATE family_center.local_activities SET
                person_id = COALESCE($2, person_id),
                title = COALESCE($3, title),
                description = COALESCE($4, description),
                color = COALESCE($5, color),
                start_at = COALESCE($6, start_at),
                end_at = COALESCE($7, end_at),
                is_all_day = COALESCE($8, is_all_day),
                category = COALESCE($9, category),
                is_time_bound = COALESCE($10, is_time_bound),
                updated_at = NOW()
               WHERE id = $1
               RETURNING id, household_id, person_id, title, description, color, start_at, end_at, is_all_day, category, is_time_bound, created_at, updated_at"#,
        )
        .bind(id)
        .bind(input.person_id.flatten())
        .bind(input.title)
        .bind(input.description.flatten())
        .bind(input.color.flatten())
        .bind(input.start_at.flatten())
        .bind(input.end_at.flatten())
        .bind(input.is_all_day)
        .bind(input.category.flatten())
        .bind(input.is_time_bound)
        .fetch_one(&self.pool)
        .await?;

        let activity = LocalActivity {
            id: row.get("id"),
            household_id: row.get("household_id"),
            person_id: row.get("person_id"),
            title: row.get("title"),
            description: row.get("description"),
            color: row.get("color"),
            start_at: row.get("start_at"),
            end_at: row.get("end_at"),
            is_all_day: row.get("is_all_day"),
            category: row.get("category"),
            is_time_bound: row.get("is_time_bound"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

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

    async fn complete_activity(
        &self,
        activity_id: Uuid,
        date: NaiveDate,
        completed_by: Option<Uuid>,
    ) -> anyhow::Result<ActivityCompletion> {
        let row = sqlx::query(
            r#"INSERT INTO family_center.activity_completions (local_activity_id, completed_date, completed_by)
               VALUES ($1, $2, $3)
               ON CONFLICT (local_activity_id, completed_date) DO UPDATE SET completed_by = EXCLUDED.completed_by
               RETURNING id, local_activity_id, completed_date, completed_by, created_at"#,
        )
        .bind(activity_id)
        .bind(date)
        .bind(completed_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(ActivityCompletion {
            id: row.get("id"),
            local_activity_id: row.get("local_activity_id"),
            completed_date: row.get("completed_date"),
            completed_by: row.get("completed_by"),
            created_at: row.get("created_at"),
        })
    }

    async fn uncomplete_activity(
        &self,
        activity_id: Uuid,
        date: NaiveDate,
    ) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM family_center.activity_completions WHERE local_activity_id = $1 AND completed_date = $2")
            .bind(activity_id)
            .bind(date)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_completions_in_range(
        &self,
        household_id: Uuid,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<ActivityCompletion>> {
        let rows = sqlx::query(
            r#"SELECT ac.id, ac.local_activity_id, ac.completed_date, ac.completed_by, ac.created_at
               FROM family_center.activity_completions ac
               JOIN family_center.local_activities la ON la.id = ac.local_activity_id
               WHERE la.household_id = $1
                 AND ac.completed_date >= $2
                 AND ac.completed_date <= $3
               ORDER BY ac.completed_date"#,
        )
        .bind(household_id)
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|r| ActivityCompletion {
                id: r.get("id"),
                local_activity_id: r.get("local_activity_id"),
                completed_date: r.get("completed_date"),
                completed_by: r.get("completed_by"),
                created_at: r.get("created_at"),
            })
            .collect())
    }
}
