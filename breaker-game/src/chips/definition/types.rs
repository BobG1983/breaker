//! Chip definition types — content types for chip definitions and templates.

use bevy::prelude::*;
use serde::Deserialize;

/// How rare a chip is — controls appearance weight in the selection pool.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rarity {
    /// Frequently appearing chips.
    Common,
    /// Moderately rare chips.
    Uncommon,
    /// Hard to find chips.
    Rare,
    /// Extremely rare, run-defining chips.
    Legendary,
    /// Evolution-tier chips — produced by combining maxed ingredient chips.
    Evolution,
}

/// A single ingredient required for a chip evolution recipe.
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct EvolutionIngredient {
    /// Name of the chip required.
    pub chip_name: String,
    /// Minimum stacks the player must hold.
    pub stacks_required: u32,
}

/// A rarity slot within a [`ChipTemplate`], defining the prefix and effects
/// for one rarity tier of a template chip.
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct RaritySlot {
    /// Display prefix prepended to the template name (e.g., "Basic", "Keen").
    pub prefix: String,
    /// The effects applied when this rarity variant is selected.
    pub effects: Vec<crate::effect::definition::RootEffect>,
}

/// A chip template loaded from RON (`.chip.ron`).
///
/// Each template defines up to four rarity variants. At load time,
/// [`expand_template`] converts each non-`None` slot into a [`ChipDefinition`].
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct ChipTemplate {
    /// Base name shared by all rarity variants.
    pub name: String,
    /// Maximum total chips from this template the player may hold.
    pub max_taken: u32,
    /// Common-rarity variant, if any.
    #[serde(default)]
    pub common: Option<RaritySlot>,
    /// Uncommon-rarity variant, if any.
    #[serde(default)]
    pub uncommon: Option<RaritySlot>,
    /// Rare-rarity variant, if any.
    #[serde(default)]
    pub rare: Option<RaritySlot>,
    /// Legendary-rarity variant, if any.
    #[serde(default)]
    pub legendary: Option<RaritySlot>,
}

/// Expand a [`ChipTemplate`] into one [`ChipDefinition`] per non-`None` rarity slot.
///
/// Each slot's prefix is prepended to the template name. An empty or
/// whitespace-only prefix causes the expanded name to equal the template name
/// (no prefix prepended).
#[must_use]
pub(crate) fn expand_template(template: &ChipTemplate) -> Vec<ChipDefinition> {
    let slots: [(Rarity, &Option<RaritySlot>); 4] = [
        (Rarity::Common, &template.common),
        (Rarity::Uncommon, &template.uncommon),
        (Rarity::Rare, &template.rare),
        (Rarity::Legendary, &template.legendary),
    ];

    slots
        .iter()
        .filter_map(|(rarity, slot_opt)| {
            let slot = slot_opt.as_ref()?;
            let name = if slot.prefix.trim().is_empty() {
                template.name.clone()
            } else {
                format!("{} {}", slot.prefix, template.name)
            };
            Some(ChipDefinition {
                name,
                description: String::new(),
                rarity: *rarity,
                max_stacks: template.max_taken,
                effects: slot.effects.clone(),
                ingredients: None,
                template_name: Some(template.name.clone()),
            })
        })
        .collect()
}

/// A single chip definition loaded from RON.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct ChipDefinition {
    /// Display name shown on the chip card.
    pub name: String,
    /// Flavor text shown below the name.
    pub description: String,
    /// How rare this chip is.
    pub rarity: Rarity,
    /// Maximum number of times this chip can be stacked.
    pub max_stacks: u32,
    /// The effects applied when this chip is selected.
    pub effects: Vec<crate::effect::definition::RootEffect>,
    /// Evolution ingredients. `None` for non-evolution chips.
    #[serde(default)]
    pub ingredients: Option<Vec<EvolutionIngredient>>,
    /// Template this chip was expanded from, if any.
    #[serde(default)]
    pub template_name: Option<String>,
}

#[cfg(test)]
impl ChipDefinition {
    /// Build a test chip wrapping the effect in `RootEffect::On` targeting `Bolt`.
    pub(crate) fn test(
        name: &str,
        effect: crate::effect::definition::EffectNode,
        max_stacks: u32,
    ) -> Self {
        Self::test_on(
            name,
            crate::effect::definition::Target::Bolt,
            effect,
            max_stacks,
        )
    }

    /// Build a simple test chip with a triggered chain and `max_stacks` = 1.
    pub(crate) fn test_simple(name: &str) -> Self {
        use crate::effect::definition::{Effect, EffectNode, Trigger};
        Self::test(
            name,
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
            },
            1,
        )
    }

    /// Build a test chip with explicit target control.
    pub(crate) fn test_on(
        name: &str,
        target: crate::effect::definition::Target,
        effect: crate::effect::definition::EffectNode,
        max_stacks: u32,
    ) -> Self {
        use crate::effect::definition::RootEffect;
        Self {
            name: name.to_owned(),
            description: format!("{name} description"),
            rarity: Rarity::Common,
            max_stacks,
            effects: vec![RootEffect::On {
                target,
                then: vec![effect],
            }],
            ingredients: None,
            template_name: None,
        }
    }
}
