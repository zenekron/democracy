use std::time::Duration;

use serenity::{model::prelude::UserId, prelude::Context};
use tokio::time::{interval, Interval};

use crate::{
    entities::{Guild, InvitePollOutcome, InvitePollWithVoteCount},
    error::Error,
    POOL,
};

#[derive(Debug, thiserror::Error)]
enum InvitePollFailureReason {
    #[error("{0} users opposed")]
    AtLeastOneOpposition(i64),

    #[error("the quorum was not reached: {0}/{1} users voted")]
    QuorumNotReached(i64, i64),
}

pub struct BackgroundPollHandler {
    ctx: Context,
    interval: Interval,
}

impl BackgroundPollHandler {
    pub fn new(ctx: Context, duration: Duration) -> Self {
        Self {
            ctx,
            interval: interval(duration),
        }
    }

    pub async fn start(&mut self) -> Result<(), Error> {
        let pool = POOL.get().expect("the Pool to be initialized");
        let http = &self.ctx.http;

        loop {
            self.interval.tick().await;

            let polls = InvitePollWithVoteCount::find_expired(pool).await?;
            for mut poll in polls {
                debug!("expired poll: {:?}", poll);

                let guild = poll.invite_poll.guild_id.to_partial_guild(http).await?;

                let settings = Guild::find_by_id(pool, &poll.invite_poll.guild_id)
                    .await?
                    .ok_or_else(|| Error::GuildNotFound(poll.invite_poll.guild_id.clone()))?;

                let guild_users = {
                    let mut max = 0_usize;
                    let mut after: Option<UserId> = None;
                    loop {
                        let page = guild.members(http, None, after).await?;
                        if page.len() == 0 {
                            break;
                        }

                        max += page.iter().filter(|m| m.user.bot == false).count();
                        after = page.last().map(|u| u.user.id);
                    }

                    max
                };

                let (outcome, reason) = {
                    let quorum = (guild_users as f32 * settings.invite_poll_quorum).ceil() as i64;
                    let count = poll.no_count + poll.yes_count;

                    if poll.no_count > 0 {
                        (
                            InvitePollOutcome::Deny,
                            Some(InvitePollFailureReason::AtLeastOneOpposition(poll.no_count)),
                        )
                    } else if count < quorum {
                        (
                            InvitePollOutcome::Deny,
                            Some(InvitePollFailureReason::QuorumNotReached(count, quorum)),
                        )
                    } else {
                        (InvitePollOutcome::Allow, None)
                    }
                };
                debug!(
                    "expired poll outcome: {:?} {}",
                    outcome,
                    reason
                        .as_ref()
                        .map(|reason| reason.to_string())
                        .unwrap_or_else(|| String::new())
                );

                if outcome == InvitePollOutcome::Allow {
                    let invite = settings
                        .invite_channel_id
                        .create_invite(http, |invite| invite.unique(true).max_uses(1))
                        .await?;

                    let pm = poll.invite_poll.user_id.create_dm_channel(http).await?;

                    pm.send_message(http, |msg| msg.content(invite.url()))
                        .await?;
                }

                poll.invite_poll
                    .close(pool, outcome, reason.map(|r| r.to_string()))
                    .await?;

                match (&poll.invite_poll.channel_id, &poll.invite_poll.message_id) {
                    (Some(channel_id), Some(message_id)) => {
                        let renderer = poll.create_renderer(self.ctx.clone()).await?;
                        channel_id
                            .edit_message(http, message_id, |edit| {
                                renderer.render_edit_message(edit)
                            })
                            .await?;
                    }
                    _ => error!("could not update poll {} because either the channel_id or message_id are missing", poll.invite_poll.id),
                }
            }
        }
    }
}
