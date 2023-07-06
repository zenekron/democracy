mod application_command_action;
mod configure;
mod message_component_action;

pub use application_command_action::ApplicationCommandAction;
pub use configure::*;
pub use message_component_action::MessageComponentAction;
use serenity::{
    async_trait,
    builder::{CreateApplicationCommand, CreateInteractionResponse},
    model::prelude::{application_command::CommandDataOptionValue, Interaction},
    prelude::Context,
};

use crate::error::Error;

#[async_trait]
pub trait Action: for<'a> TryFrom<&'a Interaction, Error = ParseActionError> {
    const ID: &'static str;

    async fn execute(&self, ctx: &Context) -> Result<CreateInteractionResponse, Error>;

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
}
