use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct AutoVoiceChannelsInstallation {
    pub channel_id: i64,
    pub category_id: i64,
    pub guild_id: i64,
    pub created_at: String,
    pub created_by: i64,
}

#[derive(Debug, FromRow)]
pub struct AutoVoiceChannel {
    pub channel_id: i64,
    pub guild_id: i64,
    pub installation_channel_id: i64,
    pub created_at: String,
    pub created_by: i64,
}
