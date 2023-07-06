use serenity::{
    async_trait,
    builder::{CreateApplicationCommand, CreateInteractionResponse},
    model::prelude::{
        application_command::CommandDataOptionValue, command::CommandOptionType, Channel,
        Interaction, InteractionResponseType,
    },
    prelude::Context,
};

use crate::{
    entities::Guild,
    error::Error,
    util::{
        colors,
        serenity::{ChannelId, GuildId},
    },
    POOL,
};

use super::{Action, ParseActionError};

const INVITE_CHANNEL_ID_OPTION_NAME: &str = "invite-channel";
const INVITE_POLL_QUORUM_OPTION_NAME: &str = "invite-poll-quorum";

#[derive(Debug)]
pub struct Configure {
    guild_id: GuildId,
    invite_channel_id: ChannelId,
    invite_poll_quorum: f32,
}

#[async_trait]
impl Action for Configure {
    const ID: &'static str = "configure";

    async fn execute(&self, ctx: &Context) -> Result<CreateInteractionResponse, Error> {
        trace!("{:?}", self);
        let pool = POOL.get().expect("the Pool to be initialized");
        let mut transaction = pool.begin().await?;

        let guild = Guild::create(
            &mut *transaction,
            &self.guild_id,
            &self.invite_channel_id,
            self.invite_poll_quorum,
        )
        .await?;
        trace!("updated settings: {:?}", guild);

        let invite_channel = guild.invite_channel_id.to_channel(&ctx.http).await?;

        transaction.commit().await?;

        let mut response = CreateInteractionResponse::default();
        response
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|data| {
                data //
                    .ephemeral(true)
                    .embed(|embed| {
                        embed
                            .title("Settings")
                            .color(colors::DISCORD_BLURPLE)
                            .field(
                                "Invite Channel",
                                match invite_channel {
                                    Channel::Guild(ch) => ch.name,
                                    Channel::Private(_) => "private".to_owned(),
                                    Channel::Category(ch) => ch.name,
                                    _ => "unknown".to_owned(),
                                },
                                true,
                            )
                            .field(
                                "Required Votes",
                                format!("{:.0}%", guild.invite_poll_quorum() * 100.0),
                                true,
                            )
                    })
            });

        Ok(response)
    }

    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("configure")
            .description("Configures the bot for the current guild")
            .create_option(|opt| {
                opt.name(INVITE_CHANNEL_ID_OPTION_NAME)
                    .kind(CommandOptionType::Channel)
                    .description("Which channels users should be invited to")
                    .required(true)
            })
            .create_option(|opt| {
                opt.name(INVITE_POLL_QUORUM_OPTION_NAME)
                    .kind(CommandOptionType::Number)
                    .description("The minimum amount of votes required")
                    .min_number_value(0.0)
                    .max_number_value(100.0)
                    .required(true)
            })
    }
}

impl<'a> TryFrom<&'a Interaction> for Configure {
    type Error = ParseActionError;

    fn try_from(value: &'a Interaction) -> Result<Self, Self::Error> {
        let interaction = value
            .as_application_command()
            .ok_or(ParseActionError::InvalidInteractionKind)?;

        // options
        let mut invite_channel_id: Option<ChannelId> = None;
        let mut invite_poll_quorum: Option<f32> = None;

        for opt in &interaction.data.options {
            match opt.name.as_str() {
                name @ INVITE_CHANNEL_ID_OPTION_NAME => {
                    invite_channel_id = match &opt.resolved {
                        Some(CommandDataOptionValue::Channel(channel)) => Ok(channel.id.into()),
                        value => Err(ParseActionError::InvalidOptionKind(
                            Self::ID,
                            name.into(),
                            "channel",
                            value.clone(),
                        )),
                    }
                    .map(Some)?;
                }
                name @ INVITE_POLL_QUORUM_OPTION_NAME => {
                    invite_poll_quorum = match &opt.resolved {
                        Some(CommandDataOptionValue::Number(val)) => Ok(*val as f32),
                        value => Err(ParseActionError::InvalidOptionKind(
                            Self::ID,
                            name.into(),
                            "f32",
                            value.clone(),
                        )),
                    }
                    .map(Some)?;
                }
                other => {
                    return Err(ParseActionError::UnknownOption(Self::ID, other.to_owned()));
                }
            }
        }

        let invite_channel_id = invite_channel_id.ok_or(ParseActionError::MissingOption(
            Self::ID,
            INVITE_CHANNEL_ID_OPTION_NAME,
        ))?;
        let invite_poll_quorum = invite_poll_quorum.ok_or(ParseActionError::MissingOption(
            Self::ID,
            INVITE_POLL_QUORUM_OPTION_NAME,
        ))?;

        let guild_id = interaction
            .guild_id
            .ok_or(ParseActionError::InvalidInteractionKind)
            .map(Into::into)?;

        Ok(Self {
            guild_id,
            invite_channel_id,
            invite_poll_quorum,
        })
    }
}
