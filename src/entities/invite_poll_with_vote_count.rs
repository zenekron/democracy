use serenity::{
    builder::{CreateComponents, CreateEmbed},
    model::prelude::component::ButtonStyle,
    prelude::Context,
};

use crate::{
    error::Error,
    util::{colors, emojis, serenity::MessageRenderer, ProgressBar},
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
                SELECT *
                FROM invite_poll_with_vote_count
                WHERE id = $1;
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
                SELECT *
                FROM invite_poll_with_vote_count
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
    pub async fn create_renderer(
        &self,
        ctx: Context,
    ) -> Result<
        Box<
            dyn for<'m, 'a, 'b> FnOnce(
                    &'m mut MessageRenderer<'a, 'b>,
                ) -> &'m mut MessageRenderer<'a, 'b>
                + Send
                + Sync
                + '_,
        >,
        Error,
    > {
        let user = self.invite_poll.user_id.to_user(&ctx.http).await?;

        let embeds = vec![{
            let mut embed = CreateEmbed::default();

            embed
                .color(match self.invite_poll.outcome {
                    Some(_) => colors::DISCORD_RED,
                    None => colors::DISCORD_GREEN,
                })
                .title("Invite Poll")
                .thumbnail(user.face());

            // row 1
            embed
                .field("Poll Id", format!("`{}`", self.invite_poll.id), true)
                .field("User", &user.name, true)
                .field(
                    "Status",
                    match self.invite_poll.outcome {
                        Some(InvitePollOutcome::Allow) => [emojis::CHECK_MARK, " Allowed"].concat(),
                        Some(InvitePollOutcome::Deny) => [emojis::CROSS_MARK, " Denied"].concat(),
                        None => [emojis::LARGE_GREEN_CIRCLE, " Pending"].concat(),
                    },
                    true,
                );

            // row 2
            embed.field(
                "Votes",
                {
                    let mut bar = ProgressBar::builder();
                    bar.max(self.yes_count + self.maybe_count + self.no_count)
                        .with_count(true)
                        .with_percentage(true);

                    format!(
                        "{} {}\n{} {}\n{} {}",
                        emojis::LARGE_GREEN_CIRCLE,
                        bar.value(self.yes_count).build().unwrap(),
                        emojis::LARGE_YELLOW_CIRCLE,
                        bar.value(self.maybe_count).build().unwrap(),
                        emojis::LARGE_RED_CIRCLE,
                        bar.value(self.no_count).build().unwrap(),
                    )
                },
                false,
            );

            embed
        }];

        let components = match self.invite_poll.outcome {
            Some(_) => CreateComponents::default(),
            None => {
                let mut components = CreateComponents::default();
                components.create_action_row(|row| {
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
                });
                components
            }
        };

        Ok(Box::new(move |message| {
            message.set_embeds(embeds).set_components(components)
        }))
    }
}
