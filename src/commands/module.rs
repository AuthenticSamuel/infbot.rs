use crate::database::auto_voice_channels_installations;
use crate::modules::auto_voice_channels;
use crate::types::AutoVoiceChannelsInstallation;
use crate::{ApplicationContext, Context, Error};
use poise::serenity_prelude as serenity;

struct Module {
    name: &'static str,
    value: &'static str,
}

async fn autocomplete_module<'a>(
    _ctx: Context<'_>,
    partial: &'a str,
) -> Vec<serenity::AutocompleteChoice> {
    let partial = partial.to_lowercase();

    let modules = vec![
        Module {
            name: "Audit",
            value: "audit",
        },
        Module {
            name: "Auto Voice Channels",
            value: "auto-voice-channels",
        },
    ];

    return modules
        .into_iter()
        .filter(|m| m.name.to_lowercase().contains(&partial))
        .map(|m| serenity::AutocompleteChoice::new(m.name, m.value))
        .take(25)
        .collect();
}

#[poise::command(
    slash_command,
    subcommands("install", "uninstall"),
    guild_only,
    required_permissions = "MANAGE_GUILD",
    required_bot_permissions = "MANAGE_CHANNELS",
    hide_in_help
)]
pub async fn module(_ctx: Context<'_>) -> Result<(), Error> {
    return Ok(());
}

#[poise::command(slash_command)]
pub async fn install(
    ctx: ApplicationContext<'_>,
    #[description = "Module to install"]
    #[autocomplete = "autocomplete_module"]
    module: String,
) -> Result<(), Error> {
    let module_install = match module.as_str() {
        "auto-voice-channels" => auto_voice_channels::install,
        unknown_module => {
            let reply = poise::CreateReply::default()
                .content(format!("The `{}` module does not exist.", unknown_module))
                .ephemeral(true);
            ctx.send(reply).await?;
            return Ok(());
        }
    };
    return module_install(ctx).await;
}

#[poise::command(
    slash_command,
    subcommands("uninstall_auto_voice_channels", "uninstall_audit"),
    guild_only,
    required_permissions = "MANAGE_GUILD",
    required_bot_permissions = "MANAGE_CHANNELS",
    hide_in_help
)]
pub async fn uninstall(_ctx: Context<'_>) -> Result<(), Error> {
    return Ok(());
}

#[poise::command(slash_command, rename = "auto-voice-channels")]
pub async fn uninstall_auto_voice_channels(
    ctx: Context<'_>,
    #[description = "Select an installation to uninstall"]
    #[autocomplete = "autocomplete_voice_channels_installations"]
    installation: String,
) -> Result<(), Error> {
    return auto_voice_channels::uninstall(ctx, installation).await;
}

#[poise::command(slash_command)]
pub async fn uninstall_audit(_ctx: Context<'_>) -> Result<(), Error> {
    return Ok(());
}

pub async fn autocomplete_voice_channels_installations(
    ctx: Context<'_>,
    partial: &str,
) -> Vec<serenity::AutocompleteChoice> {
    let partial = partial.to_lowercase();

    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => return vec![],
    };

    let installations =
        auto_voice_channels_installations::select_by_guild_id(&ctx.data().db, &guild_id).await;

    let mut choices = Vec::new();

    for installation in installations {
        let Some((choice_name, choice_value)) =
            resolve_installation_label(&ctx, &installation).await
        else {
            continue;
        };

        let choice_name_lowercase = choice_name.to_lowercase();

        if !choice_name_lowercase.contains(&partial) && !choice_value.contains(&partial) {
            continue;
        }

        let choice = serenity::AutocompleteChoice::new(choice_name, choice_value);

        choices.push(choice);
    }

    choices.truncate(25);
    return choices;
}

async fn resolve_installation_label(
    ctx: &Context<'_>,
    installation: &AutoVoiceChannelsInstallation,
) -> Option<(String, String)> {
    let channel_id = serenity::ChannelId::new(installation.channel_id as u64);
    let category_id = serenity::ChannelId::new(installation.category_id as u64);

    let channel = channel_id.to_channel(ctx.serenity_context()).await.ok()?;
    let category = category_id.to_channel(ctx.serenity_context()).await.ok()?;

    let guild_channel = channel.guild()?;
    let guild_category = category.guild()?;

    let guild_channel_name = guild_channel.name.clone();
    let guild_category_name = guild_category.name.clone();

    let choice_name = format!("{} ({})", guild_channel_name, guild_category_name);
    let choice_value = installation.channel_id.to_string();

    return Some((choice_name, choice_value));
}
