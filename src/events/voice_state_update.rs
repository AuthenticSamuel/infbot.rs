use crate::database::{auto_voice_channels, auto_voice_channels_installations};
use crate::{Data, analytics};
use poise::serenity_prelude as serenity;
use serenity::builder::CreateChannel;
use serenity::model::channel::ChannelType;

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
        if auto_voice_channels_installations::exists(pool, &new_channel_id).await {
            create_auto_voice_channel(ctx, data, new_voice_state).await;
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
    let channel_id = match new_voice_state.channel_id {
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

    let channel = match channel_id.to_channel(&ctx).await {
        Ok(c) => c.guild(),
        Err(err) => {
            eprintln!("Could not fetch category: {err}");
            return;
        }
    };

    let Some(channel) = channel else {
        return;
    };

    let category = match channel.parent_id {
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

    let _ = match member.move_to_voice_channel(ctx, &created_channel).await {
        Ok(_) => {
            auto_voice_channels::create(
                &data.db,
                &channel_id,
                &created_channel.id,
                &guild_id,
                &member.user.id,
            )
            .await;
        }
        Err(err) => {
            eprintln!("Failed to move member to auto voice channel: {err}");
        }
    };

    if let Some(client) = &data.posthog_client {
        analytics::posthog::capture_event_with_props(
            client,
            "auto_voice_channel_created",
            &guild_id.to_string(),
            vec![("installation_channel_id", serde_json::json!(channel_id))],
        )
        .await;
    }
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

    if !auto_voice_channels::exists(&data.db, &channel_id).await {
        return;
    }

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

    if members.len() > 0 {
        return;
    }

    match channel.delete(ctx).await {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Discord error: {err}");
        }
    };
    auto_voice_channels::delete(&data.db, &channel_id).await;

    if let Some(client) = &data.posthog_client {
        analytics::posthog::capture_event(
            client,
            "auto_voice_channel_deleted",
            &channel.guild_id.to_string(),
        )
        .await;
    }
}
