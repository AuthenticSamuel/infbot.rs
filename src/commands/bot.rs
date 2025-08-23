use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Get useful INFBOT information
#[poise::command(slash_command)]
pub async fn bot(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let cache = ctx.cache();

    let relative = format!("<t:{}:R>", data.started_at_unix);

    let secs = data.started_instant.elapsed().as_secs();
    let (h, m, s) = (secs / 3600, (secs % 3600) / 60, secs % 60);
    let uptime = format!("{:02}h:{:02}m:{:02}s", h, m, s);
    let uptime = uptime.as_str();

    let user_count = cache.user_count();
    let user_word = match user_count {
        1 => "user",
        _ => "users",
    };
    let user_string = format!("{user_count} {user_word}");

    let guild_count = cache.guild_count();
    let guild_word = match guild_count {
        1 => "server",
        _ => "servers",
    };
    let guild_string = format!("{guild_count} {guild_word}");

    let guild_channel_count = cache.guild_channel_count();
    let guild_channel_word = match guild_channel_count {
        1 => "user",
        _ => "users",
    };
    let guild_channel_string = format!("{guild_channel_count} {guild_channel_word}");

    let embed = serenity::CreateEmbed::new()
        .title("INFBOT Services")
        .fields(vec![
            ("Got online", relative.as_str(), true),
            ("Uptime", uptime, true),
            ("", "", true),
            ("Users", user_string.as_str(), true),
            ("Servers", guild_string.as_str(), true),
            ("Channels", guild_channel_string.as_str(), true),
            ("Version", env!("CARGO_PKG_VERSION"), true),
            ("Developer", "realZenyth", true),
            ("", "", true),
            ("Support Server", "https://discord.gg/BayN67CgAx", false),
            (
                "Source Code",
                "https://github.com/AuthenticSamuel/infbot.rs",
                false,
            ),
        ])
        .colour(serenity::Colour::new(0x818CF8));

    let reply = poise::CreateReply::default().embed(embed);

    ctx.send(reply).await?;
    return Ok(());
}
