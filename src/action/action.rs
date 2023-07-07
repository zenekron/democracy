use serenity::{
    async_trait, builder::CreateApplicationCommand, model::prelude::Interaction, prelude::Context,
};

use crate::error::Error;

use super::ParseActionError;

#[async_trait]
pub trait Action: for<'a> TryFrom<&'a Interaction, Error = ParseActionError> {
    const ID: &'static str;

    async fn execute(&self, ctx: &Context) -> Result<(), Error>;

    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
    }
}
