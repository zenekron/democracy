#[macro_use]
extern crate log;

use std::env;

use handler::Handler;
use serenity::{prelude::GatewayIntents, Client};

mod command;
mod error;
mod handler;

#[tokio::main]
async fn main() -> Result<(), crate::error::Error> {
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").expect("env variable `DISCORD_TOKEN` not found");

    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler)
        .await?;

    client.start().await?;

    Ok(())
}
