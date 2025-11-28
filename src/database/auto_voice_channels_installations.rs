use poise::serenity_prelude as serenity;

use crate::types::AutoVoiceChannelsInstallation;

pub async fn create(
    db: &sqlx::SqlitePool,
    channel_id: &serenity::ChannelId,
    category_id: &serenity::ChannelId,
    guild_id: &serenity::GuildId,
    author_id: &serenity::UserId,
) {
    let channel_id = channel_id.get() as i64;
    let category_id = category_id.get() as i64;
    let guild_id = guild_id.get() as i64;
    let author_id = author_id.get() as i64;

    return match sqlx::query!(
        "INSERT OR IGNORE INTO auto_voice_channels_installations (channel_id, category_id, guild_id, created_by) VALUES (?,?,?,?)",
        channel_id,
        category_id,
        guild_id,
        author_id,
    ).execute(db).await {
        Ok(_) => {},
        Err(err) => {
            eprintln!("DB error: {err}");
            return;
        }
    };
}

pub async fn delete_by_channel_id(db: &sqlx::SqlitePool, channel_id: &serenity::ChannelId) {
    let channel_id = channel_id.get() as i64;

    return match sqlx::query!(
        "DELETE FROM auto_voice_channels_installations WHERE channel_id = ?",
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
        "SELECT EXISTS(SELECT 1 FROM auto_voice_channels_installations WHERE channel_id = ?)",
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

pub async fn select(
    db: &sqlx::SqlitePool,
    channel_id: &serenity::ChannelId,
) -> Option<AutoVoiceChannelsInstallation> {
    let channel_id = channel_id.get() as i64;

    let installation = sqlx::query_as!(
        AutoVoiceChannelsInstallation,
        r#"
        SELECT channel_id, category_id, guild_id, created_at, created_by
        FROM auto_voice_channels_installations
        WHERE channel_id = ?
        "#,
        channel_id,
    )
    .fetch_optional(db)
    .await
    .ok()?;

    return installation;
}

pub async fn select_all(db: &sqlx::SqlitePool) -> Vec<AutoVoiceChannelsInstallation> {
    let installations = sqlx::query_as!(
        AutoVoiceChannelsInstallation,
        r#"
        SELECT channel_id, category_id, guild_id, created_at, created_by
        FROM auto_voice_channels_installations
        "#
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();

    return installations;
}

pub async fn select_by_guild_id(
    db: &sqlx::SqlitePool,
    guild_id: &serenity::GuildId,
) -> Vec<AutoVoiceChannelsInstallation> {
    let guild_id = guild_id.get() as i64;

    let installations = sqlx::query_as!(
        AutoVoiceChannelsInstallation,
        r#"
        SELECT channel_id, category_id, guild_id, created_at, created_by
        FROM auto_voice_channels_installations
        WHERE guild_id = ?
        "#,
        guild_id,
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();

    return installations;
}
