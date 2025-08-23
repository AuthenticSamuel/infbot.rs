use crate::{Data, analytics};
use poise::serenity_prelude as serenity;

pub async fn execute(_ctx: &serenity::Context, data: &Data, channel: &serenity::GuildChannel) {
    let pool = &data.db;

    let channel_id = channel.id.get() as i64;

    let result = sqlx::query!(
        "DELETE FROM auto_voice_channels_installations WHERE channel_id = ?",
        channel_id
    )
    .execute(pool)
    .await;

    if let Ok(result) = result {
        if result.rows_affected() > 0 {
            println!("Removed auto voice channels as installation channel was deleted");

            if let Some(client) = &data.posthog_client {
                analytics::posthog::capture_event(
                    client,
                    "auto_voice_channels_module_uninstalled",
                    &channel.guild_id.to_string(),
                )
                .await;
            }
        }
    }
}
