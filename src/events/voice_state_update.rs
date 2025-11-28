use crate::database::auto_voice_channels;
use crate::{Data, analytics};
use poise::serenity_prelude::{
    self as serenity, ChannelId, CreateActionRow, CreateButton, CreateEmbed, CreateMessage,
};
use serenity::builder::CreateChannel;
use serenity::model::channel::ChannelType;
use tokio::join;

pub async fn execute(
    ctx: &serenity::Context,
    data: &Data,
    old_voice_state: &Option<serenity::VoiceState>,
    new_voice_state: &serenity::VoiceState,
) {
    let pool = &data.db;
    let old_channel_id = old_voice_state.as_ref().and_then(|o| o.channel_id);
    let new_channel_id = new_voice_state.channel_id;

    if old_channel_id == new_channel_id {
        return;
    }

    if let Some(new_channel_id) = new_channel_id {
        let new_id_u64 = new_channel_id.get();

        let in_cache = {
            let cache = data.installation_channel_ids.read().await;
            cache.contains(&new_id_u64)
        };

        if in_cache {
            create_auto_voice_channel(ctx, data, new_voice_state).await
        }
    }

    let Some(old_voice_state) = old_voice_state else {
        return;
    };

    let Some(old_channel_id) = old_channel_id else {
        return;
    };

    if auto_voice_channels::exists(pool, &old_channel_id).await {
        delete_auto_voice_channel(ctx, data, old_voice_state).await
    }
}

async fn create_auto_voice_channel(
    ctx: &serenity::Context,
    data: &Data,
    new_voice_state: &serenity::VoiceState,
) {
    let installation_channel_id = match new_voice_state.channel_id {
        Some(id) => id,
        None => return,
    };

    let guild_id = match new_voice_state.guild_id {
        Some(id) => id,
        None => return,
    };

    let member = match &new_voice_state.member {
        Some(member) => member,
        None => return,
    };

    let guild_channel = match installation_channel_id.to_channel(&ctx).await {
        Ok(c) => c.guild(),
        Err(err) => {
            eprintln!("Could not fetch channel: {err}");
            return;
        }
    };

    let Some(guild_channel) = guild_channel else {
        return;
    };

    let category = match guild_channel.parent_id {
        Some(id) => id,
        None => return,
    };

    let channel_builder = CreateChannel::new(member.display_name())
        .kind(ChannelType::Voice)
        .category(category);

    let created_channel = match guild_id.create_channel(ctx, channel_builder).await {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Failed to create auto voice channel: {err}");
            return;
        }
    };

    if let Err(err) = member.move_to_voice_channel(ctx, &created_channel).await {
        eprintln!("Failed to move member to auto voice channel: {err}");
    };

    let db = data.db.clone();
    let guild_id_str = guild_id.to_string();
    let created_channel_id = created_channel.id;

    let control_panel_future = async move {
        create_control_panel(&ctx, created_channel_id).await;
    };

    let db_future = async move {
        auto_voice_channels::create(
            &db,
            &installation_channel_id,
            &created_channel_id,
            &guild_id,
            &member.user.id,
        )
        .await;
    };

    let analytics_future = async move {
        if let Some(client) = &data.posthog_client {
            analytics::posthog::capture_event_with_props(
                client,
                "auto_voice_channel_created",
                &guild_id_str,
                vec![(
                    "installation_channel_id",
                    serde_json::json!(installation_channel_id),
                )],
            )
            .await;
        }
    };

    join!(control_panel_future, db_future, analytics_future);
}

async fn delete_auto_voice_channel(
    ctx: &serenity::Context,
    data: &Data,
    old_voice_state: &serenity::VoiceState,
) {
    let channel_id = match old_voice_state.channel_id {
        Some(id) => id,
        None => return,
    };

    let channel = match channel_id.to_channel(ctx).await {
        Ok(c) => c.guild(),
        Err(err) => {
            eprintln!("Discord error: {err}");
            return;
        }
    };

    let Some(channel) = channel else {
        return;
    };

    let members = match channel.members(&ctx.cache) {
        Ok(m) => m,
        Err(err) => {
            eprintln!("Discord error: {err}");
            return;
        }
    };

    if !members.is_empty() {
        return;
    }

    if let Err(err) = channel.delete(ctx).await {
        eprintln!("Discord error: {err}");
    };

    let db_future =
        async move { auto_voice_channels::delete_by_channel_id(&data.db, &channel_id).await };

    let analytics_future = async move {
        if let Some(client) = &data.posthog_client {
            analytics::posthog::capture_event(
                client,
                "auto_voice_channel_deleted",
                &channel.guild_id.to_string(),
            )
            .await;
        }
    };

    join!(db_future, analytics_future);
}

async fn create_control_panel(ctx: &serenity::Context, guild_channel_id: ChannelId) {
    let guild_channel = match guild_channel_id.to_channel(ctx).await {
        Ok(c) => c.guild(),
        Err(err) => {
            eprintln!("Discord error: {err}");
            return;
        }
    };

    let Some(guild_channel) = guild_channel else {
        return;
    };

    let embed = CreateEmbed::new()
        .title("Channel Control Panel")
        .description("Manage this auto voice channel");

    let row = CreateActionRow::Buttons(vec![
        CreateButton::new("avc_lock_toggle")
            .label("Lock / Unlock")
            .style(serenity::ButtonStyle::Primary),
        CreateButton::new("avc_bitrate_prompt")
            .label("Set Bitrate")
            .style(serenity::ButtonStyle::Secondary),
        CreateButton::new("avc_userlimit_prompt")
            .label("Set User Limit")
            .style(serenity::ButtonStyle::Secondary),
    ]);

    let builder = CreateMessage::new().embed(embed).components(vec![row]);

    if let Err(err) = guild_channel.send_message(ctx, builder).await {
        eprintln!("Discord error: {err}");
    }
}
