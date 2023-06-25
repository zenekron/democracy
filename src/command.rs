use serenity::{
    futures::future::join_all,
    model::prelude::{
        command::Command as SerenityCommand,
        interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
    },
    prelude::Context,
};

use crate::error::Error;

#[derive(Debug)]
pub enum Command {
    Ping,
}

impl Command {
    pub async fn register(ctx: Context) -> Result<(), Error> {
        let commands = [SerenityCommand::create_global_application_command(
            &ctx.http,
            |cmd| cmd.name("ping").description("Ping"),
        )];

        let _commands = join_all(commands)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        Ok(())
    }

    pub async fn execute(
        &self,
        ctx: Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), Error> {
        debug!("{:?}", self);

        match self {
            Command::Ping => {
                command
                    .create_interaction_response(&ctx.http, |resp| {
                        resp.kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|data| data.title("Pong").content("pong"))
                    })
                    .await?;

                Ok(())
            }
        }
    }
}

impl TryFrom<&ApplicationCommandInteraction> for Command {
    type Error = Error;

    fn try_from(value: &ApplicationCommandInteraction) -> Result<Self, Self::Error> {
        match value.data.name.as_str() {
            "ping" => Ok(Command::Ping),
            other => Err(Error::UnknownCommand(other.to_owned())),
        }
    }
}
