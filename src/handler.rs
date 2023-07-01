use serenity::{
    async_trait,
    model::{
        application::interaction::application_command::ApplicationCommandInteraction,
        prelude::{interaction::Interaction, Ready},
    },
    prelude::{Context, EventHandler},
};
use sqlx::PgPool;

use crate::{action::Action, error::Error};

pub struct Handler {
    pub pool: PgPool,
}

impl Handler {
    async fn on_application_command_interaction(
        &self,
        ctx: Context,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<(), Error> {
        let action = Action::try_from(interaction)?;
        action.execute(self, ctx, interaction).await
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _event: Ready) {
        Action::register(ctx).await.unwrap();
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

            ref other => warn!("unhandled interaction: {:?}", other),
        }
    }
}
