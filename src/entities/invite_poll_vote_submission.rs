use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::InvitePollVote;

#[derive(Debug, sqlx::FromRow)]
pub struct InvitePollVoteSubmission {
    pub invite_poll_id: Uuid,
    user_id: i64,
    pub vote: InvitePollVote,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
