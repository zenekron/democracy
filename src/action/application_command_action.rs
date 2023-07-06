use serenity::{
    model::prelude::{
        command::Command, interaction::application_command::ApplicationCommandInteraction,
        Interaction,
    },
    prelude::Context,
};

use crate::error::Error;

use super::{Action, Configure, CreateInvitePoll};

#[derive(Debug)]
pub enum ApplicationCommandAction {
    Configure(Configure),
    CreateInvitePoll(CreateInvitePoll),
}

impl ApplicationCommandAction {
    pub async fn register(ctx: Context) -> Result<(), Error> {
        Command::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(Configure::register)
                .create_application_command(CreateInvitePoll::register)
        })
        .await?;

        Ok(())
    }

    pub async fn execute(&self, ctx: Context) -> Result<(), Error> {
        match self {
            ApplicationCommandAction::Configure(action) => action.execute(&ctx).await,
            ApplicationCommandAction::CreateInvitePoll(action) => action.execute(&ctx).await,
        }
    }
}

impl TryFrom<&ApplicationCommandInteraction> for ApplicationCommandAction {
    type Error = Error;

    fn try_from(interaction: &ApplicationCommandInteraction) -> Result<Self, Self::Error> {
        match interaction.data.name.as_str() {
            "configure" => {
                let action =
                    Configure::try_from(&Interaction::ApplicationCommand(interaction.clone()))?;
                Ok(Self::Configure(action))
            }
            "invite" => {
                let action = CreateInvitePoll::try_from(&Interaction::ApplicationCommand(
                    interaction.clone(),
                ))?;
                Ok(Self::CreateInvitePoll(action))
            }

            other => Err(Error::UnknownCommand(other.to_owned())),
        }
    }
}
