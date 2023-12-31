mod guild;
mod invite_poll;
mod invite_poll_vote_submission;
mod invite_poll_with_vote_count;

pub use guild::*;
pub use invite_poll::*;
pub use invite_poll_vote_submission::*;
pub use invite_poll_with_vote_count::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "invite_poll_outcome", rename_all = "lowercase")]
pub enum InvitePollOutcome {
    Allow,
    Deny,
}

#[derive(Clone, Copy, Debug, sqlx::Type, strum::EnumString)]
#[sqlx(type_name = "invite_poll_vote", rename_all = "lowercase")]
#[strum(serialize_all = "snake_case")]
pub enum InvitePollVote {
    Yes,
    No,
}
