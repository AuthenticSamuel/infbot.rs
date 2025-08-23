use crate::{Context, analytics};

pub async fn handler(ctx: Context<'_>) {
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
