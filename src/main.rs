#[macro_use]
extern crate log;

use handler::Handler;
use serenity::{prelude::GatewayIntents, Client};
use settings::Settings;
use sqlx::postgres::PgPoolOptions;

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

    let mut client = Client::builder(config.discord.token, GatewayIntents::empty())
        .event_handler(Handler { pool })
        .await?;

    client.start().await?;

    Ok(())
}
