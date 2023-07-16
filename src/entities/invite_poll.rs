use std::{fmt::Display, str::FromStr, time::Duration};

use base64::{display::Base64Display, Engine};
use chrono::{DateTime, Utc};
use serenity::model::prelude::Message;
use sqlx::{postgres::types::PgInterval, Executor, PgExecutor, Postgres};
use uuid::Uuid;

use crate::{
    error::Error,
    util::serenity::{ChannelId, GuildId, MessageId, UserId},
};

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
            .map_err(|err| Error::InvitePollIdInvalid(s.to_owned(), Box::new(err)))?;

        let id = Uuid::from_slice(&buf)
            .map_err(|err| Error::InvitePollIdInvalid(s.to_owned(), Box::new(err)))?;

        Ok(Self(id))
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct InvitePoll {
    pub id: InvitePollId,
    pub guild_id: GuildId,
    pub inviter: UserId,
    pub invitee: UserId,
    pub channel_id: Option<ChannelId>,
    pub message_id: Option<MessageId>,
    pub outcome: Option<InvitePollOutcome>,
    pub message: Option<String>,
    pub ends_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl InvitePoll {
    pub async fn create<'e, E>(
        executor: E,
        guild_id: &GuildId,
        inviter: &UserId,
        invitee: &UserId,
        duration: &Duration,
    ) -> Result<Self, Error>
    where
        E: PgExecutor<'e>,
    {
        let duration = PgInterval::try_from(*duration).map_err(sqlx::Error::Decode)?;

        let res = sqlx::query_as::<_, Self>(
            r#"
                INSERT INTO invite_poll (guild_id, inviter, invitee, ends_at)
                VALUES ($1, $2, $3, now() + $4)
                RETURNING *;
            "#,
        )
        .bind(guild_id)
        .bind(inviter)
        .bind(invitee)
        .bind(duration)
        .fetch_one(executor)
        .await?;

        Ok(res)
    }

    pub async fn update_message<'e, E>(
        &mut self,
        executor: E,
        message: &Message,
    ) -> Result<(), Error>
    where
        E: PgExecutor<'e>,
    {
        let res = sqlx::query_as::<_, Self>(
            r#"
                UPDATE invite_poll
                SET channel_id = $2, message_id = $3
                WHERE id = $1
                RETURNING *;
            "#,
        )
        .bind(&self.id)
        .bind(ChannelId::from(message.channel_id))
        .bind(MessageId::from(message.id))
        .fetch_one(executor)
        .await?;

        *self = res;
        Ok(())
    }

    pub async fn close<'c, E>(
        &mut self,
        executor: E,
        outcome: InvitePollOutcome,
        message: Option<String>,
    ) -> Result<(), Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        let res = sqlx::query_as::<_, Self>(
            r#"
                UPDATE invite_poll
                SET outcome = $2, message = $3
                WHERE id = $1
                RETURNING *;
            "#,
        )
        .bind(&self.id)
        .bind(outcome)
        .bind(message)
        .fetch_one(executor)
        .await?;

        *self = res;
        Ok(())
    }
}
