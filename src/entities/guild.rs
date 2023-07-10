use chrono::{DateTime, Utc};
use sqlx::{Executor, Postgres};

use crate::{
    error::Error,
    util::serenity::{ChannelId, GuildId},
};

#[derive(Debug, sqlx::FromRow)]
pub struct Guild {
    pub id: GuildId,
    /// The id of the channel users are going to be invited to.
    pub invite_channel_id: ChannelId,
    /// The minimum number of votes required to consider a vote valid (0.0 - 1.0).
    pub invite_poll_quorum: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Guild {
    pub async fn create_or_update<'c, E>(
        executor: E,
        id: &GuildId,
        invite_channel_id: &ChannelId,
        invite_poll_quorum: f32,
    ) -> Result<Self, Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        let res = sqlx::query_as::<_, Self>(
            r#"
                INSERT INTO guild (id, invite_channel_id, invite_poll_quorum)
                VALUES ($1, $2, $3)
                ON CONFLICT (id) DO UPDATE SET
                    invite_channel_id = EXCLUDED.invite_channel_id,
                    invite_poll_quorum = EXCLUDED.invite_poll_quorum
                RETURNING *;
            "#,
        )
        .bind(id)
        .bind(invite_channel_id)
        .bind(invite_poll_quorum)
        .fetch_one(executor)
        .await?;

        Ok(res)
    }

    pub async fn find_by_id<'c, E>(executor: E, id: &GuildId) -> Result<Option<Self>, Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        let res = sqlx::query_as::<_, Self>(
            r#"
                SELECT *
                FROM guild
                WHERE id = $1;
            "#,
        )
        .bind(id)
        .fetch_optional(executor)
        .await?;

        Ok(res)
    }
}
