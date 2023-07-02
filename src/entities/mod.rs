mod invite_poll;
mod invite_poll_vote_count;
mod invite_poll_vote_submission;

pub use invite_poll::InvitePoll;
pub use invite_poll_vote_count::InvitePollVoteCount;
pub use invite_poll_vote_submission::InvitePollVoteSubmission;

#[derive(Clone, Copy, Debug, sqlx::Type)]
#[sqlx(type_name = "invite_poll_status", rename_all = "lowercase")]
pub enum InvitePollStatus {
    Open,
    Closed,
}

#[derive(Clone, Copy, Debug, sqlx::Type)]
#[sqlx(type_name = "invite_poll_vote", rename_all = "lowercase")]
pub enum InvitePollVote {
    Yes,
    Maybe,
    No,
}
