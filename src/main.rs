#[macro_use]
extern crate log;

use std::time::Duration;

use entities::InvitePoll;
use handler::Handler;
use serenity::{prelude::GatewayIntents, Client};
use settings::Settings;
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::error::Error;

mod action;
mod entities;
mod error;
mod handler;
mod settings;
mod util;

#[tokio::main]
async fn main() -> Result<(), crate::error::Error> {
    tracing_subscriber::fmt::init();

    let config = Settings::try_load()?;

    // connect to the database and apply migrations
    let pool = PgPoolOptions::new().connect(&config.database.url).await?;
    sqlx::migrate!().run(&pool).await?;

    let mut client = Client::builder(
        config.discord.token,
        GatewayIntents::default() | GatewayIntents::GUILD_MEMBERS,
    )
    .event_handler(Handler { pool: pool.clone() })
    .await?;

    let (bres, cres) = tokio::join!(background_poll_closer(&pool), client.start());
    bres.and(cres.map_err(Into::into))
}

async fn background_poll_closer(pool: &PgPool) -> Result<(), Error> {
    let mut interval = tokio::time::interval(Duration::from_secs(60));

    loop {
        interval.tick().await;

        let polls = InvitePoll::find_pending_with_count(pool).await?;
        debug!("polls: {:?}", polls);
    }
}
