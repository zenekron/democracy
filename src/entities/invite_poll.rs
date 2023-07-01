use base64::Engine;
use chrono::{DateTime, Utc};
use serenity::model::prelude::{GuildId, UserId};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::Error;

static BASE64: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD_NO_PAD;

#[derive(Debug, sqlx::FromRow)]
pub struct InvitePoll {
    pub id: Uuid,
    guild_id: i64,
    user_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl InvitePoll {
    pub async fn create(pool: &PgPool, guild_id: GuildId, user_id: UserId) -> Result<Self, Error> {
        let res = sqlx::query_as!(
            Self,
            r#"
                INSERT INTO invite_poll(guild_id, user_id)
                VALUES ($1, $2)
                RETURNING id, guild_id AS "guild_id: _", user_id AS "user_id: _", created_at, updated_at;
            "#,
            guild_id.0 as i64,
            user_id.0 as i64
        ).fetch_one(pool).await?;

        Ok(res)
    }

    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<Option<Self>, Error> {
        let res = sqlx::query_as!(
            Self,
            r#"
                SELECT id, guild_id AS "guild_id: _", user_id AS "user_id: _", created_at, updated_at
                FROM invite_poll
                WHERE id = $1
            "#,
            id
        ).fetch_optional(pool).await?;

        Ok(res)
    }

    pub fn decode_id(id: &str) -> Result<Uuid, Error> {
        let buf = BASE64
            .decode(id)
            .map_err(|err| Error::InvitePollIdInvalid(id.to_owned(), err.into()))?;

        let id = Uuid::from_slice(buf.as_slice())
            .map_err(|err| Error::InvitePollIdInvalid(id.to_owned(), err.into()))?;

        Ok(id)
    }

    pub fn encoded_id(&self) -> String {
        BASE64.encode(self.id)
    }

    pub fn guild_id(&self) -> GuildId {
        GuildId(self.guild_id as u64)
    }

    pub fn user_id(&self) -> UserId {
        UserId(self.user_id as u64)
    }
}
