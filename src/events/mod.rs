use crate::{Data, Error};
use poise::serenity_prelude as serenity;

mod channel_delete;
mod ready;
mod voice_state_update;

pub async fn handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => ready::execute(data_about_bot),
        serenity::FullEvent::ChannelDelete { channel, .. } => {
            channel_delete::execute(data, channel).await
        }
        serenity::FullEvent::VoiceStateUpdate { old, new } => {
            voice_state_update::execute(ctx, data, old, new).await
        }
        _ => {}
    }

    return Ok(());
}
