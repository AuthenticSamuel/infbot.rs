use crate::{Data, Error};
use poise::serenity_prelude as serenity;

mod channel_delete;
mod guild_create;
mod guild_delete;
mod ready;
mod voice_state_update;

pub async fn handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::ChannelDelete { channel, .. } => {
            channel_delete::execute(ctx, data, channel).await
        }
        serenity::FullEvent::GuildCreate { guild, is_new } => {
            guild_create::execute(ctx, data, guild, is_new).await
        }
        serenity::FullEvent::GuildDelete { incomplete, .. } => {
            guild_delete::execute(ctx, data, incomplete).await
        }
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            ready::execute(ctx, data, data_about_bot).await
        }
        serenity::FullEvent::VoiceStateUpdate { old, new } => {
            voice_state_update::execute(ctx, data, old, new).await
        }
        _ => {}
    }

    return Ok(());
}
