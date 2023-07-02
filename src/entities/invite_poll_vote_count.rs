use sqlx::PgPool;
use uuid::Uuid;

use crate::error::Error;

pub struct InvitePollVoteCount {
    pub invite_poll_id: Uuid,
    pub yes_count: i64,
    pub maybe_count: i64,
    pub no_count: i64,
}

impl InvitePollVoteCount {
    pub async fn compute(pool: &PgPool, invite_poll_id: &Uuid) -> Result<Self, Error> {
        let res = sqlx::query_as!(
            Self,
            r#"
                SELECT
                    invite_poll_id AS "invite_poll_id!",
                    yes_count AS "yes_count!",
                    maybe_count AS "maybe_count!",
                    no_count AS "no_count!"
                FROM invite_poll_vote_count
                WHERE invite_poll_id = $1;
          "#,
            invite_poll_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(res.unwrap_or_else(|| InvitePollVoteCount {
            invite_poll_id: invite_poll_id.to_owned(),
            yes_count: 0,
            maybe_count: 0,
            no_count: 0,
        }))
    }
}
