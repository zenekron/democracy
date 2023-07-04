use serenity::{
    builder::CreateInteractionResponse, model::prelude::component::ButtonStyle, prelude::Context,
};

use crate::{
    error::Error,
    util::{colors, emojis, ProgressBar},
    POOL,
};

use super::{InvitePoll, InvitePollId, InvitePollOutcome};

#[derive(Debug, sqlx::FromRow)]
pub struct InvitePollWithVoteCount {
    #[sqlx(flatten)]
    pub invite_poll: InvitePoll,

    pub yes_count: i64,
    pub maybe_count: i64,
    pub no_count: i64,
}

impl InvitePollWithVoteCount {
    pub async fn find_by_id(id: &InvitePollId) -> Result<Option<Self>, Error> {
        let pool = POOL.get().expect("the Pool to be initialized");
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

    pub async fn find_expired() -> Result<Vec<Self>, Error> {
        let pool = POOL.get().expect("the Pool to be initialized");

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
                        .field(
                            "Votes",
                            {
                                let mut bar = ProgressBar::builder();
                                bar.max((self.yes_count + self.maybe_count + self.no_count) as u64)
                                    .with_count(true)
                                    .with_percentage(true);

                                format!(
                                    "{} {}\n{} {}\n{} {}",
                                    emojis::LARGE_GREEN_CIRCLE,
                                    bar.value(self.yes_count as u64).build().unwrap(),
                                    emojis::LARGE_YELLOW_CIRCLE,
                                    bar.value(self.maybe_count as u64).build().unwrap(),
                                    emojis::LARGE_RED_CIRCLE,
                                    bar.value(self.no_count as u64).build().unwrap(),
                                )
                            },
                            false,
                        )
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
