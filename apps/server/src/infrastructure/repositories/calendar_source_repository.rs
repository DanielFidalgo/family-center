use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::calendar_source::{CalendarSelection, CalendarSource};
use crate::domain::repositories::calendar_source_repository::{
    CalendarSourceWithToken, ICalendarSourceRepository,
};

pub struct CalendarSourceRepository {
    pool: PgPool,
}

impl CalendarSourceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ICalendarSourceRepository for CalendarSourceRepository {
    async fn find_by_account(
        &self,
        google_account_id: Uuid,
    ) -> anyhow::Result<Vec<CalendarSource>> {
        let calendars = sqlx::query_as!(
            CalendarSource,
            "SELECT id, google_account_id, calendar_id, name, description, color_hex, is_selected, access_role, created_at, updated_at FROM family_center.calendar_sources WHERE google_account_id = $1 ORDER BY name",
            google_account_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(calendars)
    }

    async fn create_mock_calendars(&self, google_account_id: Uuid) -> anyhow::Result<()> {
        sqlx::query!(
            r#"INSERT INTO family_center.calendar_sources (id, google_account_id, calendar_id, name, is_selected, access_role)
               VALUES
                 ($1, $2, 'mock-personal', 'Personal', TRUE, 'owner'),
                 ($3, $2, 'mock-family', 'Family', TRUE, 'writer'),
                 ($4, $2, 'mock-work', 'Work', FALSE, 'reader')
               ON CONFLICT (google_account_id, calendar_id) DO NOTHING"#,
            Uuid::new_v4(),
            google_account_id,
            Uuid::new_v4(),
            Uuid::new_v4(),
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_selection(
        &self,
        sel: &CalendarSelection,
    ) -> anyhow::Result<Option<CalendarSource>> {
        let cal = sqlx::query_as!(
            CalendarSource,
            "UPDATE family_center.calendar_sources SET is_selected = $2, updated_at = NOW() WHERE id = $1 RETURNING id, google_account_id, calendar_id, name, description, color_hex, is_selected, access_role, created_at, updated_at",
            sel.calendar_source_id,
            sel.is_selected,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(cal)
    }

    async fn find_selected_with_tokens(&self) -> anyhow::Result<Vec<CalendarSourceWithToken>> {
        let rows = sqlx::query!(
            "SELECT cs.id, cs.google_account_id, ga.access_token, ga.refresh_token FROM family_center.calendar_sources cs JOIN family_center.google_accounts ga ON cs.google_account_id = ga.id WHERE cs.is_selected = TRUE"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| CalendarSourceWithToken {
                id: r.id,
                google_account_id: r.google_account_id,
                access_token: r.access_token,
                refresh_token: r.refresh_token,
            })
            .collect())
    }

    async fn find_selected_with_tokens_by_ids(
        &self,
        ids: &[Uuid],
    ) -> anyhow::Result<Vec<CalendarSourceWithToken>> {
        let rows = sqlx::query!(
            "SELECT cs.id, cs.google_account_id, ga.access_token, ga.refresh_token FROM family_center.calendar_sources cs JOIN family_center.google_accounts ga ON cs.google_account_id = ga.id WHERE cs.id = ANY($1) AND cs.is_selected = TRUE",
            ids
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| CalendarSourceWithToken {
                id: r.id,
                google_account_id: r.google_account_id,
                access_token: r.access_token,
                refresh_token: r.refresh_token,
            })
            .collect())
    }
}
