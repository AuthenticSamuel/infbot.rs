use poise::serenity_prelude as serenity;

pub async fn create(
    db: &sqlx::SqlitePool,
    installation_channel_id: &serenity::ChannelId,
    channel_id: &serenity::ChannelId,
    guild_id: &serenity::GuildId,
    user_id: &serenity::UserId,
) {
    let installation_channel_id = installation_channel_id.get() as i64;
    let channel_id = channel_id.get() as i64;
    let guild_id = guild_id.get() as i64;
    let user_id = user_id.get() as i64;

    return match sqlx::query!(
        "INSERT OR IGNORE INTO auto_voice_channels (installation_channel_id, channel_id, guild_id, created_by) VALUES (?,?,?,?)",
        installation_channel_id,
        channel_id,
        guild_id,
        user_id,
    ).execute(db).await {
        Ok(_) => {},
        Err(err) => {
            eprintln!("DB error: {err}");
            return;
        }
    };
}

pub async fn delete(db: &sqlx::SqlitePool, channel_id: &serenity::ChannelId) {
    let channel_id = channel_id.get() as i64;

    return match sqlx::query!(
        "DELETE FROM auto_voice_channels WHERE channel_id = ?",
        channel_id
    )
    .execute(db)
    .await
    {
        Ok(_) => {}
        Err(err) => {
            eprintln!("DB error: {err}");
            return;
        }
    };
}

pub async fn exists(db: &sqlx::SqlitePool, channel_id: &serenity::ChannelId) -> bool {
    let channel_id = channel_id.get() as i64;

    return match sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM auto_voice_channels WHERE channel_id = ?)",
        channel_id
    )
    .fetch_one(db)
    .await
    {
        Ok(val) => val != 0,
        Err(err) => {
            eprintln!("DB error: {err}");
            return false;
        }
    };
}
