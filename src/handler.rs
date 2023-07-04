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
    background_poll_handler::BackgroundPollHandler,
    error::Error,
};

pub struct Handler;

impl Handler {
    async fn on_ready(&self, ctx: Context, _ready: &Ready) -> Result<(), Error> {
        ApplicationCommandAction::register(ctx.clone()).await?;
        BackgroundPollHandler::new(ctx, Duration::from_secs(60))
            .start()
            .await?;
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
