use std::time::Duration;

use serenity::{
    async_trait,
    model::prelude::{
        interaction::{
            application_command::ApplicationCommandInteraction,
            message_component::MessageComponentInteraction, Interaction,
        },
        Ready,
    },
    prelude::{Context, EventHandler},
};

use crate::{
    action::{ApplicationCommandAction, MessageComponentAction},
    entities::{InvitePollOutcome, InvitePollWithVoteCount},
    error::Error,
};

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

            let outcome = poll.outcome(ctx).await?;
            poll.invite_poll.outcome = Some(outcome);
            debug!("expired poll outcome: {:?}", outcome);

            if matches!(outcome, InvitePollOutcome::Allow) {
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

            poll.invite_poll.save().await?;
        }
    }
}
