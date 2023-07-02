use serenity::{
    model::prelude::{
        interaction::{message_component::MessageComponentInteraction, InteractionResponseType},
        UserId,
    },
    prelude::Context,
};
use uuid::Uuid;

use crate::{
    entities::{InvitePoll, InvitePollVote, InvitePollVoteSubmission},
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
        match self {
            MessageComponentAction::SubmitInvitePollVote {
                invite_poll_id,
                user_id,
                vote,
            } => {
                // submit the vote
                let _invite_poll_vote_submission = InvitePollVoteSubmission::upsert(
                    &handler.pool,
                    invite_poll_id.to_owned(),
                    user_id,
                    vote.to_owned(),
                )
                .await?;

                // load the poll
                let invite_poll = InvitePoll::find_by_id(&handler.pool, invite_poll_id)
                    .await?
                    .ok_or_else(|| Error::InvitePollNotFound(invite_poll_id.to_owned()))?;

                // re-render message
                let render = invite_poll
                    .create_interaction_response(ctx.clone(), &handler.pool)
                    .await?;
                interaction
                    .create_interaction_response(&ctx.http, |resp| {
                        render(resp).kind(InteractionResponseType::UpdateMessage)
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
