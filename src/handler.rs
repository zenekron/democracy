use sea_orm::DatabaseConnection;
use serenity::{
    async_trait,
    model::prelude::{interaction::Interaction, Ready},
    prelude::{Context, EventHandler},
};

use crate::command::Command;

pub struct Handler {
    pub database: DatabaseConnection,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _event: Ready) {
        Command::register(ctx).await.unwrap();
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        debug!("interaction: {:?}", interaction);

        match interaction {
            Interaction::ApplicationCommand(ref command_interaction) => {
                let command = match Command::try_from(command_interaction) {
                    Ok(val) => val,
                    Err(err) => {
                        error!("Command parsing failure: {:?}", err);
                        return;
                    }
                };

                match command.execute(self, ctx, command_interaction).await {
                    Ok(val) => val,
                    Err(err) => {
                        error!("Command execution error: {:?}", err);
                        return;
                    }
                }
            }

            ref other => warn!("unhandled interaction: {:?}", other),
        }
    }
}
