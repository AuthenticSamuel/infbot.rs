use poise::serenity_prelude as serenity;

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
