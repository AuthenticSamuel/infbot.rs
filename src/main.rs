use dotenvy::dotenv;
use poise::serenity_prelude as serenity;
use posthog_rs;
use std::{
    env,
    error::Error as StdError,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

mod analytics;
mod commands;
mod database;
mod events;
mod framework_events;
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

    let options = poise::FrameworkOptions {
        commands: commands::all(),
        prefix_options: poise::PrefixFrameworkOptions {
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(3600),
            ))),
            ..Default::default()
        },
        on_error: |error| {
            return Box::pin(framework_events::error::handler(error));
        },
        pre_command: |ctx| {
            return Box::pin(framework_events::pre_command::handler(ctx));
        },
        event_handler: |ctx, event, framework, data| {
            return Box::pin(events::handler(ctx, event, framework, data));
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            return Box::pin(async move {
                let discord_register_guild = env::var("DISCORD_REGISTER_GUILD");

                if let Ok(id) = discord_register_guild {
                    let register_gid = id
                        .parse::<u64>()
                        .expect("DISCORD_REGISTER_GUILD must be a valid u64");
                    let gid = serenity::model::id::GuildId::new(register_gid);
                    poise::builtins::register_in_guild(ctx, &framework.options().commands, gid)
                        .await?;
                } else {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                }

                let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
                let db = sqlx::SqlitePool::connect(&database_url).await?;

                sqlx::query!("PRAGMA foreign_keys = ON;")
                    .execute(&db)
                    .await?;

                let started_at_unix = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;

                let posthog_client = analytics::posthog::init_client().await;

                return Ok(Data {
                    db,
                    posthog_client,
                    started_at_unix,
                    started_instant: Instant::now(),
                });
            });
        })
        .options(options)
        .build();

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let discord_bot_token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");

    let mut client = serenity::ClientBuilder::new(discord_bot_token, intents)
        .framework(framework)
        .await?;

    client.start().await?;

    return Ok(());
}
