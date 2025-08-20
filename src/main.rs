mod commands;

use std::{
    env::var,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use dotenv::dotenv;
use poise::serenity_prelude::{self as serenity, GuildId};

type Error = Box<dyn std::error::Error + Send + Sync>;
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
async fn main() {
    tracing_subscriber::fmt::init();

    dotenv().ok();

    let options = poise::FrameworkOptions {
        commands: vec![commands::bot(), commands::help()],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("~".into()),
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(3600),
            ))),
            additional_prefixes: vec![
                poise::Prefix::Literal("hey bot,"),
                poise::Prefix::Literal("hey bot"),
            ],
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
        command_check: Some(|ctx| {
            return Box::pin(async move {
                if ctx.author().id == 123456789 {
                    return Ok(false);
                }
                return Ok(true);
            });
        }),
        skip_checks_for_owners: false,
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

                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    GuildId::new(756866684106833980),
                )
                .await?;

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

    let token = var("DISCORD_TOKEN").expect("Missing `DISCORD_TOKEN` environment variable.");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    return client.unwrap().start().await.unwrap();
}
