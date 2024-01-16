use std::time::Duration;

use serenity::{
    all::{CommandInteraction, CommandOptionType},
    async_trait,
    builder::{
        CreateCommand, CreateCommandOption, CreateInteractionResponse,
        CreateInteractionResponseMessage, CreateMessage,
    },
    model::prelude::Interaction,
    prelude::Context,
};

use crate::{
    entities::{InvitePoll, InvitePollWithVoteCount},
    error::Error,
    resolve_option,
    util::serenity::{GuildExt, GuildId, UserId},
    POOL,
};

use super::{Action, ParseActionError};

const ACTION_ID: &'static str = "invite";
const USER_ID_OPTION_NAME: &'static str = "user-id";
const DURATION_OPTION_NAME: &'static str = "duration";

#[derive(Debug)]
pub struct CreateInvitePoll {
    interaction: CommandInteraction,
    guild_id: GuildId,
    inviter: UserId,
    invitee: UserId,
    duration: Duration,
}

#[async_trait]
impl Action for CreateInvitePoll {
    async fn execute(&self, ctx: &Context) -> Result<(), Error> {
        let pool = POOL.get().expect("the Pool to be initialized");
        let mut transaction = pool.begin().await?;

        // preliminary checks
        let guild = self.guild_id.to_partial_guild(&ctx.http).await?;
        if guild.is_member(&ctx.http, &self.invitee).await? {
            return Err(Error::CannotInviteMember(self.invitee.clone()));
        }

        // create poll
        let invite_poll = InvitePoll::create(
            &mut *transaction,
            &self.guild_id,
            &self.inviter,
            &self.invitee,
            &self.duration,
        )
        .await?;

        // render poll
        let mut invite_poll = InvitePollWithVoteCount {
            invite_poll,
            yes_count: 0,
            no_count: 0,
        };

        let renderer = invite_poll.create_renderer(ctx.clone()).await?;
        let msg = self
            .interaction
            .channel_id
            .send_message(
                &ctx.http,
                renderer.render_create_message(CreateMessage::default()),
            )
            .await?;

        invite_poll
            .invite_poll
            .update_message(&mut *transaction, &msg)
            .await?;

        self.interaction
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::default()
                        .ephemeral(true)
                        .content(format!(
                            "https://discord.com/channels/{}/{}/{}",
                            self.guild_id.get(),
                            msg.channel_id,
                            msg.id
                        )),
                ),
            )
            .await?;

        transaction.commit().await?;

        Ok(())
    }

    fn register() -> Vec<CreateCommand> {
        vec![CreateCommand::new(ACTION_ID)
            .description("Creates a petition to invite a new user")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    USER_ID_OPTION_NAME,
                    "The ID of the user to invite",
                )
                .required(true),
            )
            .add_option(CreateCommandOption::new(
                CommandOptionType::String,
                DURATION_OPTION_NAME,
                "Duration of the poll",
            ))]
    }
}

impl<'a> TryFrom<&'a Interaction> for CreateInvitePoll {
    type Error = ParseActionError;

    fn try_from(value: &'a Interaction) -> Result<Self, Self::Error> {
        let interaction = value
            .as_command()
            .ok_or(ParseActionError::MismatchedAction)?;
        if interaction.data.name != ACTION_ID {
            return Err(ParseActionError::MismatchedAction);
        }

        // options
        let mut user_id: Option<UserId> = None;
        let mut duration: Option<Duration> = None;

        for opt in &interaction.data.options {
            match opt.name.as_str() {
                name @ USER_ID_OPTION_NAME => {
                    let value = resolve_option!(ACTION_ID, &opt.value, String, name)?;
                    let value = value.parse::<UserId>().map_err(|err| {
                        ParseActionError::InvalidOptionValue {
                            action: ACTION_ID,
                            option: name.into(),
                            value: value.to_string(),
                            source: Box::new(err),
                        }
                    })?;
                    user_id = Some(value);
                }
                name @ DURATION_OPTION_NAME => {
                    let value = resolve_option!(ACTION_ID, &opt.value, String, name)?;
                    let value = humantime::parse_duration(value).map_err(|err| {
                        ParseActionError::InvalidOptionValue {
                            action: ACTION_ID,
                            option: name.into(),
                            value: value.to_string(),
                            source: Box::new(err),
                        }
                    })?;
                    duration = Some(value);
                }
                other => {
                    return Err(ParseActionError::UnknownOption {
                        action: ACTION_ID,
                        option: other.to_owned(),
                    });
                }
            }
        }

        let user_id = user_id.ok_or(ParseActionError::MissingOption {
            action: ACTION_ID,
            option: USER_ID_OPTION_NAME.into(),
        })?;
        let duration = duration.unwrap_or(Duration::from_secs(3 * 24 * 60 * 60)); // 3 days

        let guild_id = interaction
            .guild_id
            .ok_or(ParseActionError::NotInAGuild { action: ACTION_ID })
            .map(Into::into)?;

        Ok(Self {
            interaction: interaction.clone(),
            guild_id,
            inviter: interaction.user.id.into(),
            invitee: user_id,
            duration,
        })
    }
}
