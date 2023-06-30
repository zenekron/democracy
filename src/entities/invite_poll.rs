use base64::Engine;
use chrono::{DateTime, Utc};
use serenity::{
    model::{
        application::interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
        prelude::{GuildId, ReactionType, UserId},
    },
    prelude::Context,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::Error;

static BASE64: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD_NO_PAD;

#[derive(Debug, sqlx::FromRow)]
pub struct InvitePoll {
    pub id: Uuid,
    guild_id: i64,
    user_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl InvitePoll {
    pub async fn create(pool: &PgPool, guild_id: GuildId, user_id: UserId) -> Result<Self, Error> {
        let res = sqlx::query_as!(
            Self,
            r#"
                INSERT INTO invite_poll(guild_id, user_id)
                VALUES ($1, $2)
                RETURNING id, guild_id AS "guild_id: _", user_id AS "user_id: _", created_at, updated_at;
            "#,
            guild_id.0 as i64,
            user_id.0 as i64
        ).fetch_one(pool).await?;

        Ok(res)
    }

    pub async fn create_interaction_response(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<(), Error> {
        let user = self.user_id().to_user(&ctx.http).await?;

        command
            .create_interaction_response(&ctx.http, |resp| {
                resp.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|data| {
                        data.embed(|embed| {
                            embed
                                .color(colors::PASTEL_GREEN)
                                .title("Invite Poll")
                                .thumbnail(user.face())
                                .field("Poll Id", BASE64.encode(self.id), true)
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
                                .create_button(|btn| {
                                    btn.custom_id("democracy.invite-poll-vote.no")
                                        .label("No")
                                        .emoji(ReactionType::from('🔴'))
                                })
                            })
                        })
                    })
            })
            .await
            .map_err(Into::into)
    }
}

impl InvitePoll {
    pub fn guild_id(&self) -> GuildId {
        GuildId(self.guild_id as u64)
    }

    pub fn user_id(&self) -> UserId {
        UserId(self.user_id as u64)
    }
}

mod colors {
    use serenity::utils::Color;

    // https://www.colorhexa.com/77dd77
    pub static PASTEL_GREEN: Color = Color::new(0x77dd77);
}
