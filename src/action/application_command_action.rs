use std::str::FromStr;

use base64::Engine;
use serenity::{
    model::prelude::{
        command::{Command, CommandOptionType},
        interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
        GuildId, ReactionType, UserId,
    },
    prelude::Context,
};

use crate::{entities::InvitePoll, error::Error, handler::Handler, util::colors};

static BASE64: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD_NO_PAD;

#[derive(Debug)]
pub enum ApplicationCommandAction {
    CreateInvitePoll { guild_id: GuildId, user_id: UserId },
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
        debug!("{:?}", self);

        match self {
            ApplicationCommandAction::CreateInvitePoll { guild_id, user_id } => {
                let invite_poll =
                    InvitePoll::create(&handler.pool, guild_id.to_owned(), user_id.to_owned())
                        .await?;

                // generate response
                let user = user_id.to_user(&ctx.http).await?;

                interaction
                    .create_interaction_response(&ctx.http, |resp| {
                        resp.kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|data| {
                                data.embed(|embed| {
                                    embed
                                        .color(colors::PASTEL_GREEN)
                                        .title("Invite Poll")
                                        .thumbnail(user.face())
                                        .field("Poll Id", BASE64.encode(invite_poll.id), true)
                                        .field("User", &user.name, true)
                                })
                                .components(|component| {
                                    component.create_action_row(|row| {
                                        row.create_button(|btn| {
                                            btn.custom_id("democracy.invite-poll-vote.yes")
                                                .label("Yes")
                                                .emoji(ReactionType::from('🟢'))
                                        })
                                        .create_button(|btn| {
                                            btn.custom_id("democracy.invite-poll-vote.maybe")
                                                .label("Maybe")
                                                .emoji(ReactionType::from('🟡'))
                                        })
                                        .create_button(
                                            |btn| {
                                                btn.custom_id("democracy.invite-poll-vote.no")
                                                    .label("No")
                                                    .emoji(ReactionType::from('🔴'))
                                            },
                                        )
                                    })
                                })
                            })
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
                    Some(user_id) => Ok(Self::CreateInvitePoll { guild_id, user_id }),
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
