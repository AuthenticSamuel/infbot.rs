use poise::serenity_prelude as serenity;
use serenity::ActivityData;
use std::time::Duration;
use tokio::time::sleep;

pub async fn execute(ctx: &serenity::Context, ready: &serenity::Ready) {
    println!("Logged in as {}", ready.user.name);

    start_status_loop(ctx.clone(), ready).await;
}

async fn start_status_loop(ctx: serenity::Context, ready: &serenity::Ready) {
    let guild_count = ready.guilds.len();
    tokio::spawn(async move {
        let statuses = vec![
            ActivityData::custom(format!("Managing {} servers", guild_count)),
            ActivityData::custom("Watching out for / commands"),
            ActivityData::custom("Now with more rust 🦀"),
        ];

        let mut i = 0;
        loop {
            let activity = statuses[i % statuses.len()].clone();
            ctx.set_activity(Some(activity));

            i += 1;

            sleep(Duration::from_secs(15)).await;
        }
    });
}
