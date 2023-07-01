use serenity::{
    model::prelude::{
        interaction::{message_component::MessageComponentInteraction, InteractionResponseType},
        UserId,
    },
    prelude::Context,
};
use uuid::Uuid;

use crate::{
    entities::{InvitePoll, InvitePollVote},
    error::Error,
    handler::Handler,
};

#[derive(Debug)]
pub enum MessageComponentAction {
    SubmitInvitePollVote {
        invite_poll_id: Uuid,
        /// Submitter's Id
        user_id: UserId,
        vote: InvitePollVote,
    },
}

impl MessageComponentAction {
    pub async fn execute(
        &self,
        handler: &Handler,
        ctx: Context,
        interaction: &MessageComponentInteraction,
    ) -> Result<(), Error> {
        debug!("{:?}", self);

        match self {
            MessageComponentAction::SubmitInvitePollVote {
                invite_poll_id,
                user_id,
                vote,
            } => {
                let invite_poll = InvitePoll::find_by_id(&handler.pool, invite_poll_id)
                    .await?
                    .ok_or_else(|| Error::InvitePollNotFound(invite_poll_id.to_owned()))?;
                info!("{:?}", invite_poll);

                let _invite_poll_vote_submission = invite_poll
                    .submit_vote(&handler.pool, user_id, vote.to_owned())
                    .await?;
                info!("{:?}", _invite_poll_vote_submission);

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

    fn try_from(interaction: &MessageComponentInteraction) -> Result<Self, Self::Error> {
        match interaction.data.custom_id.as_str() {
            "democracy.invite-poll-vote.yes" => {
                let invite_poll_id = interaction
                    .message
                    .embeds
                    .iter()
                    .flat_map(|embed| embed.fields.iter())
                    .find(|field| field.name == "Poll Id")
                    .map(|field| field.value.as_str())
                    .ok_or(Error::InvitePollIdNotFound)
                    .and_then(InvitePoll::decode_id)?;
                let user_id = interaction.user.id;

                Ok(Self::SubmitInvitePollVote {
                    invite_poll_id,
                    user_id,
                    vote: InvitePollVote::Yes,
                })
            }
            "democracy.invite-poll-vote.maybe" => {
                let invite_poll_id = interaction
                    .message
                    .embeds
                    .iter()
                    .flat_map(|embed| embed.fields.iter())
                    .find(|field| field.name == "Poll Id")
                    .map(|field| field.value.as_str())
                    .ok_or(Error::InvitePollIdNotFound)
                    .and_then(InvitePoll::decode_id)?;
                let user_id = interaction.user.id;

                Ok(Self::SubmitInvitePollVote {
                    invite_poll_id,
                    user_id,
                    vote: InvitePollVote::Maybe,
                })
            }
            "democracy.invite-poll-vote.no" => {
                let invite_poll_id = interaction
                    .message
                    .embeds
                    .iter()
                    .flat_map(|embed| embed.fields.iter())
                    .find(|field| field.name == "Poll Id")
                    .map(|field| field.value.as_str())
                    .ok_or(Error::InvitePollIdNotFound)
                    .and_then(InvitePoll::decode_id)?;
                let user_id = interaction.user.id;

                Ok(Self::SubmitInvitePollVote {
                    invite_poll_id,
                    user_id,
                    vote: InvitePollVote::No,
                })
            }
            _ => todo!(),
        }
    }
}
