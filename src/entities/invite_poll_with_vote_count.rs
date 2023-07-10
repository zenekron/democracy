use serenity::{
    builder::{CreateComponents, CreateEmbed},
    model::prelude::component::ButtonStyle,
    prelude::Context,
};
use sqlx::{Executor, Postgres};

use crate::{
    action::POLL_ID_FIELD_NAME,
    error::Error,
    util::{
        colors, emojis, serenity::MessageRenderer, DiscordTimestamp, DiscordTimestampStyle,
        ProgressBar,
    },
};

use super::{InvitePoll, InvitePollId, InvitePollOutcome};

#[derive(Debug, sqlx::FromRow)]
pub struct InvitePollWithVoteCount {
    #[sqlx(flatten)]
    pub invite_poll: InvitePoll,

    pub yes_count: i64,
    pub no_count: i64,
}

impl InvitePollWithVoteCount {
    pub async fn find_by_id<'c, E>(executor: E, id: &InvitePollId) -> Result<Option<Self>, Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        let res = sqlx::query_as::<_, Self>(
            r#"
                SELECT *
                FROM invite_poll_with_vote_count
                WHERE id = $1;
            "#,
        )
        .bind(id)
        .fetch_optional(executor)
        .await?;

        Ok(res)
    }

    pub async fn find_expired<'c, E>(executor: E) -> Result<Vec<Self>, Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        let res = sqlx::query_as::<_, Self>(
            r#"
                SELECT *
                FROM invite_poll_with_vote_count
                WHERE outcome IS NULL AND ends_at <= now();
            "#,
        )
        .fetch_all(executor)
        .await?;

        Ok(res)
    }

    pub async fn create_renderer(&self, ctx: Context) -> Result<MessageRenderer, Error> {
        let user = self.invite_poll.invitee.to_user(&ctx.http).await?;

        let embeds = vec![{
            let mut embed = CreateEmbed::default();

            embed
                .color(match self.invite_poll.outcome {
                    Some(InvitePollOutcome::Allow) => colors::DISCORD_GREEN,
                    Some(InvitePollOutcome::Deny) => colors::DISCORD_RED,
                    None => colors::DISCORD_BLURPLE,
                })
                .title("Invite Poll")
                .thumbnail(user.face());

            // row
            embed
                .field(
                    POLL_ID_FIELD_NAME,
                    format!("`{}`", self.invite_poll.id),
                    true,
                )
                .field("User", &user.name, true)
                .field(
                    "Status",
                    if self.invite_poll.outcome.is_none() {
                        "Open"
                    } else {
                        "Closed"
                    },
                    true,
                );

            // row
            embed
                .field(
                    "Created At",
                    DiscordTimestamp::new(
                        self.invite_poll.created_at,
                        DiscordTimestampStyle::FullShort,
                    ),
                    true,
                )
                .field(
                    "Ends At",
                    {
                        let ts = DiscordTimestamp::new(
                            self.invite_poll.ends_at,
                            DiscordTimestampStyle::FullShort,
                        );

                        format!(
                            "{} ({})",
                            ts,
                            ts.with_style(DiscordTimestampStyle::Relative)
                        )
                    },
                    true,
                );

            // row
            embed.field(
                "Votes",
                {
                    let mut bar = ProgressBar::builder();
                    bar.max(self.yes_count + self.no_count)
                        .with_count(true)
                        .with_percentage(true);

                    format!(
                        "{} {}\n{} {}",
                        emojis::LARGE_GREEN_CIRCLE,
                        bar.value(self.yes_count).build().unwrap(),
                        emojis::LARGE_RED_CIRCLE,
                        bar.value(self.no_count).build().unwrap()
                    )
                },
                false,
            );

            // row
            {
                if let Some(outcome) = self.invite_poll.outcome {
                    embed.field(
                        "Outcome",
                        match outcome {
                            InvitePollOutcome::Allow => {
                                [emojis::CHECK_MARK_BUTTON, " Allowed"].concat()
                            }
                            InvitePollOutcome::Deny => [emojis::NO_ENTRY, " Denied"].concat(),
                        },
                        true,
                    );
                } else {
                    embed.field("", "", true);
                }

                if let Some(message) = self.invite_poll.message.as_ref() {
                    embed.field(
                        match self.invite_poll.outcome {
                            Some(InvitePollOutcome::Allow) => "Invite",
                            Some(InvitePollOutcome::Deny) => "Reason",
                            None => "",
                        },
                        message,
                        true,
                    );
                } else {
                    embed.field("", "", true);
                }
            }

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
                        btn.custom_id("democracy.invite-poll-vote.no")
                            .label("No")
                            .style(ButtonStyle::Danger)
                    })
                });
                components
            }
        };

        let mut res = MessageRenderer::default();
        res.set_components(components);
        res.set_embeds(embeds);
        Ok(res)
    }
}
