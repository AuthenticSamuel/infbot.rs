use crate::{ApplicationContext, Context, Error};
use poise::{Modal, serenity_prelude as serenity};
use serenity::builder::CreateChannel;
use serenity::model::channel::ChannelType;

#[derive(Debug, Modal)]
#[name = "Auto Voice Channels configuration"]
struct InstallModal {
    #[name = "Category Name"]
    #[placeholder = "INFBOT Voice Channels"]
    category_name: Option<String>,
    #[name = "Channel Name"]
    #[placeholder = "➕ New Channel"]
    channel_name: Option<String>,
}

pub async fn install(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let pool = &ctx.data().db;
    let author_id = ctx.author().id.get() as i64;

    let data = match InstallModal::execute(ctx).await? {
        Some(d) => d,
        None => return Ok(()),
    };

    let category_name = data
        .category_name
        .unwrap_or_else(|| String::from("INFBOT Voice Channels"));
    let channel_name = data
        .channel_name
        .unwrap_or_else(|| String::from("➕ New Channel"));

    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => return Ok(()),
    };

    let category_builder = CreateChannel::new(category_name).kind(ChannelType::Category);
    let category = guild_id
        .create_channel(ctx.serenity_context(), category_builder)
        .await?;

    let channel_builder = CreateChannel::new(channel_name)
        .kind(ChannelType::Voice)
        .category(category.id);

    let channel = guild_id
        .create_channel(ctx.serenity_context(), channel_builder)
        .await?;

    let channel_id = channel.id.get() as i64;
    let category_id = category.id.get() as i64;
    let guild_id = guild_id.get() as i64;

    sqlx::query!(
        "INSERT OR IGNORE INTO auto_voice_channels_installations (channel_id, category_id, guild_id, created_by) VALUES (?,?,?,?)",
        channel_id,
        category_id,
        guild_id,
        author_id,
    )
    .execute(pool)
    .await?;

    return Ok(());
}

pub async fn uninstall(_ctx: Context<'_>) -> Result<(), Error> {
    //TODO
    return Ok(());
}
