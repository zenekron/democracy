#[macro_use]
extern crate log;

use handler::Handler;
use sea_orm::{ConnectOptions, Database};
use serenity::{prelude::GatewayIntents, Client};
use settings::Settings;

mod command;
mod entity;
mod error;
mod handler;
mod settings;

#[tokio::main]
async fn main() -> Result<(), crate::error::Error> {
    tracing_subscriber::fmt::init();

    let config = Settings::try_load()?;

    let database = {
        let mut opts = ConnectOptions::new(config.database.url);
        if let Some(schema) = config.database.schema {
            opts.set_schema_search_path(schema);
        }

        Database::connect(opts).await?
    };

    let mut client = Client::builder(config.discord.token, GatewayIntents::empty())
        .event_handler(Handler { database })
        .await?;

    client.start().await?;

    Ok(())
}
