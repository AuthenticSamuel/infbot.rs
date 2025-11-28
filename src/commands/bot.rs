use crate::{Context, Data, Error};
use poise::serenity_prelude::{self as serenity, Cache};

/// Get useful INFBOT information
#[poise::command(slash_command)]
pub async fn bot(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let cache = ctx.cache();

    let relative = get_relative_online_string(data);
    let uptime = get_uptime_string(data);
    let user_count_string = get_user_count_string(cache);
    let guild_count_string = get_guild_count_string(cache);
    let channel_count_string = get_channel_count_string(cache);

    let embed = serenity::CreateEmbed::new()
        .title("INFBOT Services")
        .fields(vec![
            ("Got online", relative.as_str(), true),
            ("Uptime", uptime.as_str(), true),
            ("", "", true),
            ("Users", user_count_string.as_str(), true),
            ("Servers", guild_count_string.as_str(), true),
            ("Channels", channel_count_string.as_str(), true),
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

fn get_relative_online_string(data: &Data) -> String {
    return format!("<t:{}:R>", data.started_at_unix);
}

fn get_uptime_string(data: &Data) -> String {
    let secs = data.started_instant.elapsed().as_secs();
    let (h, m, s) = (secs / 3600, (secs % 3600) / 60, secs % 60);
    let uptime_string = format!("{:02}h:{:02}m:{:02}s", h, m, s);

    return uptime_string;
}

fn get_user_count_string(cache: &Cache) -> String {
    let user_count = cache
        .guilds()
        .iter()
        .filter_map(|guild_id| cache.guild(*guild_id))
        .map(|guild| guild.member_count as usize)
        .sum();

    let user_word = match user_count {
        1 => "user",
        _ => "users",
    };
    let user_count_string = format!("{user_count} {user_word}");

    return user_count_string;
}

fn get_guild_count_string(cache: &Cache) -> String {
    let guild_count = cache.guild_count();
    let guild_word = match guild_count {
        1 => "server",
        _ => "servers",
    };
    let guild_count_string = format!("{guild_count} {guild_word}");

    return guild_count_string;
}

fn get_channel_count_string(cache: &Cache) -> String {
    let channel_count = cache.guild_channel_count();
    let channel_word = match channel_count {
        1 => "channel",
        _ => "channels",
    };
    let channel_count_string = format!("{channel_count} {channel_word}");

    return channel_count_string;
}
