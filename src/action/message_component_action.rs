use serenity::{
    model::prelude::{interaction::message_component::MessageComponentInteraction, Interaction},
    prelude::Context,
};

use crate::error::Error;

use super::{Action, SubmitInvitePollVote};

#[derive(Debug)]
pub enum MessageComponentAction {
    SubmitInvitePollVote(SubmitInvitePollVote),
}

impl MessageComponentAction {
    pub async fn execute(&self, ctx: Context) -> Result<(), Error> {
        match self {
            MessageComponentAction::SubmitInvitePollVote(action) => action.execute(&ctx).await?,
        }

        Ok(())
    }
}

impl TryFrom<&MessageComponentInteraction> for MessageComponentAction {
    type Error = Error;

    fn try_from(interaction: &MessageComponentInteraction) -> Result<Self, Self::Error> {
        match interaction.data.custom_id.as_str() {
            id if id.starts_with(SubmitInvitePollVote::ID) => {
                let action = SubmitInvitePollVote::try_from(&Interaction::MessageComponent(
                    interaction.clone(),
                ))?;
                Ok(Self::SubmitInvitePollVote(action))
            }
            _ => todo!(),
        }
    }
}
