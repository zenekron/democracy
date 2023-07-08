use std::time::Duration;

use serenity::{
    async_trait,
    model::prelude::{command::Command, interaction::Interaction, Ready},
    prelude::{Context, EventHandler},
};

use crate::{
    action::{Action, Actions},
    background_poll_handler::BackgroundPollHandler,
    error::Error,
};

pub struct Handler;

impl Handler {
    async fn on_ready(&self, ctx: Context, _ready: &Ready) -> Result<(), Error> {
        Command::set_global_application_commands(&ctx.http, Actions::register_all).await?;

        BackgroundPollHandler::new(ctx, Duration::from_secs(60))
            .start()
            .await?;
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

        match self.on_interaction(ctx, interaction).await {
            Ok(()) => {}
            Err(err) => error!("{0}: {0:?}", err),
        }
    }
}
