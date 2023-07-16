use std::time::Duration;

use serenity::{
    async_trait,
    builder::CreateApplicationCommands,
    model::{
        application::interaction::application_command::ApplicationCommandInteraction,
        prelude::{command::CommandOptionType, Interaction, InteractionResponseType},
    },
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
    interaction: ApplicationCommandInteraction,
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
        self.interaction
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|data| {
                        renderer.render_create_interaction_response_data(data)
                    })
            })
            .await?;

        // embed `message_id` into the `InvitePoll`
        let message = self.interaction.get_interaction_response(&ctx.http).await?;
        invite_poll
            .invite_poll
            .update_message(&mut *transaction, &message)
            .await?;

        transaction.commit().await?;

        Ok(())
    }

    fn register(commands: &mut CreateApplicationCommands) -> &mut CreateApplicationCommands {
        commands.create_application_command(|command| {
            command
                .name(ACTION_ID)
                .description("Creates a petition to invite a new user")
                .create_option(|opt| {
                    opt.name(USER_ID_OPTION_NAME)
                        .kind(CommandOptionType::String)
                        .description("The ID of the user to invite")
                        .required(true)
                })
                .create_option(|opt| {
                    opt.name(DURATION_OPTION_NAME)
                        .kind(CommandOptionType::String)
                        .description("Duration of the poll")
                })
        })
    }
}

impl<'a> TryFrom<&'a Interaction> for CreateInvitePoll {
    type Error = ParseActionError;

    fn try_from(value: &'a Interaction) -> Result<Self, Self::Error> {
        let interaction = value
            .as_application_command()
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
                    let value = resolve_option!(ACTION_ID, &opt.resolved, String, name)?;
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
                    let value = resolve_option!(ACTION_ID, &opt.resolved, String, name)?;
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
        let duration = duration.ok_or(ParseActionError::MissingOption {
            action: ACTION_ID,
            option: USER_ID_OPTION_NAME.into(),
        })?;

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
