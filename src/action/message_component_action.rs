use serenity::{
    model::prelude::{
        interaction::{message_component::MessageComponentInteraction, InteractionResponseType},
        UserId,
    },
    prelude::Context,
};

use crate::{
    entities::{InvitePollId, InvitePollVote, InvitePollVoteSubmission, InvitePollWithVoteCount},
    error::Error,
};

#[derive(Debug)]
pub enum MessageComponentAction {
    SubmitInvitePollVote {
        invite_poll_id: InvitePollId,
        /// Submitter's Id
        user_id: UserId,
        vote: InvitePollVote,
    },
}

impl MessageComponentAction {
    pub async fn execute(
        &self,
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
                let _invite_poll_vote_submission =
                    InvitePollVoteSubmission::upsert(invite_poll_id, user_id, vote.to_owned())
                        .await?;

                // load the poll
                let invite_poll = InvitePollWithVoteCount::find_by_id(invite_poll_id)
                    .await?
                    .ok_or_else(|| Error::InvitePollNotFound(invite_poll_id.to_owned()))?;

                // re-render message
                let render = invite_poll.create_interaction_response(ctx.clone()).await?;
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
                    .and_then(|s| {
                        s.strip_prefix('`')
                            .unwrap_or(s)
                            .strip_suffix('`')
                            .unwrap_or(s)
                            .parse::<InvitePollId>()
                    })?;
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
                    .and_then(|s| {
                        s.strip_prefix('`')
                            .unwrap_or(s)
                            .strip_suffix('`')
                            .unwrap_or(s)
                            .parse::<InvitePollId>()
                    })?;
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
                    .and_then(|s| {
                        s.strip_prefix('`')
                            .unwrap_or(s)
                            .strip_suffix('`')
                            .unwrap_or(s)
                            .parse::<InvitePollId>()
                    })?;
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
