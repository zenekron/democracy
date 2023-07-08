use std::{fmt::Display, ops::Deref, str::FromStr};

use serenity::builder::{
    CreateComponents, CreateEmbed, CreateInteractionResponseData, EditMessage,
};
use sqlx::Postgres;

#[derive(Debug, Default)]
pub struct MessageRenderer {
    components: Option<CreateComponents>,
    embeds: Vec<CreateEmbed>,
}

impl MessageRenderer {
    pub fn set_components(&mut self, components: CreateComponents) -> &mut Self {
        self.components = Some(components);
        self
    }

    pub fn set_embeds(&mut self, embeds: impl IntoIterator<Item = CreateEmbed>) -> &mut Self {
        self.embeds = embeds.into_iter().collect();
        self
    }

    pub fn render_create_interaction_response_data<'a, 'b>(
        self,
        data: &'a mut CreateInteractionResponseData<'b>,
    ) -> &'a mut CreateInteractionResponseData<'b> {
        if let Some(components) = self.components {
            data.set_components(components);
        }
        data.set_embeds(self.embeds);

        data
    }

    pub fn render_edit_message<'a, 'b>(
        self,
        data: &'a mut EditMessage<'b>,
    ) -> &'a mut EditMessage<'b> {
        if let Some(components) = self.components {
            data.set_components(components);
        }
        data.set_embeds(self.embeds);

        data
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

impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<@{}>", self.0.as_u64())
    }
}
