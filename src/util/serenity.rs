use std::{fmt::Display, ops::Deref, str::FromStr};

use async_trait::async_trait;
use serenity::{
    builder::{
        CreateActionRow, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
        CreateMessage, EditMessage,
    },
    http::{CacheHttp, Http, StatusCode},
    model::prelude::{Interaction, PartialGuild},
};
use sqlx::Postgres;

pub trait ErrorExt {
    fn as_http_error(&self) -> Option<&serenity::http::HttpError>;

    fn is_cannot_send_messages_to_this_user_error(&self) -> bool;
}

impl ErrorExt for serenity::Error {
    fn as_http_error(&self) -> Option<&serenity::http::HttpError> {
        match self {
            serenity::Error::Http(err) => Some(err),
            _ => None,
        }
    }

    fn is_cannot_send_messages_to_this_user_error(&self) -> bool {
        self.as_http_error()
            .and_then(|err| err.as_unsuccessful_request())
            .map(|err| err.error.code == 50007)
            .unwrap_or(false)
    }
}

pub trait HttpErrorExt {
    fn as_unsuccessful_request(&self) -> Option<&serenity::http::ErrorResponse>;
}

impl HttpErrorExt for serenity::http::HttpError {
    fn as_unsuccessful_request(&self) -> Option<&serenity::http::ErrorResponse> {
        match self {
            serenity::prelude::HttpError::UnsuccessfulRequest(err) => Some(err),
            _ => None,
        }
    }
}

#[async_trait]
pub trait GuildExt {
    async fn is_member(
        &self,
        cache_http: impl CacheHttp,
        user_id: impl Into<serenity::model::id::UserId> + Send,
    ) -> Result<bool, serenity::Error>;
}

#[async_trait]
impl GuildExt for PartialGuild {
    async fn is_member(
        &self,
        cache_http: impl CacheHttp,
        user_id: impl Into<serenity::model::id::UserId> + Send,
    ) -> Result<bool, serenity::Error> {
        let res = self.member(cache_http, user_id.into()).await;
        match res {
            Ok(_) => Ok(true),
            Err(ref err) if matches!(err, serenity::Error::Http(http) if http.status_code() == Some(StatusCode::NOT_FOUND)) => {
                Ok(false)
            }
            Err(err) => Err(err),
        }
    }
}

#[async_trait]
pub trait InteractionExt {
    async fn create_interaction_response<H>(
        &self,
        http: H,
        response: CreateInteractionResponse,
    ) -> Result<(), serenity::Error>
    where
        H: AsRef<Http> + Send + Sync;
}

#[async_trait]
impl InteractionExt for Interaction {
    async fn create_interaction_response<H>(
        &self,
        http: H,
        response: CreateInteractionResponse,
    ) -> Result<(), serenity::Error>
    where
        H: AsRef<Http> + Send + Sync,
    {
        match self {
            Interaction::Ping(_) => Ok(()),
            Interaction::Command(interaction) => {
                interaction.create_response(http.as_ref(), response).await
            }
            Interaction::Autocomplete(_) => Ok(()),
            Interaction::Component(interaction) => {
                interaction.create_response(http.as_ref(), response).await
            }

            Interaction::Modal(interaction) => {
                interaction.create_response(http.as_ref(), response).await
            }
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Default)]
pub struct MessageRenderer {
    components: Option<Vec<CreateActionRow>>,
    embeds: Vec<CreateEmbed>,
}

impl MessageRenderer {
    pub fn set_components(&mut self, components: Vec<CreateActionRow>) -> &mut Self {
        self.components = Some(components);
        self
    }

    pub fn set_embeds(&mut self, embeds: impl IntoIterator<Item = CreateEmbed>) -> &mut Self {
        self.embeds = embeds.into_iter().collect();
        self
    }

    pub fn render_create_interaction_response_data<'a>(
        self,
        mut data: CreateInteractionResponseMessage,
    ) -> CreateInteractionResponseMessage {
        if let Some(components) = self.components {
            data = data.components(components);
        }
        data.embeds(self.embeds)
    }

    pub fn render_create_message<'a>(self, mut data: CreateMessage) -> CreateMessage {
        if let Some(components) = self.components {
            data = data.components(components);
        }
        data.embeds(self.embeds)
    }

    pub fn render_edit_message<'a>(self, mut data: EditMessage) -> EditMessage {
        if let Some(components) = self.components {
            data = data.components(components);
        }
        data.embeds(self.embeds)
    }
}

macro_rules! wrap_discord_id {
    ($id:ident) => {
        #[derive(Clone, Debug)]
        pub struct $id(serenity::model::id::$id);

        impl Deref for $id {
            type Target = serenity::model::id::$id;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl From<serenity::model::id::$id> for $id {
            fn from(value: serenity::model::id::$id) -> Self {
                Self(value)
            }
        }

        impl Into<serenity::model::id::$id> for &$id {
            fn into(self) -> serenity::model::id::$id {
                self.0
            }
        }

        impl FromStr for $id {
            type Err = <u64 as FromStr>::Err;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(s.parse::<u64>()?.into()))
            }
        }

        impl sqlx::Type<Postgres> for $id {
            fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
                <String as sqlx::Type<Postgres>>::type_info()
            }

            fn compatible(ty: &<Postgres as sqlx::Database>::TypeInfo) -> bool {
                <String as sqlx::Type<Postgres>>::compatible(ty)
            }
        }

        impl<'r> sqlx::Decode<'r, Postgres> for $id {
            fn decode(
                value: <Postgres as sqlx::database::HasValueRef<'r>>::ValueRef,
            ) -> Result<Self, sqlx::error::BoxDynError> {
                let s = value.as_str()?;
                Ok(s.parse()?)
            }
        }

        impl<'q> sqlx::Encode<'q, Postgres> for $id {
            fn encode_by_ref(
                &self,
                buf: &mut <Postgres as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
            ) -> sqlx::encode::IsNull {
                buf.extend(self.0.get().to_string().as_bytes());
                sqlx::encode::IsNull::No
            }
        }
    };
}

wrap_discord_id!(GuildId);
wrap_discord_id!(UserId);
wrap_discord_id!(ChannelId);
wrap_discord_id!(MessageId);

impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<@{}>", self.0.get())
    }
}
