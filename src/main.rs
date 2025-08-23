use dotenvy::dotenv;
use poise::serenity_prelude as serenity;
use posthog_rs;
use std::{env, error::Error as StdError, time::Instant};

mod analytics;
mod commands;
mod database;
mod events;
mod framework;
mod modules;

type ApplicationContext<'a> = poise::ApplicationContext<'a, Data, Error>;
type Context<'a> = poise::Context<'a, Data, Error>;
type Error = Box<dyn StdError + Send + Sync>;

pub struct Data {
    pub db: sqlx::SqlitePool,
    pub posthog_client: Option<posthog_rs::Client>,
    pub started_at_unix: i64,
    pub started_instant: Instant,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    tracing_subscriber::fmt::init();

    dotenv().ok();

    let framework = framework::setup::init().await;

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let discord_bot_token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");

    let mut client = serenity::ClientBuilder::new(discord_bot_token, intents)
        .framework(framework)
        .await?;

    client.start().await?;

    return Ok(());
}
