use std::{ops::Deref, str::FromStr};

use serenity::builder::{
    CreateComponents, CreateEmbed, CreateInteractionResponseData, EditMessage,
};
use sqlx::Postgres;

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
                buf.extend(self.0.as_u64().to_string().as_bytes());
                sqlx::encode::IsNull::No
            }
        }
    };
}

wrap_discord_id!(GuildId);
wrap_discord_id!(UserId);
wrap_discord_id!(ChannelId);
wrap_discord_id!(MessageId);

pub enum MessageRenderer<'a, 'b> {
    CreateInteractionResponseData(&'a mut CreateInteractionResponseData<'b>),
    EditMessage(&'a mut EditMessage<'b>),
}

impl<'a, 'b> MessageRenderer<'a, 'b> {
    pub fn set_components(&mut self, components: CreateComponents) -> &mut Self {
        match self {
            MessageRenderer::CreateInteractionResponseData(message) => {
                message.set_components(components);
            }
            MessageRenderer::EditMessage(message) => {
                message.set_components(components);
            }
        }

        self
    }

    pub fn set_embeds(&mut self, embeds: impl IntoIterator<Item = CreateEmbed>) -> &mut Self {
        match self {
            MessageRenderer::CreateInteractionResponseData(message) => {
                message.set_embeds(embeds);
            }
            MessageRenderer::EditMessage(message) => {
                message.set_embeds(embeds.into_iter().collect());
            }
        }

        self
    }
}

impl<'a, 'b> From<&'a mut CreateInteractionResponseData<'b>> for MessageRenderer<'a, 'b> {
    fn from(value: &'a mut CreateInteractionResponseData<'b>) -> Self {
        Self::CreateInteractionResponseData(value)
    }
}

impl<'a, 'b> From<&'a mut EditMessage<'b>> for MessageRenderer<'a, 'b> {
    fn from(value: &'a mut EditMessage<'b>) -> Self {
        Self::EditMessage(value)
    }
}
