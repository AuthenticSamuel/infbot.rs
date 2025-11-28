use poise::serenity_prelude as serenity;
use serenity::model::guild::PremiumTier;

pub fn allowed_bitrates_for_premier_tier(premium_tier: PremiumTier) -> Vec<u32> {
    let max = match premium_tier {
        PremiumTier::Tier3 => 384_000,
        PremiumTier::Tier2 => 256_000,
        PremiumTier::Tier1 => 128_000,
        _ => 96_000,
    };

    let candidates = [
        8_000, 16_000, 32_000, 64_000, 96_000, 128_000, 256_000, 384_000,
    ];

    return candidates
        .into_iter()
        .filter(|candidate| *candidate <= max)
        .collect();
}
