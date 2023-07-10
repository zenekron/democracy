#[macro_use]
extern crate log;

use handler::Handler;
use serenity::{prelude::GatewayIntents, Client};
use settings::Settings;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::sync::OnceCell;

mod action;
mod background_poll_handler;
mod entities;
mod error;
mod handler;
mod settings;
mod util;

pub static POOL: OnceCell<PgPool> = OnceCell::const_new();

#[tokio::main]
async fn main() -> Result<(), crate::error::Error> {
    tracing_subscriber::fmt::init();

    let config = Settings::try_load()?;

    // connect to the database and apply migrations
    let pool = POOL
        .get_or_try_init(|| PgPoolOptions::new().connect(&config.database.url))
        .await?;
    sqlx::migrate!().run(pool).await?;

    let mut client = Client::builder(
        config.discord.token,
        GatewayIntents::default() | GatewayIntents::GUILD_MEMBERS,
    )
    .event_handler(Handler)
    .await?;

    client.start().await?;
    Ok(())
}
