pub use application_command_action::ApplicationCommandAction;
pub use configure::*;
pub use create_invite_poll::*;
pub use message_component_action::MessageComponentAction;
use serenity::{
    async_trait,
    builder::CreateApplicationCommand,
    model::prelude::{application_command::CommandDataOptionValue, Interaction},
    prelude::Context,
};

use crate::error::Error;

mod application_command_action;
mod configure;
mod create_invite_poll;
mod message_component_action;

#[macro_export]
macro_rules! resolve_option {
    ($resolved:expr, $kind:ident, $name:expr) => {{
        use serenity::model::prelude::application_command::CommandDataOptionValue;

        if let Some(CommandDataOptionValue::$kind(val)) = $resolved {
            Ok(val)
        } else {
            Err(ParseActionError::InvalidOptionKind(
                Self::ID,
                $name.into(),
                stringify!($kind),
                $resolved.clone(),
            ))
        }
    }};
}

#[async_trait]
pub trait Action: for<'a> TryFrom<&'a Interaction, Error = ParseActionError> {
    const ID: &'static str;

    async fn execute(&self, ctx: &Context) -> Result<(), Error>;

    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseActionError {
    #[error("")]
    InvalidInteractionKind,

    #[error("unknown option `{1}` for command `{0}`")]
    UnknownOption(&'static str, String),

    #[error("no value provided for option `{1}` of command `{0}`")]
    MissingOption(&'static str, &'static str),

    #[error(
        "expected value for option `{1}` of command `{0}` to be a `{2}` but found `{3:?}` instead"
    )]
    InvalidOptionKind(
        &'static str,
        String,
        &'static str,
        Option<CommandDataOptionValue>,
    ),

    #[error("invalid value for option `{1}` of command `{0}`: `{2}`")]
    InvalidOptionValue(&'static str, String, Box<dyn std::error::Error>),
}
