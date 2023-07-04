use std::time::Duration;

use serenity::{
    async_trait,
    model::prelude::{
        interaction::{
            application_command::ApplicationCommandInteraction,
            message_component::MessageComponentInteraction, Interaction,
        },
        Ready, UserId,
    },
    prelude::{Context, EventHandler},
};

use crate::{
    action::{ApplicationCommandAction, MessageComponentAction},
    entities::{InvitePollOutcome, InvitePollWithVoteCount},
    error::Error,
};

const VOTE_THRESHOLD: f32 = 0.8;

pub struct Handler;

impl Handler {
    async fn on_ready(&self, ctx: Context, _ready: &Ready) -> Result<(), Error> {
        ApplicationCommandAction::register(ctx.clone()).await?;
        background_poll_closer(&ctx).await?;
        Ok(())
    }

    async fn on_application_command_interaction(
        &self,
        ctx: Context,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<(), Error> {
        let action = ApplicationCommandAction::try_from(interaction)?;
        action.execute(ctx, interaction).await
    }

    async fn on_message_component_interaction(
        &self,
        ctx: Context,
        interaction: &MessageComponentInteraction,
    ) -> Result<(), Error> {
        let action = MessageComponentAction::try_from(interaction)?;
        action.execute(ctx, interaction).await
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, event: Ready) {
        self.on_ready(ctx, &event).await.unwrap();
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        debug!("interaction: {:?}", interaction);

        match interaction {
            Interaction::ApplicationCommand(ref interaction) => {
                match self
                    .on_application_command_interaction(ctx, interaction)
                    .await
                {
                    Ok(()) => {}
                    Err(err) => error!("{0}: {0:?}", err),
                }
            }

            Interaction::MessageComponent(ref interaction) => {
                match self
                    .on_message_component_interaction(ctx, interaction)
                    .await
                {
                    Ok(()) => {}
                    Err(err) => error!("{0}: {0:?}", err),
                }
            }

            ref other => warn!("unhandled interaction: {:?}", other),
        }
    }
}

async fn background_poll_closer(ctx: &Context) -> Result<(), Error> {
    let mut interval = tokio::time::interval(Duration::from_secs(60));

    loop {
        interval.tick().await;

        let polls = InvitePollWithVoteCount::find_expired().await?;
        for mut poll in polls {
            debug!("expired poll: {:?}", poll);

            let guild_users = {
                let guild = poll
                    .invite_poll
                    .guild_id()
                    .to_partial_guild(&ctx.http)
                    .await?;

                let mut max = 0_usize;
                let mut after: Option<UserId> = None;
                loop {
                    let page = guild.members(&ctx.http, None, after).await?;
                    if page.len() == 0 {
                        break;
                    }

                    max += page.iter().filter(|m| m.user.bot == false).count();
                    after = page.last().map(|u| u.user.id);
                }

                max
            };

            let outcome = {
                let required_votes = (guild_users as f32 * VOTE_THRESHOLD).ceil() as usize;

                if poll.no_count == 0
                    && (poll.yes_count + poll.maybe_count) >= required_votes as i64
                {
                    InvitePollOutcome::Allow
                } else {
                    InvitePollOutcome::Deny
                }
            };
            debug!("expired poll outcome: {:?}", outcome);

            if outcome == InvitePollOutcome::Allow {
                let guild = poll
                    .invite_poll
                    .guild_id()
                    .to_partial_guild(&ctx.http)
                    .await?;
                let general = guild
                    .channels(&ctx.http)
                    .await?
                    .into_values()
                    .find(|ch| ch.name == "general")
                    .unwrap();
                let invite = general
                    .create_invite(&ctx.http, |invite| invite.unique(true).max_uses(1))
                    .await?;

                let pm = poll
                    .invite_poll
                    .user_id()
                    .create_dm_channel(&ctx.http)
                    .await?;

                pm.send_message(&ctx.http, |msg| msg.content(invite.url()))
                    .await?;
            }

            poll.invite_poll.close(outcome).await?;
        }
    }
}
