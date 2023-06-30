use chrono::{DateTime, Utc};
use serenity::model::prelude::{GuildId, UserId};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::Error;

#[derive(Debug, sqlx::FromRow)]
pub struct Invite {
    pub id: Uuid,
    guild_id: i64,
    user_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Invite {
    pub async fn create(pool: &PgPool, guild_id: GuildId, user_id: UserId) -> Result<Self, Error> {
        let res = sqlx::query_as!(
            Invite,
            r#"
                INSERT INTO invite(guild_id, user_id)
                VALUES ($1, $2)
                RETURNING id, guild_id AS "guild_id: _", user_id AS "user_id: _", created_at, updated_at;
            "#,
            guild_id.0 as i64,
            user_id.0 as i64
        ).fetch_one(pool).await?;

        Ok(res)
    }
}

impl Invite {
    pub fn guild_id(&self) -> GuildId {
        GuildId(self.guild_id as u64)
    }

    pub fn user_id(&self) -> UserId {
        UserId(self.user_id as u64)
    }
}
