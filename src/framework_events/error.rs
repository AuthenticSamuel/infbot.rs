use crate::{Data, Error, analytics};

pub async fn handler(error: poise::FrameworkError<'_, Data, Error>) {
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
