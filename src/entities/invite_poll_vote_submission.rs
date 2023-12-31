use chrono::{DateTime, Utc};
use sqlx::{Executor, Postgres};

use crate::{error::Error, util::serenity::UserId};

use super::{InvitePollId, InvitePollVote};

#[derive(Debug, sqlx::FromRow)]
pub struct InvitePollVoteSubmission {
    pub invite_poll_id: InvitePollId,
    pub user_id: UserId,
    pub vote: InvitePollVote,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl InvitePollVoteSubmission {
    pub async fn create_or_update<'c, E>(
        executor: E,
        invite_poll_id: &InvitePollId,
        user_id: &UserId,
        vote: InvitePollVote,
    ) -> Result<Self, Error>
    where
        E: Executor<'c, Database = Postgres>,
    {
        let res = sqlx::query_as::<_, Self>(
            r#"
                INSERT INTO invite_poll_vote_submission (invite_poll_id, user_id, vote)
                VALUES ($1, $2, $3)
                ON CONFLICT (invite_poll_id, user_id) DO UPDATE SET vote = EXCLUDED.vote
                RETURNING *;
            "#,
        )
        .bind(invite_poll_id)
        .bind(user_id)
        .bind(vote)
        .fetch_one(executor)
        .await?;

        Ok(res)
    }
}
