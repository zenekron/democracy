#[macro_use]
extern crate log;

use handler::Handler;
use opentelemetry::{
    sdk::{trace, Resource},
    KeyValue,
};
use serenity::{prelude::GatewayIntents, Client};
use settings::Settings;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::sync::OnceCell;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

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
    // opentelemetry
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_trace_config(trace::config().with_resource(Resource::new(vec![
            KeyValue::new("service.name", env!("CARGO_BIN_NAME")),
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        ])))
        .install_batch(opentelemetry::runtime::Tokio)
        .unwrap();

    // tracing
    Registry::default()
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

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
