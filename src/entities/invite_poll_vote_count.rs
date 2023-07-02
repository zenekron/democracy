use std::fmt::Display;

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::Error,
    util::{emojis, ProgressBar},
};

pub struct InvitePollCount {
    pub invite_poll_id: Uuid,
    pub yes_count: i64,
    pub maybe_count: i64,
    pub no_count: i64,
}

impl InvitePollCount {
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

        Ok(res.unwrap_or_else(|| InvitePollCount {
            invite_poll_id: invite_poll_id.to_owned(),
            yes_count: 0,
            maybe_count: 0,
            no_count: 0,
        }))
    }
}

impl Display for InvitePollCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max = 2_f32;

        writeln!(
            f,
            "{} {} [{} · {:.0}%]",
            emojis::LARGE_GREEN_CIRCLE,
            ProgressBar(self.yes_count as f32 / max),
            self.yes_count,
            self.yes_count as f32 / max * 100.0
        )?;
        writeln!(
            f,
            "{} {} [{} · {:.0}%]",
            emojis::LARGE_YELLOW_CIRCLE,
            ProgressBar(self.maybe_count as f32 / max),
            self.maybe_count,
            self.maybe_count as f32 / max * 100.0
        )?;
        writeln!(
            f,
            "{} {} [{} · {:.0}%]",
            emojis::LARGE_RED_CIRCLE,
            ProgressBar(self.no_count as f32 / max),
            self.no_count,
            self.no_count as f32 / max * 100.0
        )?;

        Ok(())
    }
}
