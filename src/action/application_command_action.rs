use chrono::Duration;
use once_cell::sync::Lazy;
use serenity::{
    model::prelude::{
        application_command::CommandDataOptionValue,
        command::{Command, CommandOptionType},
        interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
        Channel,
    },
    prelude::Context,
};

use crate::{
    entities::{Guild, InvitePoll, InvitePollWithVoteCount},
    error::Error,
    util::serenity::{ChannelId, GuildId, UserId},
    POOL,
};

static DEFAULT_POLL_DURATION: Lazy<Duration> = Lazy::new(|| Duration::days(3));

#[derive(Debug)]
pub enum ApplicationCommandAction {
    Configure {
        guild_id: GuildId,
        invite_channel: ChannelId,
        vote_success_threshold: f32,
    },
    CreateInvitePoll {
        guild_id: GuildId,
        user_id: UserId,
        duration: Duration,
    },
}

impl ApplicationCommandAction {
    pub async fn register(ctx: Context) -> Result<(), Error> {
        Command::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command
                        .name("configure")
                        .description("Configures the bot for the current guild")
                        .create_option(|opt| {
                            opt.name("invite_channel")
                                .kind(CommandOptionType::Channel)
                                .description("Which channels users should be invited to")
                                .required(true)
                        })
                    .create_option(|opt| {
                        opt.name("vote_success_threshold")
                            .kind(CommandOptionType::Number)
                            .description("The minimum percentage of votes required for a vote to be considered valid")
                            .min_number_value(0.0)
                            .max_number_value(100.0)
                            .required(true)
                    })
                })
                .create_application_command(|command| {
                    command
                        .name("invite")
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
            ApplicationCommandAction::Configure {
                guild_id,
                invite_channel,
                vote_success_threshold,
            } => {
                let pool = POOL.get().expect("the Pool to be initialized");
                let mut transaction = pool.begin().await?;

                let guild = Guild::create(
                    &mut *transaction,
                    guild_id,
                    invite_channel,
                    *vote_success_threshold,
                )
                .await?;

                let invite_channel = guild.invite_channel_id.to_channel(&ctx.http).await?;

                interaction
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|data| {
                                data //
                                    .ephemeral(true)
                                    .embed(|embed| {
                                        embed.field(
                                            "Invite Channel",
                                            match invite_channel {
                                                Channel::Guild(ch) => ch.name,
                                                Channel::Private(_) => "private".to_owned(),
                                                Channel::Category(ch) => ch.name,
                                                _ => "unknown".to_owned(),
                                            },
                                            true,
                                        )
                                    })
                            })
                    })
                    .await?;

                transaction.commit().await?;

                Ok(())
            }
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
            "configure" => {
                let mut invite_channel: Option<ChannelId> = None;
                let mut vote_success_threshold: Option<f32> = None;

                for opt in &interaction.data.options {
                    match opt.name.as_str() {
                        "invite_channel" => {
                            invite_channel = match opt.resolved.as_ref() {
                                Some(CommandDataOptionValue::Channel(channel)) => {
                                    Some(channel.id.into())
                                }
                                // TODO: handle `Some(_)`
                                _ => None,
                            };
                        }

                        "vote_success_threshold" => {
                            vote_success_threshold = match opt.resolved {
                                Some(CommandDataOptionValue::Number(val)) => Some(val as _),
                                // TODO: handle `Some(_)`
                                _ => None,
                            }
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

                match (invite_channel, vote_success_threshold) {
                    (Some(invite_channel), Some(vote_success_threshold)) => Ok(Self::Configure {
                        guild_id: guild_id.into(),
                        invite_channel,
                        vote_success_threshold,
                    }),
                    (Some(_), None) => Err(Error::MissingCommandOption(
                        interaction.data.name.clone(),
                        "vote_success_threshold".to_owned(),
                    )),
                    (_, _) => Err(Error::MissingCommandOption(
                        interaction.data.name.clone(),
                        "invite_channel".to_owned(),
                    )),
                }
            }
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
