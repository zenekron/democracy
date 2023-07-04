use std::{fmt::Display, str::FromStr};

use base64::{display::Base64Display, Engine};
use chrono::{DateTime, Duration, Utc};
use serenity::model::prelude::{GuildId, UserId};
use sqlx::postgres::types::PgInterval;
use uuid::Uuid;

use crate::{error::Error, POOL};

use super::InvitePollOutcome;

static BASE64: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD_NO_PAD;

#[derive(Clone, Debug, sqlx::Type)]
#[sqlx(transparent)]
pub struct InvitePollId(pub Uuid);

impl Display for InvitePollId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Base64Display::new(self.0.as_bytes(), &BASE64).fmt(f)
    }
}

impl FromStr for InvitePollId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let buf = BASE64
            .decode(s)
            .map_err(|err| Error::InvitePollIdInvalid(s.to_owned(), err.into()))?;

        let id = Uuid::from_slice(&buf)
            .map_err(|err| Error::InvitePollIdInvalid(s.to_owned(), err.into()))?;

        Ok(Self(id))
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct InvitePoll {
    pub id: InvitePollId,
    guild_id: i64,
    user_id: i64,
    pub outcome: Option<InvitePollOutcome>,
    pub ends_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl InvitePoll {
    pub async fn new(
        guild_id: GuildId,
        user_id: UserId,
        duration: Duration,
    ) -> Result<Self, Error> {
        let pool = POOL.get().expect("the Pool to be initialized");
        let duration = PgInterval::try_from(duration).map_err(sqlx::Error::Decode)?;

        let res = sqlx::query_as::<_, Self>(
            r#"
                INSERT INTO invite_poll(guild_id, user_id, ends_at)
                VALUES ($1, $2, now() + $3)
                RETURNING *;
            "#,
        )
        .bind(guild_id.0 as i64)
        .bind(user_id.0 as i64)
        .bind(duration)
        .fetch_one(pool)
        .await?;

        Ok(res)
    }

    pub fn guild_id(&self) -> GuildId {
        GuildId(self.guild_id as u64)
    }

    pub fn user_id(&self) -> UserId {
        UserId(self.user_id as u64)
    }

    pub async fn close(&mut self, outcome: InvitePollOutcome) -> Result<(), Error> {
        let pool = POOL.get().expect("the Pool to be initialized");

        let res = sqlx::query_as::<_, Self>(
            r#"
                UPDATE invite_poll
                SET outcome = $1
                WHERE id = $2
                RETURNING *;
            "#,
        )
        .bind(outcome)
        .bind(&self.id)
        .fetch_one(pool)
        .await?;

        let _ = std::mem::replace(self, res);

        Ok(())
    }
}
