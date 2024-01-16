use std::time::Duration;

use serenity::{
    all::{Command, Interaction},
    async_trait,
    builder::{CreateInteractionResponse, CreateInteractionResponseMessage},
    model::prelude::Ready,
    prelude::{Context, EventHandler},
};

use crate::{
    action::{Action, Actions},
    background_poll_handler::BackgroundPollHandler,
    error::Error,
    util::serenity::InteractionExt,
};

pub struct Handler;

impl Handler {
    async fn on_ready(&self, ctx: Context, _ready: &Ready) -> Result<(), Error> {
        Command::set_global_commands(&ctx.http, Actions::register()).await?;

        BackgroundPollHandler::new(ctx, Duration::from_secs(10))
            .start()
            .await;
        Ok(())
    }

    async fn on_interaction(&self, ctx: Context, interaction: Interaction) -> Result<(), Error> {
        let action = Actions::try_from(&interaction)?;
        action.execute(&ctx).await
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, event: Ready) {
        self.on_ready(ctx, &event).await.unwrap();
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        debug!("interaction: {:?}", interaction);

        match self.on_interaction(ctx.clone(), interaction.clone()).await {
            Ok(()) => {}
            Err(err) if err.is_client_error() => {
                let res = interaction
                    .create_interaction_response(&ctx.http, {
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::default()
                                .ephemeral(true)
                                .content(format!("Could not perform action: {}.", err)),
                        )
                    })
                    .await;

                match res {
                    Ok(()) => {}
                    Err(err) => error!("{0}: {0:?}", err),
                }
            }
            Err(err) => error!("{0}: {0:?}", err),
        }
    }
}
