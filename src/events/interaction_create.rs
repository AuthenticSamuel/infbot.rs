use crate::{Data, utils};
use poise::serenity_prelude::{
    self as serenity, ComponentInteraction, ComponentInteractionDataKind, Context, CreateActionRow,
    CreateInteractionResponse, CreateInteractionResponseMessage, CreateSelectMenu,
    CreateSelectMenuKind, CreateSelectMenuOption, EditChannel, Interaction, ModalInteraction,
    PremiumTier,
};

pub async fn execute(ctx: &Context, data: &Data, interaction: &Interaction) {
    match interaction {
        serenity::Interaction::Component(component) => {
            handle_component(ctx, data, component).await;
        }
        serenity::Interaction::Modal(modal) => {
            handle_modal(ctx, data, modal).await;
        }
        _ => {}
    }
}

async fn handle_component(ctx: &Context, data: &Data, component: &ComponentInteraction) {
    match component.data.custom_id.as_str() {
        "avc_bitrate_prompt" => {
            handle_bitrate_prompt(ctx, component).await;
        }
        "avc_bitrate_select" => {
            handle_bitrate_select(ctx, component).await;
        }
        // "avc_userlimit_prompt" => {
        //     handle_userlimit_prompt(ctx, component).await;
        // }
        // "avc_userlimit_select" => {
        //     handle_userlimit_select(ctx, component).await;
        // }
        _ => {}
    }
}

async fn handle_bitrate_prompt(ctx: &Context, component: &ComponentInteraction) {
    let guild_id = match component.guild_id {
        Some(id) => id,
        None => return,
    };

    let guild_premium_tier: PremiumTier = async move {
        if let Some(guild) = ctx.cache.guild(guild_id) {
            return guild.premium_tier;
        }

        match guild_id.to_partial_guild(ctx).await {
            Ok(partial_guild) => partial_guild.premium_tier,
            Err(err) => {
                eprintln!("Failed to fetch guild for bitrate prompt: {err}");
                return PremiumTier::Tier0;
            }
        }
    }
    .await;

    let bitrates = utils::guild::allowed_bitrates_for_premier_tier(guild_premium_tier);

    let options = bitrates
        .iter()
        .map(|bitrate| {
            let label = format!("{} kbps", bitrate / 1000);
            let value = bitrate.to_string();
            let option = CreateSelectMenuOption::new(label, value);
            return option;
        })
        .collect();

    let mut menu = CreateSelectMenu::new(
        "avc_bitrate_select",
        CreateSelectMenuKind::String { options },
    );
    menu = menu.placeholder("Select a new bitrate");

    let row = CreateActionRow::SelectMenu(menu.clone());
    let message = CreateInteractionResponseMessage::new()
        .ephemeral(true)
        .content("Choose a new bitrate for this channel:")
        .components(vec![row]);
    let builder = CreateInteractionResponse::Message(message);

    if let Err(err) = component.create_response(ctx, builder).await {
        eprintln!("Failed to send bitrate select menu: {err}");
    }
}

async fn handle_bitrate_select(ctx: &Context, component: &ComponentInteraction) {
    let values = match &component.data.kind {
        ComponentInteractionDataKind::StringSelect { values } => values,
        _ => return,
    };

    let Some(selected) = values.first() else {
        return;
    };

    let new_bitrate: u32 = match selected.parse() {
        Ok(value) => value,
        Err(_) => return,
    };

    let channel_id = component.message.channel_id;

    let channel = match channel_id.to_channel(ctx).await {
        Ok(channel) => channel,
        Err(_) => return,
    };

    let Some(mut guild_channel) = channel.guild() else {
        return;
    };

    let edit_builder = EditChannel::new().bitrate(new_bitrate);

    if let Err(err) = guild_channel.edit(ctx, edit_builder).await {
        eprintln!("Failed to change bitrate: {err}");
    }

    let response_message = CreateInteractionResponseMessage::new()
        .content(format!("Bitrate updated to {} kbps.", new_bitrate / 1000))
        .components(vec![]);
    let response_builder = CreateInteractionResponse::Message(response_message);

    if let Err(err) = component.create_response(ctx, response_builder).await {
        eprintln!("Failed to respond to bitrate select: {err}");
    }
}

async fn handle_modal(ctx: &Context, data: &Data, modal: &ModalInteraction) {
    match modal.data.custom_id.as_str() {
        // "avc_userlimit_modal" => {
        //     handle_userlimit_modal(ctx, modal).await;
        // }
        _ => {}
    }
}
