use crate::{Data, analytics};
use poise::serenity_prelude as serenity;

pub async fn execute(_ctx: &serenity::Context, data: &Data, guild: &serenity::UnavailableGuild) {
    if let Some(client) = &data.posthog_client {
        analytics::posthog::capture_event(client, "guild_left", &guild.id.to_string()).await;
    }
}
