use crate::database::{auto_voice_channels, auto_voice_channels_installations};
use crate::{ApplicationContext, Context, Error, analytics};
use futures::stream::{self, StreamExt};
use poise::{Modal, serenity_prelude as serenity};
use serenity::builder::CreateChannel;
use serenity::model::channel::ChannelType;
use tokio::join;

#[derive(Debug, Modal)]
#[name = "Auto Voice Channels configuration"]
struct InstallModal {
    #[name = "Category Name"]
    #[placeholder = "Default: INFBOT Voice Channels"]
    category_name: Option<String>,
    #[name = "Channel Name"]
    #[placeholder = "Default: ➕ New Channel"]
    channel_name: Option<String>,
}

pub async fn install(ctx: ApplicationContext<'_>) -> Result<(), Error> {
    let pool = &ctx.data().db;

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

    auto_voice_channels_installations::create(
        pool,
        &channel.id,
        &category.id,
        &guild_id,
        &ctx.author().id,
    )
    .await;

    {
        let mut cache = ctx.data().installation_channel_ids.write().await;
        cache.insert(channel.id.get());
    }

    if let Some(client) = &ctx.data().posthog_client {
        analytics::posthog::capture_event_with_props(
            client,
            "auto_voice_channels_module_installed",
            &guild_id.to_string(),
            vec![("channel_id", serde_json::json!(&channel.id))],
        )
        .await;
    }

    return Ok(());
}

pub async fn uninstall(ctx: Context<'_>, installation_channel_id: String) -> Result<(), Error> {
    let pool = &ctx.data().db;

    ctx.defer().await?;

    let progress_message = ctx
        .send(
            poise::CreateReply::default().content("Starting Auto Voice Channels uninstallation..."),
        )
        .await?;

    let installation_channel_id_u64: u64 = match installation_channel_id.parse() {
        Ok(id) => id,
        Err(_) => {
            progress_message
                .edit(
                    ctx,
                    poise::CreateReply::default().content("Invalid installation selected."),
                )
                .await?;
            return Ok(());
        }
    };

    let installation_channel_id = serenity::ChannelId::new(installation_channel_id_u64);
    let http = ctx.serenity_context();

    progress_message
        .edit(
            ctx,
            poise::CreateReply::default()
                .content("Loading installation information from the database..."),
        )
        .await?;

    let installation =
        match auto_voice_channels_installations::select(pool, &installation_channel_id).await {
            Some(installation) => installation,
            None => {
                progress_message
                    .edit(
                        ctx,
                        poise::CreateReply::default()
                            .content("That Auto Voice Channels installation no longer exists."),
                    )
                    .await?;
                return Ok(());
            }
        };

    progress_message
        .edit(
            ctx,
            poise::CreateReply::default().content("Deleting related voice channels..."),
        )
        .await?;

    let voice_channels =
        auto_voice_channels::select_by_installation_channel_id(pool, &installation_channel_id)
            .await;

    let delete_voice_channels_future = async {
        let concurrency = 10;

        let voice_channel_ids: Vec<u64> = voice_channels
            .iter()
            .map(|voice_channel| voice_channel.channel_id as u64)
            .collect();

        stream::iter(voice_channel_ids)
            .map(|voice_channel_id| {
                let http = http.clone();
                let voice_channel_id = serenity::ChannelId::new(voice_channel_id);

                async move {
                    if let Ok(voice_channel) = voice_channel_id.to_channel(&http).await {
                        if let Some(guild_channel) = voice_channel.guild() {
                            if let Err(err) = guild_channel.delete(&http).await {
                                eprintln!(
                                    "Failed to delete related voice channel {}: {err}",
                                    voice_channel_id
                                );
                            }
                        }
                    }
                }
            })
            .buffer_unordered(concurrency)
            .collect::<Vec<()>>()
            .await;
    };

    let delete_db_future = async {
        join!(
            auto_voice_channels_installations::delete_by_channel_id(pool, &installation_channel_id),
            auto_voice_channels::delete_by_installation_channel_id(pool, &installation_channel_id),
        );
    };

    join!(delete_voice_channels_future, delete_db_future);

    {
        let mut cache = ctx.data().installation_channel_ids.write().await;
        cache.remove(&installation_channel_id.get());
    }

    progress_message
        .edit(
            ctx,
            poise::CreateReply::default()
                .content("Deleting installation creation channel and category..."),
        )
        .await?;

    if let Ok(installation_channel) = installation_channel_id.to_channel(http).await {
        if let Some(installation_guild_channel) = installation_channel.guild() {
            if let Err(err) = installation_guild_channel.delete(http).await {
                eprintln!("Failed to delete installation creation channel: {err}");
            }
        }
    }

    let installation_category_id = serenity::ChannelId::new(installation.category_id as u64);

    if let Ok(installation_category) = installation_category_id.to_channel(http).await {
        if let Some(installation_guild_category) = installation_category.guild() {
            if let Err(err) = installation_guild_category.delete(http).await {
                eprintln!("Failed to delete installation category: {err}");
            }
        }
    }

    progress_message
        .edit(
            ctx,
            poise::CreateReply::default()
                .content("Uninstalled the Auto Voice Channels installation."),
        )
        .await?;

    if let Some(client) = &ctx.data().posthog_client {
        let guild_id = match ctx.guild_id() {
            Some(id) => id,
            None => return Ok(()),
        };

        analytics::posthog::capture_event_with_props(
            client,
            "auto_voice_channels_module_uninstalled",
            &guild_id.to_string(),
            vec![("channel_id", serde_json::json!(&installation_channel_id))],
        )
        .await;
    }

    return Ok(());
}
