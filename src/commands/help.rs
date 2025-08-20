use crate::{Context, Error};

/// View available INFBOT commands
#[poise::command(slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    let config = poise::builtins::HelpConfiguration {
        extra_text_at_bottom: "INFBOT Services",
        ..Default::default()
    };

    poise::builtins::help(ctx, command.as_deref(), config).await?;

    return Ok(());
}
