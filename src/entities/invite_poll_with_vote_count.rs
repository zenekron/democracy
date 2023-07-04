use serenity::{
    builder::CreateInteractionResponse,
    model::prelude::{component::ButtonStyle, UserId},
    prelude::Context,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::Error,
    util::{colors, emojis, ProgressBar},
};

use super::{InvitePoll, InvitePollStatus};

#[derive(Debug, sqlx::FromRow)]
pub struct InvitePollWithVoteCount {
    #[sqlx(flatten)]
    pub invite_poll: InvitePoll,

    pub yes_count: i64,
    pub maybe_count: i64,
    pub no_count: i64,
}

///
/// sql
///
impl InvitePollWithVoteCount {
    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<Option<Self>, Error> {
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
                WHERE status = 'open' AND ends_at <= now();
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
                        .color(match self.invite_poll.status {
                            InvitePollStatus::Open => colors::PASTEL_GREEN,
                            InvitePollStatus::Closed => colors::PASTEL_RED,
                        })
                        .title("Invite Poll")
                        .thumbnail(user.face())
                        .field(
                            "Poll Id",
                            ["`", self.invite_poll.encoded_id().as_str(), "`"].concat(),
                            true,
                        )
                        .field(
                            "Status",
                            match self.invite_poll.status {
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
                .components(|component| match self.invite_poll.status {
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
