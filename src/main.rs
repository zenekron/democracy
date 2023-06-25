#[macro_use]
extern crate log;

use handler::Handler;
use sea_orm::Database;
use serenity::{prelude::GatewayIntents, Client};
use settings::Settings;

mod command;
mod error;
mod handler;
mod settings;

#[tokio::main]
async fn main() -> Result<(), crate::error::Error> {
    tracing_subscriber::fmt::init();

    let config = Settings::try_load()?;
    let _db = Database::connect(config.database.url).await?;

    let mut client = Client::builder(config.discord.token, GatewayIntents::empty())
        .event_handler(Handler)
        .await?;

    client.start().await?;

    Ok(())
}
