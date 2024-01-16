use serenity::{
    all::{CommandInteraction, CommandOptionType},
    async_trait,
    builder::{
        CreateCommand, CreateCommandOption, CreateEmbed, CreateInteractionResponse,
        CreateInteractionResponseMessage,
    },
    model::prelude::{Channel, Interaction},
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
    interaction: CommandInteraction,
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

        let guild = Guild::create_or_update(
            &mut *transaction,
            &self.guild_id,
            &self.invite_channel_id,
            self.invite_poll_quorum,
        )
        .await?;
        trace!("updated settings: {:?}", guild);

        let invite_channel = guild.invite_channel_id.to_channel(&ctx.http).await?;

        self.interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::default()
                        .ephemeral(true)
                        .add_embed(
                            CreateEmbed::default()
                                .title("Settings")
                                .color(colors::DISCORD_BLURPLE)
                                .field(
                                    "Invite Channel",
                                    match invite_channel {
                                        Channel::Guild(ch) => ch.name,
                                        Channel::Private(_) => "private".to_owned(),
                                        _ => "unknown".to_owned(),
                                    },
                                    true,
                                )
                                .field(
                                    "Required Votes",
                                    format!("{:.0}%", guild.invite_poll_quorum * 100.0),
                                    true,
                                ),
                        ),
                ),
            )
            .await?;

        transaction.commit().await?;

        Ok(())
    }

    fn register() -> Vec<CreateCommand> {
        vec![CreateCommand::new(ACTION_ID)
            .description("Configures the bot for the current guild")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::Channel,
                    INVITE_CHANNEL_ID_OPTION_NAME,
                    "Which channels users should be invited to",
                )
                .required(true),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    INVITE_POLL_QUORUM_OPTION_NAME,
                    "The minimum amount of votes required",
                )
                .min_int_value(0)
                .max_int_value(100)
                .required(true),
            )]
    }
}

impl<'a> TryFrom<&'a Interaction> for Configure {
    type Error = ParseActionError;

    fn try_from(value: &'a Interaction) -> Result<Self, Self::Error> {
        let interaction = value
            .as_command()
            .ok_or(ParseActionError::MismatchedAction)?;
        if interaction.data.name != ACTION_ID {
            return Err(ParseActionError::MismatchedAction);
        }

        // check permissions
        let permissions = interaction
            .member
            .as_ref()
            .ok_or(ParseActionError::InsufficientPermissions)?
            .permissions
            .ok_or(ParseActionError::InsufficientPermissions)?;
        if !permissions.administrator() {
            return Err(ParseActionError::InsufficientPermissions);
        }

        // options
        let mut invite_channel_id: Option<ChannelId> = None;
        let mut invite_poll_quorum: Option<f32> = None;

        for opt in &interaction.data.options {
            match opt.name.as_str() {
                name @ INVITE_CHANNEL_ID_OPTION_NAME => {
                    let value = resolve_option!(ACTION_ID, &opt.value, Channel, name)?;
                    invite_channel_id = Some((*value).into());
                }
                name @ INVITE_POLL_QUORUM_OPTION_NAME => {
                    let value = resolve_option!(ACTION_ID, &opt.value, Integer, name)?;
                    let value = ((*value).clamp(0, 100) as f32) / 100.0;
                    invite_poll_quorum = Some(value);
                }
                other => {
                    return Err(ParseActionError::UnknownOption {
                        action: ACTION_ID,
                        option: other.to_owned(),
                    });
                }
            }
        }

        let invite_channel_id = invite_channel_id.ok_or(ParseActionError::MissingOption {
            action: ACTION_ID,
            option: INVITE_CHANNEL_ID_OPTION_NAME.into(),
        })?;
        let invite_poll_quorum = invite_poll_quorum.ok_or(ParseActionError::MissingOption {
            action: ACTION_ID,
            option: INVITE_POLL_QUORUM_OPTION_NAME.into(),
        })?;

        let guild_id = interaction
            .guild_id
            .ok_or(ParseActionError::NotInAGuild { action: ACTION_ID })
            .map(Into::into)?;

        Ok(Self {
            interaction: interaction.clone(),
            guild_id,
            invite_channel_id,
            invite_poll_quorum,
        })
    }
}
