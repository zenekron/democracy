#[derive(Clone, Copy, Debug, sqlx::Type)]
#[sqlx(type_name = "invite_poll_vote", rename_all = "lowercase")]
pub enum InvitePollVote {
    Yes,
    Maybe,
    No,
}
