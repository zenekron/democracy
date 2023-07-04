mod invite_poll;
mod invite_poll_vote_submission;
mod invite_poll_with_vote_count;

pub use invite_poll::InvitePoll;
pub use invite_poll_vote_submission::InvitePollVoteSubmission;
pub use invite_poll_with_vote_count::InvitePollWithVoteCount;

#[derive(Clone, Copy, Debug, sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
pub enum InvitePollOutcome {
    Allow,
    Deny,
}

#[derive(Clone, Copy, Debug, sqlx::Type)]
#[sqlx(type_name = "invite_poll_vote", rename_all = "lowercase")]
pub enum InvitePollVote {
    Yes,
    Maybe,
    No,
}
