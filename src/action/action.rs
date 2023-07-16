use serenity::{
    async_trait, builder::CreateApplicationCommands, model::prelude::Interaction, prelude::Context,
};

use crate::error::Error;

use super::ParseActionError;

#[async_trait]
pub trait Action: for<'a> TryFrom<&'a Interaction, Error = ParseActionError> {
    async fn execute(&self, ctx: &Context) -> Result<(), Error>;

    fn register(commands: &mut CreateApplicationCommands) -> &mut CreateApplicationCommands {
        commands
    }
}
