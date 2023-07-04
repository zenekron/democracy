use serenity::{
    builder::CreateInteractionResponse,
    model::prelude::{component::ButtonStyle, UserId},
    prelude::Context,
};
use sqlx::PgPool;

use crate::{
    error::Error,
    util::{colors, emojis, ProgressBar},
};

use super::{InvitePoll, InvitePollId, InvitePollOutcome};

const VOTE_THRESHOLD: f32 = 0.8;

#[derive(Debug, sqlx::FromRow)]
pub struct InvitePollWithVoteCount {
    #[sqlx(flatten)]
    pub invite_poll: InvitePoll,

    pub yes_count: i64,
    pub maybe_count: i64,
    pub no_count: i64,
}

impl InvitePollWithVoteCount {
    pub async fn outcome(&self, ctx: &Context) -> Result<InvitePollOutcome, Error> {
        match self.invite_poll.outcome {
            Some(val) => Ok(val),
            None => {
                let user_count = self.user_count(ctx).await?;
                let required_votes = (user_count as f32 * VOTE_THRESHOLD).ceil() as i64;

                if self.no_count == 0 && (self.yes_count + self.maybe_count) >= required_votes {
                    Ok(InvitePollOutcome::Allow)
                } else {
                    Ok(InvitePollOutcome::Deny)
                }
            }
        }
    }

    async fn user_count(&self, ctx: &Context) -> Result<u64, Error> {
        let guild = self
            .invite_poll
            .guild_id()
            .to_partial_guild(&ctx.http)
            .await?;

        let mut max = 0;
        let mut after: Option<UserId> = None;
        loop {
            let page = guild.members(&ctx.http, None, after).await?;
            if page.len() == 0 {
                break;
            }

            max += page.iter().filter(|m| m.user.bot == false).count() as u64;
            after = page.last().map(|u| u.user.id);
        }

        Ok(max)
    }
}

///
/// sql
///
impl InvitePollWithVoteCount {
    pub async fn find_by_id(pool: &PgPool, id: &InvitePollId) -> Result<Option<Self>, Error> {
        let res = sqlx::query_as::<_, Self>(
            r#"
                SELECT * FROM invite_poll_with_vote_count WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(res)
    }

    pub async fn find_expired(pool: &PgPool) -> Result<Vec<Self>, Error> {
        let res = sqlx::query_as::<_, Self>(
            r#"
                SELECT * FROM invite_poll_with_vote_count
                WHERE outcome IS NULL AND ends_at <= now();
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(res)
    }
}

///
/// Discord
///
impl InvitePollWithVoteCount {
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
        let user = self.invite_poll.user_id().to_user(&ctx.http).await?;
        let guild = self
            .invite_poll
            .guild_id()
            .to_partial_guild(&ctx.http)
            .await?;

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
                bar.value(self.yes_count as u64).build().unwrap(),
                emojis::LARGE_YELLOW_CIRCLE,
                bar.value(self.maybe_count as u64).build().unwrap(),
                emojis::LARGE_RED_CIRCLE,
                bar.value(self.no_count as u64).build().unwrap(),
            )
        };

        Ok(Box::new(move |resp| {
            resp.interaction_response_data(|data| {
                data.embed(|embed| {
                    embed
                        .color(match self.invite_poll.outcome {
                            Some(InvitePollOutcome::Allow) => colors::DISCORD_GREEN,
                            Some(InvitePollOutcome::Deny) => colors::DISCORD_RED,
                            None => colors::DISCORD_BLURPLE,
                        })
                        .title("Invite Poll")
                        .thumbnail(user.face())
                        .field("Poll Id", format!("`{}`", self.invite_poll.id), true)
                        .field(
                            "Status",
                            match self.invite_poll.outcome {
                                Some(_) => emojis::LARGE_RED_CIRCLE.to_string() + " Closed",
                                None => emojis::LARGE_GREEN_CIRCLE.to_string() + " Open",
                            },
                            true,
                        )
                        .field("", "", true)
                        .field("User", &user.name, true)
                        .field("Votes", votes, false)
                })
                .components(|component| match self.invite_poll.outcome {
                    Some(_) => component,
                    None => component.create_action_row(|row| {
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
                })
            })
        }))
    }
}
