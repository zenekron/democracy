use std::time::Duration;

use serenity::{
    async_trait,
    builder::CreateApplicationCommand,
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
    util::serenity::{GuildId, UserId},
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
    user_id: UserId,
    duration: Duration,
}

#[async_trait]
impl Action for CreateInvitePoll {
    async fn execute(&self, ctx: &Context) -> Result<(), Error> {
        let pool = POOL.get().expect("the Pool to be initialized");
        let mut transaction = pool.begin().await?;

        let invite_poll = InvitePoll::create(
            &mut *transaction,
            &self.guild_id,
            &self.user_id,
            &self.duration,
        )
        .await?;

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

        let message = self.interaction.get_interaction_response(&ctx.http).await?;
        invite_poll
            .invite_poll
            .update_message(&mut *transaction, &message)
            .await?;

        transaction.commit().await?;

        Ok(())
    }

    fn register() -> Option<CreateApplicationCommand> {
        let mut command = CreateApplicationCommand::default();
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
            });

        Some(command)
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
                        ParseActionError::InvalidOptionValue(ACTION_ID, name.into(), err.into())
                    })?;
                    user_id = Some(value);
                }
                name @ DURATION_OPTION_NAME => {
                    let value = resolve_option!(ACTION_ID, &opt.resolved, String, name)?;
                    let value = humantime::parse_duration(value).map_err(|err| {
                        ParseActionError::InvalidOptionValue(ACTION_ID, name.into(), err.into())
                    })?;
                    duration = Some(value);
                }
                other => {
                    return Err(ParseActionError::UnknownOption(ACTION_ID, other.to_owned()));
                }
            }
        }

        let user_id = user_id.ok_or(ParseActionError::MissingOption(
            ACTION_ID,
            USER_ID_OPTION_NAME,
        ))?;
        let duration = duration.ok_or(ParseActionError::MissingOption(
            ACTION_ID,
            USER_ID_OPTION_NAME,
        ))?;

        let guild_id = interaction
            .guild_id
            .ok_or(ParseActionError::InvalidInteractionKind)
            .map(Into::into)?;

        Ok(Self {
            interaction: interaction.clone(),
            guild_id,
            user_id,
            duration,
        })
    }
}
