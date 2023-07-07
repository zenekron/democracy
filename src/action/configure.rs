use serenity::{
    async_trait,
    builder::CreateApplicationCommand,
    model::prelude::{
        application_command::ApplicationCommandInteraction, command::CommandOptionType, Channel,
        Interaction, InteractionResponseType,
    },
    prelude::Context,
};

use crate::{
    entities::Guild,
    error::Error,
    resolve_option,
    util::{
        colors,
        serenity::{ChannelId, GuildId},
    },
    POOL,
};

use super::{Action, ParseActionError};

const ACTION_ID: &'static str = "configure";
const INVITE_CHANNEL_ID_OPTION_NAME: &'static str = "invite-channel";
const INVITE_POLL_QUORUM_OPTION_NAME: &'static str = "invite-poll-quorum";

#[derive(Debug)]
pub struct Configure {
    interaction: ApplicationCommandInteraction,
    guild_id: GuildId,
    invite_channel_id: ChannelId,
    invite_poll_quorum: f32,
}

#[async_trait]
impl Action for Configure {
    async fn execute(&self, ctx: &Context) -> Result<(), Error> {
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

        self.interaction
            .create_interaction_response(&ctx.http, |response| {
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
                    })
            })
            .await?;

        transaction.commit().await?;

        Ok(())
    }

    fn register() -> Option<CreateApplicationCommand> {
        let mut command = CreateApplicationCommand::default();
        command
            .name(ACTION_ID)
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
            });

        Some(command)
    }
}

impl<'a> TryFrom<&'a Interaction> for Configure {
    type Error = ParseActionError;

    fn try_from(value: &'a Interaction) -> Result<Self, Self::Error> {
        let interaction = value
            .as_application_command()
            .ok_or(ParseActionError::MismatchedAction)?;
        if interaction.data.name != ACTION_ID {
            return Err(ParseActionError::MismatchedAction);
        }

        // options
        let mut invite_channel_id: Option<ChannelId> = None;
        let mut invite_poll_quorum: Option<f32> = None;

        for opt in &interaction.data.options {
            match opt.name.as_str() {
                name @ INVITE_CHANNEL_ID_OPTION_NAME => {
                    let value = resolve_option!(ACTION_ID, &opt.resolved, Channel, name)?;
                    invite_channel_id = Some(value.id.into());
                }
                name @ INVITE_POLL_QUORUM_OPTION_NAME => {
                    let value = resolve_option!(ACTION_ID, &opt.resolved, Number, name)?;
                    invite_poll_quorum = Some(*value as f32);
                }
                other => {
                    return Err(ParseActionError::UnknownOption(ACTION_ID, other.to_owned()));
                }
            }
        }

        let invite_channel_id = invite_channel_id.ok_or(ParseActionError::MissingOption(
            ACTION_ID,
            INVITE_CHANNEL_ID_OPTION_NAME,
        ))?;
        let invite_poll_quorum = invite_poll_quorum.ok_or(ParseActionError::MissingOption(
            ACTION_ID,
            INVITE_POLL_QUORUM_OPTION_NAME,
        ))?;

        let guild_id = interaction
            .guild_id
            .ok_or(ParseActionError::InvalidInteractionKind)
            .map(Into::into)?;

        Ok(Self {
            interaction: interaction.clone(),
            guild_id,
            invite_channel_id,
            invite_poll_quorum,
        })
    }
}
