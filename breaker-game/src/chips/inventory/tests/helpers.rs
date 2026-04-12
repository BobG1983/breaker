use ordered_float::OrderedFloat;

use crate::{
    chips::definition::{ChipDefinition, Rarity},
    effect_v3::{
        effects::{DamageBoostConfig, PiercingConfig, SizeBoostConfig},
        types::{EffectType, Tree},
    },
};

/// Helper: create a Piercing Shot definition with `max_stacks=3`, Common rarity.
pub(super) fn piercing_shot_def() -> ChipDefinition {
    ChipDefinition::test(
        "Piercing Shot",
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
        3,
    )
}

/// Helper: create a Wide Breaker definition with `max_stacks=3`, Rare rarity.
pub(super) fn wide_breaker_def() -> ChipDefinition {
    ChipDefinition {
        rarity: Rarity::Rare,
        ..ChipDefinition::test(
            "Wide Breaker",
            Tree::Fire(EffectType::SizeBoost(SizeBoostConfig {
                multiplier: OrderedFloat(20.0),
            })),
            3,
        )
    }
}

/// Helper: create a Damage Up definition with `max_stacks=2`, Common rarity.
pub(super) fn damage_up_def() -> ChipDefinition {
    ChipDefinition::test(
        "Damage Up",
        Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: OrderedFloat(0.5),
        })),
        2,
    )
}

/// Helper: create a chip definition with `max_stacks=1`, Common rarity.
pub(super) fn single_stack_def() -> ChipDefinition {
    ChipDefinition::test(
        "Single Stack",
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
        1,
    )
}

/// Helper: create a chip definition with a `template_name`.
pub(super) fn template_chip_def(
    name: &str,
    template_name: &str,
    max_stacks: u32,
) -> ChipDefinition {
    ChipDefinition {
        template_name: Some(template_name.to_owned()),
        ..ChipDefinition::test(
            name,
            Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
            max_stacks,
        )
    }
}

/// Helper: create a chip definition with no template.
pub(super) fn standalone_chip_def(name: &str, max_stacks: u32) -> ChipDefinition {
    ChipDefinition::test(
        name,
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
        max_stacks,
    )
}
