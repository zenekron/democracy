use std::time::Duration;

use serenity::{model::prelude::UserId, prelude::Context};
use tokio::time::{interval, Interval};

use crate::{
    entities::{Guild, InvitePollOutcome, InvitePollWithVoteCount},
    error::Error,
};

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
        let http = &self.ctx.http;

        loop {
            self.interval.tick().await;

            let polls = InvitePollWithVoteCount::find_expired().await?;
            for mut poll in polls {
                debug!("expired poll: {:?}", poll);

                let guild = poll.invite_poll.guild_id.to_partial_guild(http).await?;

                let settings = Guild::find_by_id(&poll.invite_poll.guild_id)
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

                let outcome = {
                    let required_votes = (guild_users as f32
                        * (settings.vote_success_threshold / 100.0))
                        .ceil() as i64;

                    if poll.no_count == 0 && (poll.yes_count + poll.maybe_count) >= required_votes {
                        InvitePollOutcome::Allow
                    } else {
                        InvitePollOutcome::Deny
                    }
                };
                debug!("expired poll outcome: {:?}", outcome);

                if outcome == InvitePollOutcome::Allow {
                    let invite = settings
                        .invite_channel_id
                        .create_invite(http, |invite| invite.unique(true).max_uses(1))
                        .await?;

                    let pm = poll.invite_poll.user_id.create_dm_channel(http).await?;

                    pm.send_message(http, |msg| msg.content(invite.url()))
                        .await?;
                }

                poll.invite_poll.close(outcome).await?;

                match (&poll.invite_poll.channel_id, &poll.invite_poll.message_id) {
                    (Some(channel_id), Some(message_id)) => {
                        let render = poll.create_renderer(self.ctx.clone()).await?;
                        channel_id
                            .edit_message(http, message_id, |edit| {
                                render(&mut edit.into());
                                edit})
                            .await?;
                    }
                    _ => error!("could not update poll {} because either the channel_id or message_id are missing", poll.invite_poll.id),
                }
            }
        }
    }
}
