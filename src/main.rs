use std::{
    error::Error as StdError,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use poise::serenity_prelude as serenity;

use crate::config::Config;

mod commands;
mod config;

type Error = Box<dyn StdError + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    // bot_user: serenity::model::user::CurrentUser,
    started_at_unix: i64,
    started_instant: Instant,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    tracing_subscriber::fmt::init();

    let cfg = Config::from_env()?;

    let options = poise::FrameworkOptions {
        commands: vec![commands::bot(), commands::help()],
        prefix_options: poise::PrefixFrameworkOptions {
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(3600),
            ))),
            ..Default::default()
        },
        on_error: |error| {
            return Box::pin(on_error(error));
        },
        pre_command: |ctx| {
            return Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            });
        },
        event_handler: |_ctx, event, _framework, _data| {
            return Box::pin(async move {
                println!(
                    "Got an event in event handler: {:?}",
                    event.snake_case_name()
                );
                return Ok(());
            });
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .setup(move |ctx, ready, framework| {
            return Box::pin(async move {
                println!("Logged in as {}", ready.user.name);

                if let Some(id) = cfg.discord_register_guild {
                    let gid = serenity::model::id::GuildId::new(id);
                    poise::builtins::register_in_guild(ctx, &framework.options().commands, gid)
                        .await?;
                } else {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                }

                let started_at_unix = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;

                return Ok(Data {
                    // bot_user: ready.user.clone(),
                    started_at_unix,
                    started_instant: Instant::now(),
                });
            });
        })
        .options(options)
        .build();

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let mut client = serenity::ClientBuilder::new(cfg.discord_bot_token.clone(), intents)
        .framework(framework)
        .await?;

    client.start().await?;

    return Ok(());
}
