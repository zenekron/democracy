use base64::Engine;
use chrono::{DateTime, Duration, Utc};
use serenity::{
    builder::CreateInteractionResponse,
    model::prelude::{component::ButtonStyle, GuildId, UserId},
    prelude::Context,
};
use sqlx::{postgres::types::PgInterval, PgPool};
use uuid::Uuid;

use crate::{
    error::Error,
    util::{colors, emojis, ProgressBar},
};

use super::{InvitePollStatus, InvitePollVoteCount};

static BASE64: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD_NO_PAD;

#[derive(Debug, sqlx::FromRow)]
pub struct InvitePoll {
    pub id: Uuid,
    guild_id: i64,
    user_id: i64,
    pub status: InvitePollStatus,
    pub ends_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl InvitePoll {
    pub async fn create(
        pool: &PgPool,
        guild_id: GuildId,
        user_id: UserId,
        duration: Duration,
    ) -> Result<Self, Error> {
        let duration = PgInterval::try_from(duration).map_err(sqlx::Error::Decode)?;

        let res = sqlx::query_as!(
            Self,
            r#"
                INSERT INTO invite_poll(guild_id, user_id, ends_at)
                VALUES ($1, $2, now() + $3)
                RETURNING
                    id, guild_id AS "guild_id: _",
                    user_id AS "user_id: _",
                    status AS "status: _",
                    ends_at,
                    created_at,
                    updated_at
                ;
            "#,
            guild_id.0 as i64,
            user_id.0 as i64,
            duration
        )
        .fetch_one(pool)
        .await?;

        Ok(res)
    }

    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<Option<Self>, Error> {
        let res = sqlx::query_as!(
            Self,
            r#"
                SELECT
                    id, guild_id AS "guild_id: _",
                    user_id AS "user_id: _",
                    status AS "status: _",
                    ends_at,
                    created_at,
                    updated_at
                FROM invite_poll
                WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(res)
    }

    pub async fn find_pending_with_count(pool: &PgPool) -> Result<Vec<InvitePollWithCount>, Error> {
        let res = sqlx::query_as::<_, InvitePollWithCount>(
            r#"
                SELECT IP.*, IPVC.*
                FROM invite_poll IP
                    INNER JOIN invite_poll_vote_count IPVC ON IP.id = IPVC.invite_poll_id
                WHERE IP.status = 'open' AND IP.ends_at < now();
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(res)
    }

    pub fn decode_id(id: &str) -> Result<Uuid, Error> {
        let id = id
            .strip_prefix('`')
            .unwrap_or(id)
            .strip_suffix('`')
            .unwrap_or(id);

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

    pub async fn create_interaction_response(
        &self,
        ctx: Context,
        pool: &PgPool,
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
        let guild = self.guild_id().to_partial_guild(&ctx.http).await?;
        let count = InvitePollVoteCount::compute(pool, &self.id).await?;

        let max = {
            let mut max = 0;
            let mut last_user_id: Option<UserId> = None;

            loop {
                let members = guild.members(&ctx.http, None, last_user_id).await?;
                if members.len() == 0 {
                    break;
                }
                max += members.iter().filter(|m| m.user.bot == false).count() as u64;
                last_user_id = members.last().map(|u| u.user.id);
            }

            max
        };

        let votes = {
            let mut bar = ProgressBar::builder();
            bar.max(max).with_count(true).with_percentage(true);

            format!(
                "{} {}\n{} {}\n{} {}",
                emojis::LARGE_GREEN_CIRCLE,
                bar.value(count.yes_count as u64).build().unwrap(),
                emojis::LARGE_YELLOW_CIRCLE,
                bar.value(count.maybe_count as u64).build().unwrap(),
                emojis::LARGE_RED_CIRCLE,
                bar.value(count.no_count as u64).build().unwrap(),
            )
        };

        Ok(Box::new(move |resp| {
            resp.interaction_response_data(|data| {
                data.embed(|embed| {
                    embed
                        .color(match self.status {
                            InvitePollStatus::Open => colors::PASTEL_GREEN,
                            InvitePollStatus::Closed => colors::PASTEL_RED,
                        })
                        .title("Invite Poll")
                        .thumbnail(user.face())
                        .field(
                            "Poll Id",
                            ["`", self.encoded_id().as_str(), "`"].concat(),
                            true,
                        )
                        .field(
                            "Status",
                            match self.status {
                                InvitePollStatus::Open => {
                                    emojis::LARGE_GREEN_CIRCLE.to_string() + " Open"
                                }
                                InvitePollStatus::Closed => {
                                    emojis::LARGE_RED_CIRCLE.to_string() + " Closed"
                                }
                            },
                            true,
                        )
                        .field("", "", true)
                        .field("User", &user.name, true)
                        .field("Votes", votes, false)
                })
                .components(|component| match self.status {
                    InvitePollStatus::Open => component.create_action_row(|row| {
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
                    }),
                    InvitePollStatus::Closed => component,
                })
            })
        }))
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct InvitePollWithCount {
    #[sqlx(flatten)]
    pub invite_poll: InvitePoll,

    #[sqlx(flatten)]
    pub count: InvitePollVoteCount,
}
