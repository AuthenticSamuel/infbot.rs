use std::{
    env,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use poise::serenity_prelude as serenity;

use crate::{Context, Data, Error, analytics, commands, events};

pub async fn init() -> poise::Framework<Data, Error> {
    let options = create_options().await;

    return poise::Framework::builder()
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
}

async fn create_options() -> poise::FrameworkOptions<Data, Error> {
    return poise::FrameworkOptions {
        commands: commands::all(),
        prefix_options: poise::PrefixFrameworkOptions {
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(3600),
            ))),
            ..Default::default()
        },
        on_error: |error| Box::pin(on_error(error)),
        pre_command: |ctx| Box::pin(pre_command(ctx)),
        event_handler: |ctx, event, framework, data| {
            Box::pin(events::handler(ctx, event, framework, data))
        },
        ..Default::default()
    };
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => {
            panic!("Failed to start bot: {:?}", error)
        }
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error);
            if let Some(client) = &ctx.data().posthog_client {
                analytics::posthog::capture_event_with_props(
                    client,
                    "command_failed",
                    &ctx.guild_id().map(|g| g.get()).unwrap_or(0).to_string(),
                    vec![
                        (
                            "command_name",
                            serde_json::json!(ctx.command().qualified_name),
                        ),
                        ("error", serde_json::json!(format!("{:?}", error))),
                    ],
                )
                .await;
            }
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e);
            }
        }
    }
}

pub async fn pre_command(ctx: Context<'_>) {
    if let Some(client) = &ctx.data().posthog_client {
        analytics::posthog::capture_event_with_props(
            client,
            "command_used",
            &ctx.guild_id().map(|g| g.get()).unwrap_or(0).to_string(),
            vec![
                (
                    "command_name",
                    serde_json::json!(ctx.command().qualified_name),
                ),
                ("channel_id", serde_json::json!(ctx.channel_id())),
            ],
        )
        .await;
    }
    println!("Executing command {}...", ctx.command().qualified_name);
}
