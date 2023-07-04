use std::str::FromStr;

use chrono::Duration;
use once_cell::sync::Lazy;
use serenity::{
    model::prelude::{
        command::{Command, CommandOptionType},
        interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
        GuildId, UserId,
    },
    prelude::Context,
};

use crate::{
    entities::{InvitePoll, InvitePollWithVoteCount},
    error::Error,
    handler::Handler,
};

static DEFAULT_POLL_DURATION: Lazy<Duration> = Lazy::new(|| Duration::days(3));

#[derive(Debug)]
pub enum ApplicationCommandAction {
    CreateInvitePoll {
        guild_id: GuildId,
        user_id: UserId,
        duration: Duration,
    },
}

impl ApplicationCommandAction {
    pub async fn register(ctx: Context) -> Result<(), Error> {
        let _invite = Command::create_global_application_command(&ctx.http, |cmd| {
            cmd.name("invite")
                .description("Creates a petition to invite a new user")
                .create_option(|opt| {
                    opt.name("user_id")
                        .kind(CommandOptionType::String)
                        .description("The ID of the user to invite")
                        .required(true)
                })
                .create_option(|opt| {
                    opt.name("duration")
                        .kind(CommandOptionType::Integer)
                        .description("Duration in days of the poll")
                        .min_int_value(1)
                })
        })
        .await?;

        Ok(())
    }

    pub async fn execute(
        &self,
        handler: &Handler,
        ctx: Context,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<(), Error> {
        match self {
            ApplicationCommandAction::CreateInvitePoll {
                guild_id,
                user_id,
                duration,
            } => {
                let invite_poll = InvitePoll::create(
                    &handler.pool,
                    guild_id.to_owned(),
                    user_id.to_owned(),
                    *duration,
                )
                .await?;

                let invite_poll = InvitePollWithVoteCount {
                    invite_poll,
                    yes_count: 0,
                    maybe_count: 0,
                    no_count: 0,
                };

                let render = invite_poll.create_interaction_response(ctx.clone()).await?;
                interaction
                    .create_interaction_response(&ctx.http, |resp| {
                        render(resp).kind(InteractionResponseType::ChannelMessageWithSource)
                    })
                    .await?;

                Ok(())
            }
        }
    }
}

impl TryFrom<&ApplicationCommandInteraction> for ApplicationCommandAction {
    type Error = Error;

    fn try_from(interaction: &ApplicationCommandInteraction) -> Result<Self, Self::Error> {
        match interaction.data.name.as_str() {
            "invite" => {
                let mut user_id: Option<UserId> = None;
                let mut duration: Option<Duration> = None;

                for opt in &interaction.data.options {
                    match opt.name.as_str() {
                        "user_id" => {
                            user_id = opt
                                .value
                                .as_ref()
                                .and_then(|val| val.as_str())
                                .map(FromStr::from_str)
                                .transpose()?;
                        }

                        "duration" => {
                            duration = opt
                                .value
                                .as_ref()
                                .and_then(|val| val.as_i64())
                                .map(Duration::days);
                        }

                        other => {
                            return Err(Error::UnknownCommandOption(
                                "invite".to_owned(),
                                other.to_owned(),
                            ))
                        }
                    }
                }

                let guild_id = interaction
                    .guild_id
                    .ok_or_else(|| Error::GuildCommandNotInGuild(interaction.data.name.clone()))?;

                match user_id {
                    Some(user_id) => Ok(Self::CreateInvitePoll {
                        guild_id,
                        user_id,
                        duration: duration.unwrap_or(*DEFAULT_POLL_DURATION),
                    }),
                    None => Err(Error::MissingCommandOption(
                        interaction.data.name.clone(),
                        "user_id".to_owned(),
                    )),
                }
            }

            other => Err(Error::UnknownCommand(other.to_owned())),
        }
    }
}
