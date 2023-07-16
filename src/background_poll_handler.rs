use std::time::Duration;

use serenity::{model::prelude::UserId, prelude::Context};
use sqlx::PgPool;
use tokio::time::{interval, Interval};

use crate::{
    entities::{Guild, InvitePollOutcome, InvitePollWithVoteCount},
    error::Error,
    util::serenity::ErrorExt,
    POOL,
};

#[derive(Debug, thiserror::Error)]
enum InvitePollMessage {
    #[error("{0} users opposed")]
    AtLeastOneOpposition(i64),

    #[error("the quorum was not reached: {0}/{1} users voted")]
    QuorumNotReached(i64, i64),

    #[error("{0}")]
    InviteUrl(String),
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

    pub async fn start(&mut self) {
        let pool = POOL.get().expect("the Pool to be initialized");

        loop {
            self.interval.tick().await;
            match self.tick(pool).await {
                Ok(()) => {}
                Err(err) => error!("failed to tick expired polls: {:?}", err),
            }
        }
    }

    async fn tick(&self, pool: &PgPool) -> Result<(), Error> {
        let polls = InvitePollWithVoteCount::find_expired(pool).await?;
        for mut poll in polls {
            match self.close_poll(pool, &mut poll).await {
                Ok(()) => {}
                Err(err) => error!(
                    "failed to tick expired poll {}: {:?}",
                    poll.invite_poll.id, err
                ),
            }
            {}
        }

        Ok(())
    }

    async fn close_poll(
        &self,
        pool: &PgPool,
        poll: &mut InvitePollWithVoteCount,
    ) -> Result<(), Error> {
        let http = &self.ctx.http;

        debug!("closing poll {:?}", poll);

        let guild = poll.invite_poll.guild_id.to_partial_guild(http).await?;
        let settings = Guild::find_by_id(pool, &poll.invite_poll.guild_id)
            .await?
            .ok_or_else(|| Error::GuildNotFound(poll.invite_poll.guild_id.clone()))?;

        let guild_user_count = {
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

        let (outcome, mut message) = {
            let quorum = (guild_user_count as f32 * settings.invite_poll_quorum).ceil() as i64;
            let count = poll.no_count + poll.yes_count;

            if poll.no_count > 0 {
                (
                    InvitePollOutcome::Deny,
                    Some(InvitePollMessage::AtLeastOneOpposition(poll.no_count)),
                )
            } else if count < quorum {
                (
                    InvitePollOutcome::Deny,
                    Some(InvitePollMessage::QuorumNotReached(count, quorum)),
                )
            } else {
                (InvitePollOutcome::Allow, None)
            }
        };

        debug!(
            "closing poll {} with outcome {:?} and message {}",
            poll.invite_poll.id,
            outcome,
            message
                .as_ref()
                .map(|r| r.to_string())
                .unwrap_or("".to_string())
        );

        if outcome == InvitePollOutcome::Allow {
            let invite = settings
                .invite_channel_id
                .create_invite(http, |invite| invite.unique(true).max_uses(1))
                .await?;

            'send_message: {
                // try sending the server invite directly to the user
                let pm = poll.invite_poll.invitee.create_dm_channel(http).await?;
                let res = pm.send_message(http, |msg| {
                    msg.content(format!(
                            "Hello! You have been invited by {} to **{}**!\nAccept the following invite to join them!\n{}",
                            poll.invite_poll.inviter,
                            guild.name,
                            invite.url()
                            ))
                })
                .await;

                match res {
                    Ok(_) => {
                        break 'send_message;
                    }
                    Err(err) if err.is_cannot_send_messages_to_this_user_error() => {
                        // continue
                    }
                    Err(err) => {
                        return Err(err.into());
                    }
                }

                // if that fails send the invite to the inviter
                let pm = poll.invite_poll.inviter.create_dm_channel(http).await?;
                let res = pm.send_message(http, |msg| {
                    msg.content(format!(
                            "Hello! The invite poll you started in **{}** for {} has ended successfully!\nPlease send them the following invite url!\n{}",
                            guild.name,
                            poll.invite_poll.invitee,
                            invite.url()
                            ))
                })
                .await;

                match res {
                    Ok(_) => {
                        break 'send_message;
                    }
                    Err(err) if err.is_cannot_send_messages_to_this_user_error() => {
                        // continue
                    }
                    Err(err) => {
                        return Err(err.into());
                    }
                }

                // if that fails send it to the server owner
                let pm = guild.owner_id.create_dm_channel(http).await?;
                let res = pm.send_message(http, |msg| {
                    msg.content(format!(
                            "Hello! The invite poll in **{}** by {} for {} has ended successfully!\nPlease send them the following invite url!\n{}",
                            guild.name,
                            poll.invite_poll.inviter,
                            poll.invite_poll.invitee,
                            invite.url()
                            ))
                })
                .await;

                match res {
                    Ok(_) => {
                        break 'send_message;
                    }
                    Err(err) if err.is_cannot_send_messages_to_this_user_error() => {
                        // continue
                    }
                    Err(err) => {
                        return Err(err.into());
                    }
                }

                // if everything fails fall back to putting it in the embed
                message = Some(InvitePollMessage::InviteUrl(invite.url()));
            }
        }

        poll.invite_poll
            .close(pool, outcome, message.map(|r| r.to_string()))
            .await?;

        match (&poll.invite_poll.channel_id, &poll.invite_poll.message_id) {
            (Some(channel_id), Some(message_id)) => {
                let renderer = poll.create_renderer(self.ctx.clone()).await?;
                channel_id
                    .edit_message(http, message_id, |edit| renderer.render_edit_message(edit))
                    .await?;
            }
            _ => error!(
                "could not update poll {} because either the `channel_id` or `message_id` are missing",
                poll.invite_poll.id
            ),
        }

        Ok(())
    }
}
