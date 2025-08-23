use crate::modules::auto_voice_channels;
use crate::{ApplicationContext, Context, Error};
use futures::{Stream, StreamExt};

async fn autocomplete_module<'a>(
    _ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    return futures::stream::iter(&["auto-voice-channels"])
        .filter(move |name| futures::future::ready(name.starts_with(partial)))
        .map(|name| name.to_string());
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

#[poise::command(slash_command)]
pub async fn uninstall(
    ctx: Context<'_>,
    #[description = "Module to uninstall"]
    #[autocomplete = "autocomplete_module"]
    module: String,
) -> Result<(), Error> {
    let module_uninstall = match module.as_str() {
        "auto-voice-channels" => auto_voice_channels::uninstall,
        unknown_module => {
            let reply = poise::CreateReply::default()
                .content(format!("The `{}` module does not exist.", unknown_module))
                .ephemeral(true);
            ctx.send(reply).await?;
            return Ok(());
        }
    };
    return module_uninstall(ctx).await;
}
