use serenity::{
    async_trait,
    model::prelude::{
        message_component::MessageComponentInteraction, Interaction, InteractionResponseType,
    },
    prelude::Context,
};

use crate::{
    entities::{InvitePollId, InvitePollVote, InvitePollVoteSubmission, InvitePollWithVoteCount},
    error::Error,
    util::serenity::UserId,
    POOL,
};

use super::{Action, ParseActionError};

const ACTION_ID: &'static str = "democracy.invite-poll-vote";
const POLL_ID_FIELD_NAME: &'static str = "Poll Id";

#[derive(Debug)]
pub struct SubmitInvitePollVote {
    interaction: MessageComponentInteraction,
    invite_poll_id: InvitePollId,
    /// Submitter's Id
    user_id: UserId,
    vote: InvitePollVote,
}

#[async_trait]
impl Action for SubmitInvitePollVote {
    async fn execute(&self, ctx: &Context) -> Result<(), Error> {
        let pool = POOL.get().expect("the Pool to be initialized");

        // submit the vote
        let _invite_poll_vote_submission = InvitePollVoteSubmission::create_or_update(
            pool,
            &self.invite_poll_id,
            &self.user_id,
            self.vote,
        )
        .await?;

        // load the poll
        let invite_poll = InvitePollWithVoteCount::find_by_id(&self.invite_poll_id)
            .await?
            .ok_or_else(|| Error::InvitePollNotFound(self.invite_poll_id.to_owned()))?;

        // re-render message
        let render = invite_poll.create_renderer(ctx.clone()).await?;
        self.interaction
            .create_interaction_response(&ctx.http, |resp| {
                resp.kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|data| {
                        render(&mut data.into());
                        data
                    })
            })
            .await?;

        Ok(())
    }
}

impl<'a> TryFrom<&'a Interaction> for SubmitInvitePollVote {
    type Error = ParseActionError;

    fn try_from(value: &'a Interaction) -> Result<Self, Self::Error> {
        let interaction = value
            .as_message_component()
            .ok_or(ParseActionError::MismatchedAction)?;
        if !interaction.data.custom_id.starts_with(ACTION_ID) {
            return Err(ParseActionError::MismatchedAction);
        }

        let invite_poll_id = {
            let field = interaction
                .message
                .embeds
                .iter()
                .flat_map(|embed| embed.fields.iter())
                .find(|field| field.name == POLL_ID_FIELD_NAME)
                .ok_or(ParseActionError::MissingOption(ACTION_ID, "poll-id"))?;

            let val = field.value.as_str();
            let val = val
                .strip_prefix('`')
                .unwrap_or(val)
                .strip_suffix('`')
                .unwrap_or(val);

            val.parse::<InvitePollId>().map_err(|err| {
                ParseActionError::InvalidOptionValue(ACTION_ID, "poll-id".to_string(), err.into())
            })?
        };

        let vote = {
            let vote = interaction
                .data
                .custom_id
                .strip_prefix([ACTION_ID, "."].concat().as_str())
                .ok_or_else(|| {
                    ParseActionError::InvalidOptionValue(
                        ACTION_ID,
                        "vote".into(),
                        "id did not match the current action's".into(),
                    )
                })?;

            vote.parse::<InvitePollVote>().map_err(|err| {
                ParseActionError::InvalidOptionValue(ACTION_ID, "vote".into(), err.into())
            })?
        };

        let user_id = UserId::from(interaction.user.id);

        Ok(Self {
            interaction: interaction.clone(),
            invite_poll_id,
            user_id,
            vote,
        })
    }
}
