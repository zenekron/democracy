use chrono::{DateTime, Utc};
use sqlx::{Executor, Postgres};

use crate::{
    error::Error,
    util::serenity::{ChannelId, GuildId},
    POOL,
};

#[derive(Debug, sqlx::FromRow)]
pub struct Guild {
    pub id: GuildId,
    pub invite_channel_id: ChannelId,
    pub vote_success_threshold: f32,
    pub max_maybe_votes_threshold: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Guild {
    pub async fn create<'c, E>(
        executor: E,
        id: &GuildId,
        invite_channel_id: &ChannelId,
        vote_success_threshold: f32,
        max_maybe_votes_threshold: f32,
    ) -> Result<Self, Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        let res = sqlx::query_as::<_, Self>(
            r#"
                INSERT INTO guild (id, invite_channel_id, vote_success_threshold, max_maybe_votes_threshold)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (id) DO UPDATE SET
                    invite_channel_id = EXCLUDED.invite_channel_id,
                    vote_success_threshold = EXCLUDED.vote_success_threshold,
                    max_maybe_votes_threshold = EXCLUDED.max_maybe_votes_threshold
                RETURNING *;
            "#,
        )
        .bind(id)
        .bind(invite_channel_id)
        .bind(vote_success_threshold)
        .bind(max_maybe_votes_threshold)
        .fetch_one(executor)
        .await?;

        Ok(res)
    }

    pub async fn find_by_id(id: &GuildId) -> Result<Option<Self>, Error> {
        let pool = POOL.get().expect("the Pool to be initialized");

        let res = sqlx::query_as::<_, Self>(
            r#"
                SELECT *
                FROM guild
                WHERE id = $1;
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(res)
    }
}
