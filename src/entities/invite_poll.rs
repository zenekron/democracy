use std::fmt::Display;

use base64::Engine;
use chrono::{DateTime, Utc};
use serenity::{
    builder::CreateInteractionResponse,
    model::prelude::{component::ButtonStyle, GuildId, UserId},
    prelude::Context,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::Error,
    util::{colors, emojis, ProgressBar},
};

use super::{invite_poll_vote_submission::InvitePollVoteSubmission, InvitePollVote};

static BASE64: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD_NO_PAD;

#[derive(Debug, sqlx::FromRow)]
pub struct InvitePoll {
    pub id: Uuid,
    guild_id: i64,
    user_id: i64,
    pub yes_count: i32,
    pub maybe_count: i32,
    pub no_count: i32,
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
                RETURNING id, guild_id AS "guild_id: _", user_id AS "user_id: _", yes_count, maybe_count, no_count, created_at, updated_at;
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
                SELECT id, guild_id AS "guild_id: _", user_id AS "user_id: _", yes_count, maybe_count, no_count, created_at, updated_at
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

    pub async fn submit_vote(
        &self,
        pool: &PgPool,
        user_id: &UserId,
        vote: InvitePollVote,
    ) -> Result<InvitePollVoteSubmission, Error> {
        InvitePollVoteSubmission::upsert(pool, self.id, user_id, vote).await
    }

    pub async fn create_interaction_response(
        &self,
        ctx: Context,
    ) -> Result<
        Box<
            dyn for<'a, 'b> FnOnce(
                    &'a mut CreateInteractionResponse<'b>,
                ) -> &'a mut CreateInteractionResponse<'b>
                + Send
                + Sync
                + '_,
        >,
        Error,
    > {
        let user = self.user_id().to_user(&ctx.http).await?;

        Ok(Box::new(move |resp| {
            resp.interaction_response_data(|data| {
                data.embed(|embed| {
                    embed
                        .color(colors::PASTEL_GREEN)
                        .title("Invite Poll")
                        .thumbnail(user.face())
                        .field("Poll Id", self.encoded_id(), true)
                        .field("User", &user.name, true)
                        .field("Results", self, false)
                })
                .components(|component| {
                    component.create_action_row(|row| {
                        row.create_button(|btn| {
                            btn.custom_id("democracy.invite-poll-vote.yes")
                                .label("Yes")
                                .style(ButtonStyle::Success)
                        })
                        .create_button(|btn| {
                            btn.custom_id("democracy.invite-poll-vote.maybe")
                                .label("Maybe")
                                .style(ButtonStyle::Primary)
                        })
                        .create_button(|btn| {
                            btn.custom_id("democracy.invite-poll-vote.no")
                                .label("No")
                                .style(ButtonStyle::Danger)
                        })
                    })
                })
            })
        }))
    }
}

impl Display for InvitePoll {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max = 2_f32;

        writeln!(
            f,
            "{} {} [{} · {:.0}%]",
            emojis::LARGE_GREEN_CIRCLE,
            ProgressBar(self.yes_count as f32 / max),
            self.yes_count,
            self.yes_count as f32 / max * 100.0
        )?;
        writeln!(
            f,
            "{} {} [{} · {:.0}%]",
            emojis::LARGE_YELLOW_CIRCLE,
            ProgressBar(self.maybe_count as f32 / max),
            self.maybe_count,
            self.maybe_count as f32 / max * 100.0
        )?;
        writeln!(
            f,
            "{} {} [{} · {:.0}%]",
            emojis::LARGE_RED_CIRCLE,
            ProgressBar(self.no_count as f32 / max),
            self.no_count,
            self.no_count as f32 / max * 100.0
        )?;

        Ok(())
    }
}
