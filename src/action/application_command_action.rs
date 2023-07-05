use chrono::Duration;
use once_cell::sync::Lazy;
use serenity::{
    model::prelude::{
        command::{Command, CommandOptionType},
        interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
    },
    prelude::Context,
};

use crate::{
    entities::{InvitePoll, InvitePollWithVoteCount},
    error::Error,
    util::serenity::{GuildId, UserId},
    POOL,
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
                        .kind(CommandOptionType::String)
                        .description("Duration of the poll")
                })
        })
        .await?;

        Ok(())
    }

    pub async fn execute(
        &self,
        ctx: Context,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<(), Error> {
        match self {
            ApplicationCommandAction::CreateInvitePoll {
                guild_id,
                user_id,
                duration,
            } => {
                let pool = POOL.get().expect("the Pool to be initialized");
                let mut transaction = pool.begin().await?;

                let invite_poll = InvitePoll::create(
                    &mut *transaction,
                    guild_id.to_owned(),
                    user_id.to_owned(),
                    *duration,
                )
                .await?;

                let mut invite_poll = InvitePollWithVoteCount {
                    invite_poll,
                    yes_count: 0,
                    maybe_count: 0,
                    no_count: 0,
                };

                let render = invite_poll.create_renderer(ctx.clone()).await?;
                interaction
                    .create_interaction_response(&ctx.http, |resp| {
                        resp.kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|data| {
                                render(&mut data.into());
                                data
                            })
                    })
                    .await?;
                let message = interaction.get_interaction_response(&ctx.http).await?;
                invite_poll
                    .invite_poll
                    .update_message(&mut *transaction, &message)
                    .await?;

                transaction.commit().await?;

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
                                .map(|str| {
                                    str.parse::<UserId>().map_err(|err| {
                                        Error::InvitePollIdInvalid(str.to_owned(), err.into())
                                    })
                                })
                                .transpose()?;
                        }

                        "duration" => {
                            duration = opt
                                .value
                                .as_ref()
                                .and_then(|val| val.as_str())
                                .map::<Result<_, Box<dyn std::error::Error>>, _>(|str| {
                                    let dur = humantime::parse_duration(str)?;
                                    let dur = Duration::from_std(dur)?;
                                    Ok(dur)
                                })
                                .transpose()
                                .map_err(Error::InvalidDuration)?;
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
                        guild_id: guild_id.into(),
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
