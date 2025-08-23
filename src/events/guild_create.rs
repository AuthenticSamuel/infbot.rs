use crate::{Data, analytics};
use poise::serenity_prelude as serenity;

pub async fn execute(
    _ctx: &serenity::Context,
    data: &Data,
    guild: &serenity::Guild,
    is_new: &Option<bool>,
) {
    if let Some(client) = &data.posthog_client {
        analytics::posthog::capture_event_with_props(
            client,
            "guild_joined",
            &guild.id.to_string(),
            vec![
                ("is_new", serde_json::json!(is_new)),
                ("member_count", serde_json::json!(guild.member_count)),
            ],
        )
        .await;
    }
}
