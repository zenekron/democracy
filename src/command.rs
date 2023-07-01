use std::str::FromStr;

use serenity::{
    model::prelude::{
        command::{Command as SerenityCommand, CommandOptionType},
        interaction::application_command::ApplicationCommandInteraction,
        UserId,
    },
    prelude::Context,
};

use crate::{entities::InvitePoll, error::Error, handler::Handler};

#[derive(Debug)]
pub enum Command {
    Invite { user_id: UserId },
}

impl Command {
    pub async fn register(ctx: Context) -> Result<(), Error> {
        let _invite = SerenityCommand::create_global_application_command(&ctx.http, |cmd| {
            cmd.name("invite")
                .description("Creates a petition to invite a new user")
                .create_option(|opt| {
                    opt.name("user_id")
                        .kind(CommandOptionType::String)
                        .description("The ID of the user to invite")
                        .required(true)
                })
        })
        .await?;

        Ok(())
    }

    pub async fn execute(
        &self,
        handler: &Handler,
        ctx: Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), Error> {
        debug!("{:?}", self);

        match self {
            Command::Invite { user_id } => {
                let guild_id = command
                    .guild_id
                    .ok_or_else(|| Error::GuildCommandNotInGuild("invite".to_string()))?;

                let invite_poll =
                    InvitePoll::create(&handler.pool, guild_id, user_id.to_owned()).await?;

                invite_poll
                    .create_interaction_response(&ctx, command)
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
            "invite" => {
                let mut user_id: Option<UserId> = None;

                for opt in &value.data.options {
                    match opt.name.as_str() {
                        "user_id" => {
                            user_id = opt
                                .value
                                .as_ref()
                                .and_then(|val| val.as_str())
                                .map(FromStr::from_str)
                                .transpose()?;
                        }

                        other => {
                            return Err(Error::UnknownCommandOption(
                                "invite".to_owned(),
                                other.to_owned(),
                            ))
                        }
                    }
                }

                match user_id {
                    Some(user_id) => Ok(Self::Invite { user_id }),
                    None => Err(Error::MissingCommandOption(
                        "invite".to_owned(),
                        "user_id".to_owned(),
                    )),
                }
            }

            other => Err(Error::UnknownCommand(other.to_owned())),
        }
    }
}

mod colors {
    use serenity::utils::Color;

    // https://www.colorhexa.com/77dd77
    pub static PASTEL_GREEN: Color = Color::new(0x77dd77);
}
