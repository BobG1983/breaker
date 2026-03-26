//! Chip definition types — `TriggerChain` variants and content types.

use bevy::prelude::*;
use serde::Deserialize;

pub use crate::effect::definition::{ImpactTarget, Target};

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

/// Recursive enum encoding all chip effect logic — trigger wrapper variants nest
/// around leaf action variants. Used for both passive effects (via `OnSelected`)
/// and triggered abilities (via bridge system evaluation).
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub enum TriggerChain {
    /// Area damage around impact point — expanding wavefront.
    Shockwave {
        /// Base radius of the shockwave effect.
        base_range: f32,
        /// Additional radius per stack beyond the first.
        range_per_level: f32,
        /// Current stack count (starts at 1, incremented at runtime).
        stacks: u32,
        /// Expansion speed in world units per second.
        speed: f32,
    },
    /// Spawns additional bolts on trigger.
    MultiBolt {
        /// Base number of extra bolts to spawn.
        base_count: u32,
        /// Additional bolts per stack beyond the first.
        count_per_level: u32,
        /// Current stack count.
        stacks: u32,
    },
    /// Temporary shield protecting the breaker.
    Shield {
        /// Base duration in seconds.
        base_duration: f32,
        /// Additional duration per stack beyond the first.
        duration_per_level: f32,
        /// Current stack count.
        stacks: u32,
    },
    /// Deducts a life from the breaker.
    LoseLife,
    /// Applies a time penalty in seconds.
    TimePenalty {
        /// Duration of the penalty in seconds.
        seconds: f32,
    },
    /// Spawns an additional bolt.
    SpawnBolt,
    /// Scales a target's speed by a multiplier, clamped within base/max bounds.
    SpeedBoost {
        /// Which entity to apply the speed change to.
        target: Target,
        /// Multiplier applied to the current velocity magnitude.
        multiplier: f32,
    },
    /// Spawns a tethered chain bolt at the anchor bolt's position.
    ChainBolt {
        /// Maximum distance the chain bolt can travel from its anchor.
        tether_distance: f32,
    },
    /// Chain lightning arcing between nearby cells.
    ChainLightning {
        /// Number of arcs from the origin cell.
        arcs: u32,
        /// Maximum arc range in world units.
        range: f32,
        /// Damage multiplier per arc (applied to base bolt damage).
        damage_mult: f32,
    },
    /// Spawns a temporary phantom breaker entity.
    SpawnPhantom {
        /// How long the phantom persists in seconds.
        duration: f32,
        /// Maximum active phantoms at once.
        max_active: u32,
    },
    /// Fires a piercing beam through cells in a line.
    PiercingBeam {
        /// Damage multiplier for the beam.
        damage_mult: f32,
        /// Width of the beam in world units.
        width: f32,
    },
    /// Creates a gravity well that attracts bolts.
    GravityWell {
        /// Attraction strength.
        strength: f32,
        /// Duration in seconds.
        duration: f32,
        /// Effect radius in world units.
        radius: f32,
        /// Maximum active wells at once.
        max: u32,
    },
    /// Temporary invulnerability after bolt loss.
    SecondWind {
        /// Duration of invulnerability in seconds.
        invuln_secs: f32,
    },
    /// Fires on a perfect bump. Inner vec allows multiple effects per trigger.
    OnPerfectBump(Vec<Self>),
    /// Fires on bolt impact with a specific surface.
    OnImpact(ImpactTarget, Vec<Self>),
    /// Fires when a cell is destroyed.
    OnCellDestroyed(Vec<Self>),
    /// Fires when a bolt is lost.
    OnBoltLost(Vec<Self>),
    /// Fires on any non-whiff bump (Early, Late, or Perfect).
    OnBump(Vec<Self>),
    /// Fires on an early bump.
    OnEarlyBump(Vec<Self>),
    /// Fires on a late bump.
    OnLateBump(Vec<Self>),
    /// Fires when a bump whiffs (misses).
    OnBumpWhiff(Vec<Self>),
    /// Passive effects: evaluated immediately on chip selection.
    OnSelected(Vec<Self>),
    /// Bolt passes through N cells before stopping.
    Piercing(u32),
    /// Adds fractional bonus damage per stack.
    DamageBoost(f32),
    /// Bolt chains to N additional cells on hit.
    ChainHit(u32),
    /// Size boost: on `Target::Bolt` adjusts radius, on `Target::Breaker` adjusts width.
    SizeBoost(Target, f32),
    /// Bolt attracts nearby cells (attraction force per stack).
    Attraction(f32),
    /// Flat bump force increase per stack.
    BumpForce(f32),
    /// Flat tilt control sensitivity increase per stack.
    TiltControl(f32),
    /// Ramping damage bonus that accumulates per cell hit and resets on breaker bounce.
    RampingDamage {
        /// Damage bonus added per cell hit.
        bonus_per_hit: f32,
    },
    /// Selects a random effect from a weighted pool of `TriggerChain` entries.
    ///
    /// Each entry is `(weight, chain)`. Inner chains may be leaves or trigger wrappers.
    RandomEffect(Vec<(f32, TriggerChain)>),
    /// Counts cell destructions and fires a random effect from the pool when threshold is reached.
    ///
    /// First field is the threshold count, second is the weighted pool.
    EntropyEngine(u32, Vec<(f32, TriggerChain)>),
}

impl TriggerChain {
    /// Returns the nesting depth of this chain.
    ///
    /// Leaf variants return 0, trigger variants return 1 + inner depth.
    #[must_use]
    pub(crate) fn depth(&self) -> u32 {
        match self {
            Self::Shockwave { .. }
            | Self::MultiBolt { .. }
            | Self::Shield { .. }
            | Self::LoseLife
            | Self::SpawnBolt
            | Self::TimePenalty { .. }
            | Self::SpeedBoost { .. }
            | Self::ChainBolt { .. }
            | Self::ChainLightning { .. }
            | Self::SpawnPhantom { .. }
            | Self::PiercingBeam { .. }
            | Self::GravityWell { .. }
            | Self::SecondWind { .. }
            | Self::Piercing(_)
            | Self::DamageBoost(_)
            | Self::ChainHit(_)
            | Self::SizeBoost(..)
            | Self::Attraction(_)
            | Self::BumpForce(_)
            | Self::TiltControl(_)
            | Self::RampingDamage { .. }
            | Self::RandomEffect(_)
            | Self::EntropyEngine(..) => 0,
            Self::OnPerfectBump(effects)
            | Self::OnImpact(_, effects)
            | Self::OnCellDestroyed(effects)
            | Self::OnBoltLost(effects)
            | Self::OnBump(effects)
            | Self::OnEarlyBump(effects)
            | Self::OnLateBump(effects)
            | Self::OnBumpWhiff(effects)
            | Self::OnSelected(effects) => 1 + effects.iter().map(Self::depth).max().unwrap_or(0),
        }
    }

    /// Returns true if this is a leaf (action) variant, false if it is a trigger wrapper.
    #[must_use]
    pub(crate) fn is_leaf(&self) -> bool {
        self.depth() == 0
    }
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
    pub effects: Vec<crate::effect::definition::EffectNode>,
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
    pub common: Option<RaritySlot>,
    /// Uncommon-rarity variant, if any.
    pub uncommon: Option<RaritySlot>,
    /// Rare-rarity variant, if any.
    pub rare: Option<RaritySlot>,
    /// Legendary-rarity variant, if any.
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
    pub effects: Vec<crate::effect::definition::EffectNode>,
    /// Evolution ingredients. `None` for non-evolution chips.
    #[serde(default)]
    pub ingredients: Option<Vec<EvolutionIngredient>>,
    /// Template this chip was expanded from, if any.
    #[serde(default)]
    pub template_name: Option<String>,
}

#[cfg(test)]
impl ChipDefinition {
    /// Build a test chip with full control over effect and stacking.
    pub(crate) fn test(
        name: &str,
        effect: crate::effect::definition::EffectNode,
        max_stacks: u32,
    ) -> Self {
        Self {
            name: name.to_owned(),
            description: format!("{name} description"),
            rarity: Rarity::Common,
            max_stacks,
            effects: vec![effect],
            ingredients: None,
            template_name: None,
        }
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
}

#[cfg(test)]
impl TriggerChain {
    /// Build a `Shockwave` leaf with `range_per_level: 0.0`, `stacks: 1`, and `speed: 400.0`.
    pub(crate) fn test_shockwave(range: f32) -> Self {
        Self::Shockwave {
            base_range: range,
            range_per_level: 0.0,
            stacks: 1,
            speed: 400.0,
        }
    }

    /// Build a `MultiBolt` leaf with `count_per_level: 0` and `stacks: 1`.
    pub(crate) fn test_multi_bolt(count: u32) -> Self {
        Self::MultiBolt {
            base_count: count,
            count_per_level: 0,
            stacks: 1,
        }
    }

    /// Build a `Shield` leaf with `duration_per_level: 0.0` and `stacks: 1`.
    pub(crate) fn test_shield(duration: f32) -> Self {
        Self::Shield {
            base_duration: duration,
            duration_per_level: 0.0,
            stacks: 1,
        }
    }

    /// Build a `LoseLife` leaf.
    pub(crate) fn test_lose_life() -> Self {
        Self::LoseLife
    }

    /// Build a `TimePenalty` leaf with the given duration in seconds.
    pub(crate) fn test_time_penalty(seconds: f32) -> Self {
        Self::TimePenalty { seconds }
    }

    /// Build a `SpawnBolt` leaf.
    pub(crate) fn test_spawn_bolt() -> Self {
        Self::SpawnBolt
    }

    /// Build a `SpeedBoost` leaf targeting `Bolt` with the given multiplier.
    pub(crate) fn test_speed_boost(multiplier: f32) -> Self {
        Self::SpeedBoost {
            target: Target::Bolt,
            multiplier,
        }
    }

    /// Build a `Piercing` leaf with the given count.
    pub(crate) fn test_piercing(count: u32) -> Self {
        Self::Piercing(count)
    }

    /// Build a `DamageBoost` leaf with the given boost value.
    pub(crate) fn test_damage_boost(boost: f32) -> Self {
        Self::DamageBoost(boost)
    }

    /// Build a `SizeBoost` leaf targeting `Breaker` with the given value.
    pub(crate) fn test_size_boost_breaker(val: f32) -> Self {
        Self::SizeBoost(Target::Breaker, val)
    }

    /// Build a `ChainBolt` leaf with the given tether distance.
    pub(crate) fn test_chain_bolt(tether_distance: f32) -> Self {
        Self::ChainBolt { tether_distance }
    }

    /// Build a `ChainLightning` leaf with given arcs, range, and `damage_mult: 0.5`.
    pub(crate) fn test_chain_lightning(arcs: u32, range: f32) -> Self {
        Self::ChainLightning {
            arcs,
            range,
            damage_mult: 0.5,
        }
    }

    /// Build a `SpawnPhantom` leaf with given duration and `max_active: 2`.
    pub(crate) fn test_spawn_phantom(duration: f32) -> Self {
        Self::SpawnPhantom {
            duration,
            max_active: 2,
        }
    }

    /// Build a `PiercingBeam` leaf with given `damage_mult` and `width: 20.0`.
    pub(crate) fn test_piercing_beam(damage_mult: f32) -> Self {
        Self::PiercingBeam {
            damage_mult,
            width: 20.0,
        }
    }

    /// Build a `GravityWell` leaf with given strength, radius, `duration: 5.0`, and `max: 2`.
    pub(crate) fn test_gravity_well(strength: f32, radius: f32) -> Self {
        Self::GravityWell {
            strength,
            duration: 5.0,
            radius,
            max: 2,
        }
    }

    /// Build a `SecondWind` leaf with the given invulnerability duration.
    pub(crate) fn test_second_wind(invuln_secs: f32) -> Self {
        Self::SecondWind { invuln_secs }
    }

    /// Build a `RampingDamage` leaf with the given per-hit bonus.
    pub(crate) fn test_ramping_damage(bonus_per_hit: f32) -> Self {
        Self::RampingDamage { bonus_per_hit }
    }

    /// Build a `RandomEffect` leaf with the given weighted pool entries.
    pub(crate) fn test_random_effect(pool: Vec<(f32, TriggerChain)>) -> Self {
        Self::RandomEffect(pool)
    }

    /// Build an `EntropyEngine` leaf with the given threshold and pool.
    pub(crate) fn test_entropy_engine(threshold: u32, pool: Vec<(f32, TriggerChain)>) -> Self {
        Self::EntropyEngine(threshold, pool)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::definition::{Effect, EffectNode, Trigger};

    // =========================================================================
    // C7 Wave 1 Part H: ChipDefinition with Vec<EffectNode> (behaviors 40-41)
    // =========================================================================

    #[test]
    fn chip_definition_effects_is_vec_effect_node() {
        let def = ChipDefinition {
            name: "Test".to_owned(),
            description: String::new(),
            rarity: Rarity::Common,
            max_stacks: 3,
            effects: vec![EffectNode::When {
                trigger: Trigger::Selected,
                then: vec![EffectNode::Do(Effect::Piercing(1))],
            }],
            ingredients: None,
            template_name: None,
        };
        assert!(matches!(
            def.effects[0],
            EffectNode::When {
                trigger: Trigger::Selected,
                ..
            }
        ));
    }

    #[test]
    fn chip_definition_empty_effects_is_valid() {
        let def = ChipDefinition {
            name: "Empty".to_owned(),
            description: String::new(),
            rarity: Rarity::Common,
            max_stacks: 1,
            effects: vec![],
            ingredients: None,
            template_name: None,
        };
        assert!(def.effects.is_empty());
    }

    #[test]
    fn chip_definition_ron_with_effect_node_syntax() {
        let ron_str = r#"(name: "Piercing Shot", description: "test", rarity: Common, max_stacks: 3, effects: [When(trigger: OnSelected, then: [Do(Piercing(1))])])"#;
        let def: ChipDefinition =
            ron::de::from_str(ron_str).expect("ChipDefinition with EffectNode RON should parse");
        assert_eq!(def.name, "Piercing Shot");
        assert_eq!(
            def.effects[0],
            EffectNode::When {
                trigger: Trigger::Selected,
                then: vec![EffectNode::Do(Effect::Piercing(1))]
            }
        );
    }

    #[test]
    fn chip_definition_ron_triggered_chain_effect_node() {
        let ron_str = r#"(name: "Surge", description: "...", rarity: Rare, max_stacks: 1, effects: [When(trigger: OnPerfectBump, then: [Do(Shockwave(base_range: 64.0, range_per_level: 0.0, stacks: 1, speed: 400.0))])])"#;
        let def: ChipDefinition = ron::de::from_str(ron_str)
            .expect("ChipDefinition with triggered EffectNode RON should parse");
        assert!(matches!(
            def.effects[0],
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                ..
            }
        ));
    }

    // =========================================================================
    // C7 Wave 1 Part H: RaritySlot with Vec<EffectNode> (behavior 42)
    // =========================================================================

    #[test]
    fn rarity_slot_effects_is_vec_effect_node() {
        let slot = RaritySlot {
            prefix: "Basic".to_owned(),
            effects: vec![EffectNode::When {
                trigger: Trigger::Selected,
                then: vec![EffectNode::Do(Effect::Piercing(1))],
            }],
        };
        assert_eq!(slot.effects.len(), 1);
        assert!(matches!(
            slot.effects[0],
            EffectNode::When {
                trigger: Trigger::Selected,
                ..
            }
        ));
    }

    // =========================================================================
    // C7 Wave 1 Part H: ChipTemplate with Vec<EffectNode> (behavior 43)
    // =========================================================================

    #[test]
    fn chip_template_ron_with_effect_node_syntax() {
        let ron_str = r#"(name: "Surge", max_taken: 3, common: Some((prefix: "Basic", effects: [When(trigger: OnPerfectBump, then: [Do(SpeedBoost(multiplier: 1.2))])])), uncommon: None, rare: None, legendary: None)"#;
        let template: ChipTemplate =
            ron::de::from_str(ron_str).expect("ChipTemplate with EffectNode RON should parse");
        assert_eq!(template.name, "Surge");
        let common = template.common.unwrap();
        assert!(matches!(
            common.effects[0],
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                ..
            }
        ));
    }

    #[test]
    fn expand_template_produces_chip_definition_with_effect_node() {
        let template = ChipTemplate {
            name: "Surge".to_owned(),
            max_taken: 3,
            common: Some(RaritySlot {
                prefix: "Basic".to_owned(),
                effects: vec![EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.2 })],
                }],
            }),
            uncommon: None,
            rare: None,
            legendary: None,
        };
        let defs = expand_template(&template);
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "Basic Surge");
        assert_eq!(defs[0].rarity, Rarity::Common);
        assert_eq!(defs[0].max_stacks, 3);
        assert!(matches!(
            defs[0].effects[0],
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                ..
            }
        ));
    }

    // =========================================================================
    // Preserved tests: ChipDefinition test constructors
    // =========================================================================

    #[test]
    fn chip_definition_test_constructs_with_effect_node() {
        let def = ChipDefinition::test("Piercing", EffectNode::Do(Effect::Piercing(1)), 3);
        assert_eq!(def.name, "Piercing");
        assert_eq!(def.max_stacks, 3);
        assert_eq!(def.effects.len(), 1);
    }

    #[test]
    fn chip_definition_test_simple_constructs() {
        let def = ChipDefinition::test_simple("Test");
        assert_eq!(def.name, "Test");
        assert_eq!(def.max_stacks, 1);
        assert_eq!(def.effects.len(), 1);
        assert!(matches!(
            def.effects[0],
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                ..
            }
        ));
    }

    // =========================================================================
    // Preserved tests: Rarity deserialization
    // =========================================================================

    #[test]
    fn rarity_deserializes_common() {
        let r: Rarity = ron::de::from_str("Common").expect("should parse Common");
        assert_eq!(r, Rarity::Common);
    }

    #[test]
    fn rarity_deserializes_uncommon() {
        let r: Rarity = ron::de::from_str("Uncommon").expect("should parse Uncommon");
        assert_eq!(r, Rarity::Uncommon);
    }

    #[test]
    fn rarity_deserializes_rare() {
        let r: Rarity = ron::de::from_str("Rare").expect("should parse Rare");
        assert_eq!(r, Rarity::Rare);
    }

    #[test]
    fn rarity_deserializes_legendary() {
        let r: Rarity = ron::de::from_str("Legendary").expect("should parse Legendary");
        assert_eq!(r, Rarity::Legendary);
    }

    // =========================================================================
    // Preserved tests: ImpactTarget deserialization
    // =========================================================================

    #[test]
    fn impact_target_deserializes_cell() {
        let t: ImpactTarget = ron::de::from_str("Cell").expect("should parse Cell");
        assert_eq!(t, ImpactTarget::Cell);
    }

    #[test]
    fn impact_target_deserializes_breaker() {
        let t: ImpactTarget = ron::de::from_str("Breaker").expect("should parse Breaker");
        assert_eq!(t, ImpactTarget::Breaker);
    }

    #[test]
    fn impact_target_deserializes_wall() {
        let t: ImpactTarget = ron::de::from_str("Wall").expect("should parse Wall");
        assert_eq!(t, ImpactTarget::Wall);
    }

    // =========================================================================
    // Preserved tests: Target deserialization
    // =========================================================================

    #[test]
    fn target_deserializes_bolt() {
        let t: Target = ron::de::from_str("Bolt").expect("should parse Bolt");
        assert_eq!(t, Target::Bolt);
    }

    #[test]
    fn target_deserializes_breaker() {
        let t: Target = ron::de::from_str("Breaker").expect("should parse Breaker");
        assert_eq!(t, Target::Breaker);
    }

    #[test]
    fn target_deserializes_all_bolts() {
        let t: Target = ron::de::from_str("AllBolts").expect("should parse AllBolts");
        assert_eq!(t, Target::AllBolts);
    }

    #[test]
    fn target_cell_is_valid_variant() {
        let result = ron::de::from_str::<Target>("Cell");
        assert!(result.is_ok(), "Target::Cell should be a valid variant");
        assert_eq!(result.unwrap(), Target::Cell);
    }

    // =========================================================================
    // Preserved tests: EvolutionIngredient
    // =========================================================================

    #[test]
    fn evolution_ingredient_deserializes_from_ron() {
        let ron_str = r#"(chip_name: "Piercing Shot", stacks_required: 2)"#;
        let ingredient: EvolutionIngredient =
            ron::de::from_str(ron_str).expect("should parse EvolutionIngredient");
        assert_eq!(ingredient.chip_name, "Piercing Shot");
        assert_eq!(ingredient.stacks_required, 2);
    }

    // =========================================================================
    // Preserved tests: expand_template mechanics
    // =========================================================================

    #[test]
    fn expand_template_all_none_returns_empty() {
        let template = ChipTemplate {
            name: "Empty".to_owned(),
            max_taken: 1,
            common: None,
            uncommon: None,
            rare: None,
            legendary: None,
        };
        let defs = expand_template(&template);
        assert!(defs.is_empty());
    }

    #[test]
    fn expanded_chip_empty_prefix_uses_template_name() {
        let template = ChipTemplate {
            name: "Glass Cannon".to_owned(),
            max_taken: 1,
            common: None,
            uncommon: None,
            rare: None,
            legendary: Some(RaritySlot {
                prefix: String::new(),
                effects: vec![EffectNode::Do(Effect::DamageBoost(1.0))],
            }),
        };
        let defs = expand_template(&template);
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "Glass Cannon");
        assert_eq!(defs[0].rarity, Rarity::Legendary);
    }

    #[test]
    fn expanded_defs_have_correct_rarities() {
        let template = ChipTemplate {
            name: "AllSlots".to_owned(),
            max_taken: 5,
            common: Some(RaritySlot {
                prefix: "C".to_owned(),
                effects: vec![EffectNode::Do(Effect::Piercing(1))],
            }),
            uncommon: Some(RaritySlot {
                prefix: "U".to_owned(),
                effects: vec![EffectNode::Do(Effect::Piercing(2))],
            }),
            rare: Some(RaritySlot {
                prefix: "R".to_owned(),
                effects: vec![EffectNode::Do(Effect::Piercing(3))],
            }),
            legendary: Some(RaritySlot {
                prefix: "L".to_owned(),
                effects: vec![EffectNode::Do(Effect::Piercing(4))],
            }),
        };
        let defs = expand_template(&template);
        assert_eq!(defs.len(), 4);
        assert_eq!(defs[0].rarity, Rarity::Common);
        assert_eq!(defs[1].rarity, Rarity::Uncommon);
        assert_eq!(defs[2].rarity, Rarity::Rare);
        assert_eq!(defs[3].rarity, Rarity::Legendary);
    }
}
