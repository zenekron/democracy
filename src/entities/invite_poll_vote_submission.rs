use chrono::{DateTime, Utc};
use serenity::model::prelude::UserId;

use crate::{error::Error, POOL};

use super::{InvitePollId, InvitePollVote};

#[derive(Debug, sqlx::FromRow)]
pub struct InvitePollVoteSubmission {
    pub invite_poll_id: InvitePollId,
    user_id: i64,
    pub vote: InvitePollVote,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl InvitePollVoteSubmission {
    pub async fn upsert(
        invite_poll_id: &InvitePollId,
        user_id: &UserId,
        vote: InvitePollVote,
    ) -> Result<Self, Error> {
        let pool = POOL.get().expect("the Pool to be initialized");
        let res = sqlx::query_as::<_, Self>(
            r#"
                INSERT INTO invite_poll_vote_submission(invite_poll_id, user_id, vote)
                VALUES ($1, $2, $3)
                ON CONFLICT (invite_poll_id, user_id) DO UPDATE SET vote = EXCLUDED.vote
                RETURNING *;
            "#,
        )
        .bind(invite_poll_id.0)
        .bind(user_id.0 as i64)
        .bind(vote as InvitePollVote)
        .fetch_one(pool)
        .await?;

        Ok(res)
    }
}
