use sqlx::mysql::MySqlPool;

/// consumer の冪等化。SQS は at-least-once なので処理済みイベントIDを記録して二重処理を弾く
pub struct ProcessedEvents {
    pool: MySqlPool,
}

impl ProcessedEvents {
    #[must_use]
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn already_done(&self, event_id: i64) -> Result<bool, sqlx::Error> {
        let row =
            sqlx::query!("select event_id from processed_events where event_id = ?", event_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(row.is_some())
    }

    /// 二重に mark しても失敗しないよう insert ignore にする
    pub async fn mark(&self, event_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!("insert ignore into processed_events (event_id) values (?)", event_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
