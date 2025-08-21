use poise::serenity_prelude as serenity;

use crate::{Context, Error};

/// Get useful server information
#[poise::command(guild_only, slash_command)]
pub async fn server(ctx: Context<'_>) -> Result<(), Error> {
    let gid = ctx.guild_id().expect("guild_only ensures a guild context");

    let name: String;
    let owner_id: serenity::model::id::UserId;
    let mut member_count: Option<u64> = None;

    if let Some(guild) = gid.to_guild_cached(&ctx.cache()) {
        name = guild.name.clone();
        owner_id = guild.owner_id;
        member_count = Some(guild.member_count as u64);
    } else {
        let pg = gid.to_partial_guild(&ctx.http()).await?;
        name = pg.name.clone();
        owner_id = pg.owner_id;
    }

    if member_count.is_none() {
        if let Ok(pg) = gid.to_partial_guild(&ctx.http()).await {
            if let Some(n) = pg.approximate_member_count {
                member_count = Some(n as u64);
            }
        }
    }

    let member_count = member_count
        .map(|n| n.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let created_ts = gid.created_at();
    let created_unix = created_ts.unix_timestamp();
    let created_at = format!("<t:{}:F>\n<t:{}:R>", created_unix, created_unix);

    let owner = format!("<@{}>", owner_id);

    let embed = serenity::CreateEmbed::new()
        .title(format!("Server: {}", name))
        .fields(vec![
            ("Members", member_count, false),
            ("Created", created_at, false),
            ("Created by", owner, false),
        ])
        .colour(serenity::Colour::new(0x818CF8))
        .timestamp(serenity::Timestamp::now());

    let reply = poise::CreateReply::default().embed(embed);

    ctx.send(reply).await?;

    return Ok(());
}
