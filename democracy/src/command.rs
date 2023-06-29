use std::str::FromStr;

use chrono::Utc;
use sea_orm::{ActiveValue, EntityTrait};
use serenity::{
    model::prelude::{
        command::{Command as SerenityCommand, CommandOptionType},
        interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
        ReactionType, UserId,
    },
    prelude::Context,
};
use uuid::Uuid;

use crate::{entity::invite_poll, error::Error, handler::Handler};

#[derive(Debug)]
pub enum Command {
    Ping,
    Invite { user_id: UserId },
}

impl Command {
    pub async fn register(ctx: Context) -> Result<(), Error> {
        let _ping = SerenityCommand::create_global_application_command(&ctx.http, |cmd| {
            cmd.name("ping").description("Ping")
        })
        .await?;

        let _invite = SerenityCommand::create_global_application_command(&ctx.http, |cmd| {
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
        command: &ApplicationCommandInteraction,
    ) -> Result<(), Error> {
        debug!("{:?}", self);

        match self {
            Command::Ping => {
                command
                    .create_interaction_response(&ctx.http, |resp| {
                        resp.kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|data| data.title("Pong").content("pong"))
                    })
                    .await?;

                Ok(())
            }

            Command::Invite { user_id } => {
                let user = user_id.to_user(&ctx.http).await?;
                debug!("user: {:?}", user);

                let guild_id = command
                    .guild_id
                    .ok_or_else(|| Error::GuildCommandNotInGuild("invite".to_string()))?;

                let invite = invite_poll::ActiveModel {
                    id: ActiveValue::Set(Uuid::new_v4()),
                    guild_id: ActiveValue::Set(guild_id.0 as i64),
                    user_id: ActiveValue::Set(user_id.0 as i64),
                    created_at: ActiveValue::Set(Utc::now().naive_utc()),
                    updated_at: ActiveValue::Set(Utc::now().naive_utc()),
                };
                invite_poll::Entity::insert(invite)
                    .exec(&handler.database)
                    .await?;

                command
                    .create_interaction_response(&ctx.http, |resp| {
                        resp.kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|data| {
                                data.embed(|embed| {
                                    embed
                                        .color(colors::PASTEL_GREEN)
                                        .title("Invite Poll")
                                        .thumbnail(user.face())
                                        .field("User", &user.name, true)
                                })
                                .components(|component| {
                                    component.create_action_row(|row| {
                                        row.create_button(|btn| {
                                            btn.custom_id("invite.vote.yes")
                                                .label("Yes")
                                                .emoji(ReactionType::from('🟢'))
                                        })
                                        .create_button(|btn| {
                                            btn.custom_id("invite.vote.maybe")
                                                .label("Maybe")
                                                .emoji(ReactionType::from('🟡'))
                                        })
                                        .create_button(
                                            |btn| {
                                                btn.custom_id("invite.vote.no")
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

impl TryFrom<&ApplicationCommandInteraction> for Command {
    type Error = Error;

    fn try_from(value: &ApplicationCommandInteraction) -> Result<Self, Self::Error> {
        match value.data.name.as_str() {
            "ping" => Ok(Command::Ping),
            "invite" => {
                let mut user_id: Option<UserId> = None;

                for opt in &value.data.options {
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

                match user_id {
                    Some(user_id) => Ok(Self::Invite { user_id }),
                    None => Err(Error::MissingCommandOption(
                        "invite".to_owned(),
                        "user_id".to_owned(),
                    )),
                }
            }

            other => Err(Error::UnknownCommand(other.to_owned())),
        }
    }
}

mod colors {
    use serenity::utils::Color;

    // https://www.colorhexa.com/77dd77
    pub static PASTEL_GREEN: Color = Color::new(0x77dd77);
}
