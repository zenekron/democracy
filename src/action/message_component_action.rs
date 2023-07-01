use serenity::{
    model::prelude::interaction::{
        message_component::MessageComponentInteraction, InteractionResponseType,
    },
    prelude::Context,
};

use crate::{error::Error, handler::Handler};

static BASE64: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD_NO_PAD;

#[derive(Debug)]
pub enum MessageComponentAction {
    SubmitInvitePollVote,
}

impl MessageComponentAction {
    pub async fn execute(
        &self,
        _handler: &Handler,
        ctx: Context,
        interaction: &MessageComponentInteraction,
    ) -> Result<(), Error> {
        debug!("{:?}", self);

        match self {
            MessageComponentAction::SubmitInvitePollVote => {
                interaction
                    .create_interaction_response(&ctx.http, |resp| {
                        resp.kind(InteractionResponseType::UpdateMessage)
                            .interaction_response_data(|data| data)
                    })
                    .await?;
            }
        }

        Ok(())
    }
}

impl TryFrom<&MessageComponentInteraction> for MessageComponentAction {
    type Error = Error;

    fn try_from(_interaction: &MessageComponentInteraction) -> Result<Self, Self::Error> {
        todo!()
    }
}
