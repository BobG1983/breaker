//! Chip definition types — shared across Amps, Augments, and Overclocks.

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
}

/// Effect variants for Amp chips (passive bolt upgrades).
#[derive(Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum AmpEffect {
    /// Bolt passes through N cells before stopping.
    Piercing(u32),
    /// Adds fractional bonus damage per stack. Formula: damage = `BASE_BOLT_DAMAGE` * (1.0 + boost).
    DamageBoost(f32),
    /// Adds flat speed per stack.
    SpeedBoost(f32),
    /// Bolt chains to N additional cells on hit.
    ChainHit(u32),
    /// Increases bolt radius by a fraction per stack.
    SizeBoost(f32),
    /// Bolt attracts nearby cells (attraction force per stack).
    Attraction(f32),
}

/// Effect variants for Augment chips (passive breaker upgrades).
#[derive(Deserialize, Clone, Copy, Debug, PartialEq)]
pub(crate) enum AugmentEffect {
    /// Adds flat width per stack.
    WidthBoost(f32),
    /// Adds flat speed per stack.
    SpeedBoost(f32),
    /// Adds flat bump force per stack.
    BumpForce(f32),
    /// Adds flat tilt control sensitivity per stack.
    TiltControl(f32),
}

/// Discriminates which entity a speed boost effect targets.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpeedBoostTarget {
    /// Target the specific bolt that triggered the effect.
    Bolt,
    /// Target the breaker entity.
    Breaker,
    /// Target all bolt entities in play.
    AllBolts,
}

/// Discriminates which surface triggered an impact event.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImpactTarget {
    /// Bolt hit a cell.
    Cell,
    /// Bolt bounced off the breaker.
    Breaker,
    /// Bolt bounced off a wall.
    Wall,
}

/// Trigger chain for Overclock effects — defines when and what happens.
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
        target: SpeedBoostTarget,
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
    OnBumpSuccess(Vec<Self>),
    /// Fires on an early bump.
    OnEarlyBump(Vec<Self>),
    /// Fires on a late bump.
    OnLateBump(Vec<Self>),
    /// Fires when a bump whiffs (misses).
    OnBumpWhiff(Vec<Self>),
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
            | Self::SecondWind { .. } => 0,
            Self::OnPerfectBump(effects)
            | Self::OnImpact(_, effects)
            | Self::OnCellDestroyed(effects)
            | Self::OnBoltLost(effects)
            | Self::OnBumpSuccess(effects)
            | Self::OnEarlyBump(effects)
            | Self::OnLateBump(effects)
            | Self::OnBumpWhiff(effects) => 1 + effects.iter().map(Self::depth).max().unwrap_or(0),
        }
    }

    /// Returns true if this is a leaf (action) variant, false if it is a trigger wrapper.
    #[must_use]
    pub(crate) const fn is_leaf(&self) -> bool {
        matches!(
            self,
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
        )
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

/// A recipe that combines existing chips into a new evolved chip.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct EvolutionRecipe {
    /// Chips consumed by this evolution.
    pub ingredients: Vec<EvolutionIngredient>,
    /// The chip produced when this recipe is fulfilled.
    pub result_definition: ChipDefinition,
}

/// Top-level effect wrapper for any chip type.
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub enum ChipEffect {
    /// Applies an Amp (bolt) effect.
    Amp(AmpEffect),
    /// Applies an Augment (breaker) effect.
    Augment(AugmentEffect),
    /// Triggered ability with a trigger chain.
    Overclock(TriggerChain),
}

/// Triggered when a chip effect should be applied.
///
/// Dispatched by `apply_chip_effect` for each selected chip.
/// Each per-effect observer self-selects via pattern matching on `effect`.
#[derive(Event, Clone, Debug)]
pub(crate) struct ChipEffectApplied {
    /// The effect to apply.
    pub effect: ChipEffect,
    /// Maximum stacks for this chip.
    pub max_stacks: u32,
    /// The chip name for attribution through the trigger chain pipeline.
    pub chip_name: String,
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
    pub effects: Vec<ChipEffect>,
}

#[cfg(test)]
impl ChipDefinition {
    /// Build a test chip with full control over effect and stacking.
    pub(crate) fn test(name: &str, effect: ChipEffect, max_stacks: u32) -> Self {
        Self {
            name: name.to_owned(),
            description: format!("{name} description"),
            rarity: Rarity::Common,
            max_stacks,
            effects: vec![effect],
        }
    }

    /// Build a simple test chip with `Overclock` effect and `max_stacks` = 1.
    pub(crate) fn test_simple(name: &str) -> Self {
        Self::test(
            name,
            ChipEffect::Overclock(TriggerChain::test_shockwave(64.0)),
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
            target: SpeedBoostTarget::Bolt,
            multiplier,
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chip_definition_deserializes_from_ron() {
        let ron_str = r#"(name: "Piercing Shot", description: "Bolt passes through", rarity: Common, max_stacks: 3, effects: [Amp(Piercing(1))])"#;
        let def: ChipDefinition = ron::de::from_str(ron_str).expect("should parse ChipDefinition");
        assert_eq!(def.name, "Piercing Shot");
        assert_eq!(def.description, "Bolt passes through");
        assert_eq!(def.rarity, Rarity::Common);
        assert_eq!(def.max_stacks, 3);
        assert_eq!(def.effects[0], ChipEffect::Amp(AmpEffect::Piercing(1)));
    }

    // --- Part A: New type deserialization tests ---

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

    #[test]
    fn amp_effect_deserializes_piercing() {
        let e: AmpEffect = ron::de::from_str("Piercing(1)").expect("should parse Piercing(1)");
        assert_eq!(e, AmpEffect::Piercing(1));
    }

    #[test]
    fn amp_effect_deserializes_damage_boost() {
        let e: AmpEffect =
            ron::de::from_str("DamageBoost(1.5)").expect("should parse DamageBoost(1.5)");
        assert_eq!(e, AmpEffect::DamageBoost(1.5));
    }

    #[test]
    fn amp_effect_deserializes_speed_boost() {
        let e: AmpEffect =
            ron::de::from_str("SpeedBoost(50.0)").expect("should parse SpeedBoost(50.0)");
        assert_eq!(e, AmpEffect::SpeedBoost(50.0));
    }

    #[test]
    fn amp_effect_deserializes_chain_hit() {
        let e: AmpEffect = ron::de::from_str("ChainHit(2)").expect("should parse ChainHit(2)");
        assert_eq!(e, AmpEffect::ChainHit(2));
    }

    #[test]
    fn amp_effect_deserializes_size_boost() {
        let e: AmpEffect =
            ron::de::from_str("SizeBoost(0.5)").expect("should parse SizeBoost(0.5)");
        assert_eq!(e, AmpEffect::SizeBoost(0.5));
    }

    #[test]
    fn augment_effect_deserializes_width_boost() {
        let e: AugmentEffect =
            ron::de::from_str("WidthBoost(20.0)").expect("should parse WidthBoost(20.0)");
        assert_eq!(e, AugmentEffect::WidthBoost(20.0));
    }

    #[test]
    fn augment_effect_deserializes_speed_boost() {
        let e: AugmentEffect =
            ron::de::from_str("SpeedBoost(30.0)").expect("should parse SpeedBoost(30.0)");
        assert_eq!(e, AugmentEffect::SpeedBoost(30.0));
    }

    #[test]
    fn augment_effect_deserializes_bump_force() {
        let e: AugmentEffect =
            ron::de::from_str("BumpForce(10.0)").expect("should parse BumpForce(10.0)");
        assert_eq!(e, AugmentEffect::BumpForce(10.0));
    }

    #[test]
    fn augment_effect_deserializes_tilt_control() {
        let e: AugmentEffect =
            ron::de::from_str("TiltControl(5.0)").expect("should parse TiltControl(5.0)");
        assert_eq!(e, AugmentEffect::TiltControl(5.0));
    }

    #[test]
    fn chip_effect_deserializes_amp_piercing() {
        let e: ChipEffect =
            ron::de::from_str("Amp(Piercing(1))").expect("should parse Amp(Piercing(1))");
        assert_eq!(e, ChipEffect::Amp(AmpEffect::Piercing(1)));
    }

    #[test]
    fn chip_effect_deserializes_augment_width_boost() {
        let e: ChipEffect = ron::de::from_str("Augment(WidthBoost(20.0))")
            .expect("should parse Augment(WidthBoost(20.0))");
        assert_eq!(e, ChipEffect::Augment(AugmentEffect::WidthBoost(20.0)));
    }

    #[test]
    fn chip_effect_deserializes_overclock() {
        let e: ChipEffect = ron::de::from_str(
            "Overclock(Shockwave(base_range: 64.0, range_per_level: 0.0, stacks: 1, speed: 400.0))",
        )
        .expect("should parse Overclock(Shockwave)");
        assert_eq!(
            e,
            ChipEffect::Overclock(TriggerChain::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })
        );
    }

    #[test]
    fn chip_definition_deserializes_with_all_new_fields() {
        let ron_str = r#"(
            name: "Piercing Shot",
            description: "Bolt passes through",
            rarity: Common,
            max_stacks: 3,
            effects: [Amp(Piercing(1))]
        )"#;
        let def: ChipDefinition =
            ron::de::from_str(ron_str).expect("should parse ChipDefinition with new fields");
        assert_eq!(def.name, "Piercing Shot");
        assert_eq!(def.description, "Bolt passes through");
        assert_eq!(def.rarity, Rarity::Common);
        assert_eq!(def.max_stacks, 3);
        assert_eq!(def.effects[0], ChipEffect::Amp(AmpEffect::Piercing(1)));
    }

    // --- Existing RON file tests (will fail until RON files are updated) ---

    #[test]
    fn amp_ron_parses() {
        let ron_str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/amps/piercing.amp.ron"
        ));
        let def: ChipDefinition = ron::de::from_str(ron_str).expect("amp RON should parse");
        assert_eq!(def.effects[0], ChipEffect::Amp(AmpEffect::Piercing(1)));
    }

    #[test]
    fn augment_ron_parses() {
        let ron_str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/augments/wide_breaker.augment.ron"
        ));
        let def: ChipDefinition = ron::de::from_str(ron_str).expect("augment RON should parse");
        assert_eq!(
            def.effects[0],
            ChipEffect::Augment(AugmentEffect::WidthBoost(20.0))
        );
    }

    #[test]
    fn overclock_ron_parses() {
        let ron_str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/overclocks/surge.overclock.ron"
        ));
        let def: ChipDefinition = ron::de::from_str(ron_str).expect("overclock RON should parse");
        assert_eq!(
            def.effects[0],
            ChipEffect::Overclock(TriggerChain::OnPerfectBump(vec![TriggerChain::OnImpact(
                ImpactTarget::Cell,
                vec![TriggerChain::Shockwave {
                    base_range: 64.0,
                    range_per_level: 32.0,
                    stacks: 1,
                    speed: 400.0,
                }],
            ),]))
        );
    }

    #[test]
    fn chip_definition_with_multiple_effects_deserializes() {
        let ron_str = r#"(
            name: "Hybrid",
            description: "Two effects",
            rarity: Rare,
            max_stacks: 2,
            effects: [Amp(Piercing(1)), Augment(WidthBoost(20.0))]
        )"#;
        let def: ChipDefinition =
            ron::de::from_str(ron_str).expect("should parse ChipDefinition with multiple effects");
        assert_eq!(def.effects.len(), 2);
        assert_eq!(def.effects[0], ChipEffect::Amp(AmpEffect::Piercing(1)));
        assert_eq!(
            def.effects[1],
            ChipEffect::Augment(AugmentEffect::WidthBoost(20.0))
        );
    }

    // --- TriggerChain deserialization tests ---

    #[test]
    fn trigger_chain_deserializes_shockwave() {
        let tc: TriggerChain = ron::de::from_str(
            "Shockwave(base_range: 64.0, range_per_level: 0.0, stacks: 1, speed: 400.0)",
        )
        .expect("should parse Shockwave");
        assert_eq!(
            tc,
            TriggerChain::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            }
        );
    }

    #[test]
    fn trigger_chain_deserializes_multi_bolt() {
        let tc: TriggerChain =
            ron::de::from_str("MultiBolt(base_count: 3, count_per_level: 0, stacks: 1)")
                .expect("should parse MultiBolt");
        assert_eq!(
            tc,
            TriggerChain::MultiBolt {
                base_count: 3,
                count_per_level: 0,
                stacks: 1,
            }
        );
    }

    #[test]
    fn trigger_chain_deserializes_shield() {
        let tc: TriggerChain =
            ron::de::from_str("Shield(base_duration: 5.0, duration_per_level: 0.0, stacks: 1)")
                .expect("should parse Shield");
        assert_eq!(
            tc,
            TriggerChain::Shield {
                base_duration: 5.0,
                duration_per_level: 0.0,
                stacks: 1,
            }
        );
    }

    #[test]
    fn trigger_chain_deserializes_on_perfect_bump_leaf() {
        let tc: TriggerChain = ron::de::from_str(
            "OnPerfectBump([Shockwave(base_range: 64.0, range_per_level: 0.0, stacks: 1, speed: 400.0)])",
        )
        .expect("should parse OnPerfectBump wrapping Shockwave");
        assert_eq!(
            tc,
            TriggerChain::OnPerfectBump(vec![TriggerChain::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            }])
        );
    }

    #[test]
    fn trigger_chain_deserializes_nested_two_deep() {
        let tc: TriggerChain = ron::de::from_str(
            "OnPerfectBump([OnImpact(Cell, [Shockwave(base_range: 64.0, range_per_level: 0.0, stacks: 1, speed: 400.0)])])",
        )
        .expect("should parse double-nested TriggerChain");
        assert_eq!(
            tc,
            TriggerChain::OnPerfectBump(vec![TriggerChain::OnImpact(
                ImpactTarget::Cell,
                vec![TriggerChain::Shockwave {
                    base_range: 64.0,
                    range_per_level: 0.0,
                    stacks: 1,
                    speed: 400.0,
                }],
            )])
        );
    }

    // --- TriggerChain depth tests ---

    #[test]
    fn trigger_chain_depth_leaf_is_zero() {
        assert_eq!(TriggerChain::test_shockwave(64.0).depth(), 0);
        assert_eq!(TriggerChain::test_multi_bolt(3).depth(), 0);
        assert_eq!(TriggerChain::test_shield(5.0).depth(), 0);
        assert_eq!(TriggerChain::test_chain_bolt(200.0).depth(), 0);
    }

    #[test]
    fn trigger_chain_depth_single_trigger_is_one() {
        let tc = TriggerChain::OnPerfectBump(vec![TriggerChain::test_shockwave(64.0)]);
        assert_eq!(tc.depth(), 1);
    }

    #[test]
    fn trigger_chain_depth_nested_is_two() {
        let tc = TriggerChain::OnPerfectBump(vec![TriggerChain::OnImpact(
            ImpactTarget::Cell,
            vec![TriggerChain::test_shockwave(64.0)],
        )]);
        assert_eq!(tc.depth(), 2);
    }

    #[test]
    fn trigger_chain_depth_three_deep_is_three() {
        let tc = TriggerChain::OnPerfectBump(vec![TriggerChain::OnImpact(
            ImpactTarget::Cell,
            vec![TriggerChain::OnCellDestroyed(vec![
                TriggerChain::test_shockwave(64.0),
            ])],
        )]);
        assert_eq!(tc.depth(), 3);
    }

    // --- TriggerChain is_leaf tests ---

    #[test]
    fn trigger_chain_is_leaf_true_for_leaves() {
        assert!(TriggerChain::test_shockwave(64.0).is_leaf());
        assert!(TriggerChain::test_multi_bolt(3).is_leaf());
        assert!(TriggerChain::test_shield(5.0).is_leaf());
        assert!(TriggerChain::test_chain_bolt(200.0).is_leaf());
    }

    #[test]
    fn trigger_chain_is_leaf_false_for_triggers() {
        let leaf = TriggerChain::test_shockwave(64.0);
        assert!(!TriggerChain::OnPerfectBump(vec![leaf.clone()]).is_leaf());
        assert!(!TriggerChain::OnImpact(ImpactTarget::Cell, vec![leaf.clone()]).is_leaf());
        assert!(!TriggerChain::OnCellDestroyed(vec![leaf.clone()]).is_leaf());
        assert!(!TriggerChain::OnBoltLost(vec![leaf.clone()]).is_leaf());
        assert!(!TriggerChain::OnBumpSuccess(vec![leaf]).is_leaf());
    }

    // --- ChipEffect with TriggerChain tests ---

    #[test]
    fn chip_effect_overclock_with_trigger_chain_deserializes() {
        let e: ChipEffect = ron::de::from_str(
            "Overclock(Shockwave(base_range: 64.0, range_per_level: 0.0, stacks: 1, speed: 400.0))",
        )
        .expect("should parse Overclock with TriggerChain");
        assert_eq!(
            e,
            ChipEffect::Overclock(TriggerChain::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })
        );
    }

    #[test]
    fn full_surge_chain_ron_parses() {
        let e: ChipEffect = ron::de::from_str(
            "Overclock(OnPerfectBump([OnImpact(Cell, [Shockwave(base_range: 64.0, range_per_level: 32.0, stacks: 1, speed: 400.0)])]))",
        )
        .expect("should parse full surge chain as ChipEffect");
        assert_eq!(
            e,
            ChipEffect::Overclock(TriggerChain::OnPerfectBump(vec![TriggerChain::OnImpact(
                ImpactTarget::Cell,
                vec![TriggerChain::Shockwave {
                    base_range: 64.0,
                    range_per_level: 32.0,
                    stacks: 1,
                    speed: 400.0,
                }],
            ),]))
        );
    }

    // --- ImpactTarget standalone deserialization ---

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

    // --- OnImpact with Breaker and Wall targets ---

    #[test]
    fn trigger_chain_deserializes_on_impact_breaker_leaf() {
        let tc: TriggerChain = ron::de::from_str(
            "OnImpact(Breaker, [MultiBolt(base_count: 2, count_per_level: 0, stacks: 1)])",
        )
        .expect("should parse OnImpact(Breaker, MultiBolt)");
        assert_eq!(
            tc,
            TriggerChain::OnImpact(
                ImpactTarget::Breaker,
                vec![TriggerChain::MultiBolt {
                    base_count: 2,
                    count_per_level: 0,
                    stacks: 1,
                }],
            )
        );
    }

    #[test]
    fn trigger_chain_deserializes_on_impact_wall_leaf() {
        let tc: TriggerChain = ron::de::from_str(
            "OnImpact(Wall, [Shield(base_duration: 5.0, duration_per_level: 0.0, stacks: 1)])",
        )
        .expect("should parse OnImpact(Wall, Shield)");
        assert_eq!(
            tc,
            TriggerChain::OnImpact(
                ImpactTarget::Wall,
                vec![TriggerChain::Shield {
                    base_duration: 5.0,
                    duration_per_level: 0.0,
                    stacks: 1,
                }],
            )
        );
    }

    // --- OnBumpSuccess deserialization ---

    #[test]
    fn trigger_chain_deserializes_on_bump_success_leaf() {
        let tc: TriggerChain = ron::de::from_str(
            "OnBumpSuccess([Shield(base_duration: 3.0, duration_per_level: 0.0, stacks: 1)])",
        )
        .expect("should parse OnBumpSuccess(Shield)");
        assert_eq!(
            tc,
            TriggerChain::OnBumpSuccess(vec![TriggerChain::Shield {
                base_duration: 3.0,
                duration_per_level: 0.0,
                stacks: 1,
            }])
        );
    }

    // --- OnBumpSuccess depth and is_leaf ---

    #[test]
    fn on_bump_success_depth_is_one() {
        let tc = TriggerChain::OnBumpSuccess(vec![TriggerChain::test_shield(3.0)]);
        assert_eq!(tc.depth(), 1);
    }

    #[test]
    fn on_impact_depth_is_one() {
        let tc =
            TriggerChain::OnImpact(ImpactTarget::Cell, vec![TriggerChain::test_shockwave(64.0)]);
        assert_eq!(tc.depth(), 1);
    }

    // --- Phase D: Stacking effective value tests ---

    /// Compute effective f32 value: `base + (stacks - 1) * per_level`.
    #[expect(
        clippy::cast_precision_loss,
        reason = "stacks is always small (< max_stacks)"
    )]
    fn effective_f32(base: f32, per_level: f32, stacks: u32) -> f32 {
        (stacks.saturating_sub(1) as f32).mul_add(per_level, base)
    }

    #[test]
    fn shockwave_effective_range_at_stacks_1() {
        let effective = effective_f32(64.0, 32.0, 1);
        assert!(
            (effective - 64.0).abs() < f32::EPSILON,
            "stacks=1: effective should be base_range 64.0, got {effective}"
        );
    }

    #[test]
    fn shockwave_effective_range_at_stacks_2() {
        let effective = effective_f32(64.0, 32.0, 2);
        assert!(
            (effective - 96.0).abs() < f32::EPSILON,
            "stacks=2: effective should be 96.0 (64.0 + 1*32.0), got {effective}"
        );
    }

    #[test]
    fn shockwave_effective_range_at_stacks_3() {
        let effective = effective_f32(64.0, 32.0, 3);
        assert!(
            (effective - 128.0).abs() < f32::EPSILON,
            "stacks=3: effective should be 128.0 (64.0 + 2*32.0), got {effective}"
        );
    }

    #[test]
    fn shockwave_effective_range_at_stacks_0() {
        let effective = effective_f32(64.0, 32.0, 0);
        assert!(
            (effective - 64.0).abs() < f32::EPSILON,
            "stacks=0: saturating_sub prevents underflow, effective should be 64.0, got {effective}"
        );
    }

    #[test]
    fn multi_bolt_effective_count_at_stacks_1() {
        let base_count: u32 = 3;
        let count_per_level: u32 = 1;
        let stacks: u32 = 1;
        let effective = base_count + stacks.saturating_sub(1) * count_per_level;
        assert_eq!(
            effective, 3,
            "stacks=1: effective should be base_count 3, got {effective}"
        );
    }

    #[test]
    fn multi_bolt_effective_count_at_stacks_2() {
        let base_count: u32 = 3;
        let count_per_level: u32 = 1;
        let stacks: u32 = 2;
        let effective = base_count + stacks.saturating_sub(1) * count_per_level;
        assert_eq!(
            effective, 4,
            "stacks=2: effective should be 4 (3 + 1*1), got {effective}"
        );
    }

    #[test]
    fn shield_effective_duration_at_stacks_1() {
        let effective = effective_f32(5.0, 2.0, 1);
        assert!(
            (effective - 5.0).abs() < f32::EPSILON,
            "stacks=1: effective should be base_duration 5.0, got {effective}"
        );
    }

    #[test]
    fn shield_effective_duration_at_stacks_3() {
        let effective = effective_f32(5.0, 2.0, 3);
        assert!(
            (effective - 9.0).abs() < f32::EPSILON,
            "stacks=3: effective should be 9.0 (5.0 + 2*2.0), got {effective}"
        );
    }

    // --- Phase D: RON deserialization with new stacking fields ---

    #[test]
    fn shockwave_ron_deserializes_with_new_fields() {
        let tc: TriggerChain = ron::de::from_str(
            "Shockwave(base_range: 64.0, range_per_level: 32.0, stacks: 1, speed: 400.0)",
        )
        .expect("should parse Shockwave with stacking fields");
        assert_eq!(
            tc,
            TriggerChain::Shockwave {
                base_range: 64.0,
                range_per_level: 32.0,
                stacks: 1,
                speed: 400.0,
            }
        );
    }

    #[test]
    fn multi_bolt_ron_deserializes_with_new_fields() {
        let tc: TriggerChain =
            ron::de::from_str("MultiBolt(base_count: 3, count_per_level: 1, stacks: 1)")
                .expect("should parse MultiBolt with stacking fields");
        assert_eq!(
            tc,
            TriggerChain::MultiBolt {
                base_count: 3,
                count_per_level: 1,
                stacks: 1,
            }
        );
    }

    #[test]
    fn shield_ron_deserializes_with_new_fields() {
        let tc: TriggerChain =
            ron::de::from_str("Shield(base_duration: 5.0, duration_per_level: 2.0, stacks: 1)")
                .expect("should parse Shield with stacking fields");
        assert_eq!(
            tc,
            TriggerChain::Shield {
                base_duration: 5.0,
                duration_per_level: 2.0,
                stacks: 1,
            }
        );
    }

    // --- Phase D: Convenience constructor tests ---

    #[test]
    fn test_shockwave_convenience_constructor() {
        assert_eq!(
            TriggerChain::test_shockwave(64.0),
            TriggerChain::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            }
        );
    }

    #[test]
    fn test_multi_bolt_convenience_constructor() {
        assert_eq!(
            TriggerChain::test_multi_bolt(3),
            TriggerChain::MultiBolt {
                base_count: 3,
                count_per_level: 0,
                stacks: 1,
            }
        );
    }

    #[test]
    fn test_shield_convenience_constructor() {
        assert_eq!(
            TriggerChain::test_shield(5.0),
            TriggerChain::Shield {
                base_duration: 5.0,
                duration_per_level: 0.0,
                stacks: 1,
            }
        );
    }

    // --- New leaf variant deserialization tests ---

    #[test]
    fn trigger_chain_deserializes_lose_life() {
        let tc: TriggerChain = ron::de::from_str("LoseLife").expect("should parse LoseLife");
        assert_eq!(tc, TriggerChain::LoseLife);
    }

    #[test]
    fn trigger_chain_deserializes_lose_life_wrapped_in_on_bolt_lost() {
        let tc: TriggerChain =
            ron::de::from_str("OnBoltLost([LoseLife])").expect("should parse OnBoltLost(LoseLife)");
        assert_eq!(tc, TriggerChain::OnBoltLost(vec![TriggerChain::LoseLife]));
    }

    #[test]
    fn trigger_chain_deserializes_time_penalty() {
        let tc: TriggerChain =
            ron::de::from_str("TimePenalty(seconds: 5.0)").expect("should parse TimePenalty");
        assert_eq!(tc, TriggerChain::TimePenalty { seconds: 5.0 });
    }

    #[test]
    fn trigger_chain_deserializes_time_penalty_fractional() {
        let tc: TriggerChain = ron::de::from_str("TimePenalty(seconds: 2.5)")
            .expect("should parse TimePenalty with fractional seconds");
        assert_eq!(tc, TriggerChain::TimePenalty { seconds: 2.5 });
    }

    #[test]
    fn trigger_chain_deserializes_spawn_bolt() {
        let tc: TriggerChain = ron::de::from_str("SpawnBolt").expect("should parse SpawnBolt");
        assert_eq!(tc, TriggerChain::SpawnBolt);
    }

    #[test]
    fn trigger_chain_deserializes_spawn_bolt_wrapped_in_on_bump_success() {
        let tc: TriggerChain = ron::de::from_str("OnBumpSuccess([SpawnBolt])")
            .expect("should parse OnBumpSuccess(SpawnBolt)");
        assert_eq!(
            tc,
            TriggerChain::OnBumpSuccess(vec![TriggerChain::SpawnBolt])
        );
    }

    #[test]
    fn trigger_chain_deserializes_speed_boost_bolt() {
        let tc: TriggerChain = ron::de::from_str("SpeedBoost(target: Bolt, multiplier: 1.5)")
            .expect("should parse SpeedBoost(target: Bolt)");
        assert_eq!(
            tc,
            TriggerChain::SpeedBoost {
                target: SpeedBoostTarget::Bolt,
                multiplier: 1.5,
            }
        );
    }

    #[test]
    fn trigger_chain_deserializes_speed_boost_bolt_identity() {
        let tc: TriggerChain = ron::de::from_str("SpeedBoost(target: Bolt, multiplier: 1.0)")
            .expect("should parse SpeedBoost(target: Bolt) with identity multiplier");
        assert_eq!(
            tc,
            TriggerChain::SpeedBoost {
                target: SpeedBoostTarget::Bolt,
                multiplier: 1.0,
            }
        );
    }

    // --- New trigger variant deserialization tests ---

    #[test]
    fn trigger_chain_deserializes_on_early_bump_wrapping_leaf() {
        let tc: TriggerChain = ron::de::from_str("OnEarlyBump([LoseLife])")
            .expect("should parse OnEarlyBump(LoseLife)");
        assert_eq!(tc, TriggerChain::OnEarlyBump(vec![TriggerChain::LoseLife]));
    }

    #[test]
    fn trigger_chain_deserializes_on_early_bump_nested_two_deep() {
        let tc: TriggerChain = ron::de::from_str(
            "OnEarlyBump([OnImpact(Cell, [Shockwave(base_range: 64.0, range_per_level: 0.0, stacks: 1, speed: 400.0)])])",
        )
        .expect("should parse OnEarlyBump nested two deep");
        assert_eq!(
            tc,
            TriggerChain::OnEarlyBump(vec![TriggerChain::OnImpact(
                ImpactTarget::Cell,
                vec![TriggerChain::Shockwave {
                    base_range: 64.0,
                    range_per_level: 0.0,
                    stacks: 1,
                    speed: 400.0,
                }],
            )])
        );
    }

    #[test]
    fn trigger_chain_deserializes_on_late_bump_wrapping_leaf() {
        let tc: TriggerChain = ron::de::from_str("OnLateBump([TimePenalty(seconds: 3.0)])")
            .expect("should parse OnLateBump(TimePenalty)");
        assert_eq!(
            tc,
            TriggerChain::OnLateBump(vec![TriggerChain::TimePenalty { seconds: 3.0 }])
        );
    }

    #[test]
    fn trigger_chain_deserializes_on_bump_whiff_wrapping_spawn_bolt() {
        let tc: TriggerChain = ron::de::from_str("OnBumpWhiff([SpawnBolt])")
            .expect("should parse OnBumpWhiff(SpawnBolt)");
        assert_eq!(tc, TriggerChain::OnBumpWhiff(vec![TriggerChain::SpawnBolt]));
    }

    #[test]
    fn trigger_chain_deserializes_on_bump_whiff_wrapping_speed_boost() {
        let tc: TriggerChain =
            ron::de::from_str("OnBumpWhiff([SpeedBoost(target: Bolt, multiplier: 1.5)])")
                .expect("should parse OnBumpWhiff(SpeedBoost)");
        assert_eq!(
            tc,
            TriggerChain::OnBumpWhiff(vec![TriggerChain::SpeedBoost {
                target: SpeedBoostTarget::Bolt,
                multiplier: 1.5,
            }])
        );
    }

    // --- New variant depth tests ---

    #[test]
    fn new_leaves_have_depth_zero() {
        assert_eq!(TriggerChain::LoseLife.depth(), 0);
        assert_eq!(TriggerChain::SpawnBolt.depth(), 0);
        assert_eq!(TriggerChain::TimePenalty { seconds: 5.0 }.depth(), 0);
        assert_eq!(
            TriggerChain::SpeedBoost {
                target: SpeedBoostTarget::Bolt,
                multiplier: 1.5,
            }
            .depth(),
            0
        );
    }

    #[test]
    fn new_triggers_wrapping_leaf_have_depth_one() {
        assert_eq!(
            TriggerChain::OnEarlyBump(vec![TriggerChain::LoseLife]).depth(),
            1
        );
        assert_eq!(
            TriggerChain::OnLateBump(vec![TriggerChain::SpawnBolt]).depth(),
            1
        );
        assert_eq!(
            TriggerChain::OnBumpWhiff(vec![TriggerChain::TimePenalty { seconds: 5.0 }]).depth(),
            1
        );
    }

    #[test]
    fn on_bump_whiff_nested_two_deep_has_depth_two() {
        let tc = TriggerChain::OnBumpWhiff(vec![TriggerChain::OnImpact(
            ImpactTarget::Cell,
            vec![TriggerChain::LoseLife],
        )]);
        assert_eq!(tc.depth(), 2);
    }

    // --- New variant is_leaf tests ---

    #[test]
    fn new_leaves_return_is_leaf_true() {
        assert!(TriggerChain::LoseLife.is_leaf());
        assert!(TriggerChain::SpawnBolt.is_leaf());
        assert!(TriggerChain::TimePenalty { seconds: 5.0 }.is_leaf());
        assert!(
            TriggerChain::SpeedBoost {
                target: SpeedBoostTarget::Bolt,
                multiplier: 1.5,
            }
            .is_leaf()
        );
    }

    #[test]
    fn new_triggers_return_is_leaf_false() {
        assert!(!TriggerChain::OnEarlyBump(vec![TriggerChain::LoseLife]).is_leaf());
        assert!(!TriggerChain::OnLateBump(vec![TriggerChain::SpawnBolt]).is_leaf());
        assert!(
            !TriggerChain::OnBumpWhiff(vec![TriggerChain::TimePenalty { seconds: 5.0 }]).is_leaf()
        );
    }

    // --- Convenience constructor tests ---

    #[test]
    fn test_lose_life_convenience_constructor() {
        assert_eq!(TriggerChain::test_lose_life(), TriggerChain::LoseLife);
    }

    #[test]
    fn test_time_penalty_convenience_constructor() {
        assert_eq!(
            TriggerChain::test_time_penalty(5.0),
            TriggerChain::TimePenalty { seconds: 5.0 }
        );
    }

    #[test]
    fn test_spawn_bolt_convenience_constructor() {
        assert_eq!(TriggerChain::test_spawn_bolt(), TriggerChain::SpawnBolt);
    }

    // --- ChipEffect integration with new variants ---

    #[test]
    fn chip_effect_overclock_with_on_bump_whiff_lose_life() {
        let e: ChipEffect = ron::de::from_str("Overclock(OnBumpWhiff([LoseLife]))")
            .expect("should parse Overclock(OnBumpWhiff(LoseLife))");
        assert_eq!(
            e,
            ChipEffect::Overclock(TriggerChain::OnBumpWhiff(vec![TriggerChain::LoseLife]))
        );
    }

    // --- SpeedBoostTarget deserialization tests ---

    #[test]
    fn speed_boost_target_deserializes_bolt() {
        let t: SpeedBoostTarget = ron::de::from_str("Bolt").expect("should parse Bolt");
        assert_eq!(t, SpeedBoostTarget::Bolt);
    }

    #[test]
    fn speed_boost_target_deserializes_breaker() {
        let t: SpeedBoostTarget = ron::de::from_str("Breaker").expect("should parse Breaker");
        assert_eq!(t, SpeedBoostTarget::Breaker);
    }

    #[test]
    fn speed_boost_target_deserializes_all_bolts() {
        let t: SpeedBoostTarget = ron::de::from_str("AllBolts").expect("should parse AllBolts");
        assert_eq!(t, SpeedBoostTarget::AllBolts);
    }

    // --- SpeedBoost variant deserialization tests ---

    #[test]
    fn trigger_chain_deserializes_speed_boost() {
        let tc: TriggerChain = ron::de::from_str("SpeedBoost(target: Bolt, multiplier: 1.5)")
            .expect("should parse SpeedBoost");
        assert_eq!(
            tc,
            TriggerChain::SpeedBoost {
                target: SpeedBoostTarget::Bolt,
                multiplier: 1.5,
            }
        );
    }

    #[test]
    fn trigger_chain_deserializes_speed_boost_wrapped_in_on_perfect_bump() {
        let tc: TriggerChain =
            ron::de::from_str("OnPerfectBump([SpeedBoost(target: Bolt, multiplier: 1.5)])")
                .expect("should parse OnPerfectBump(SpeedBoost)");
        assert_eq!(
            tc,
            TriggerChain::OnPerfectBump(vec![TriggerChain::SpeedBoost {
                target: SpeedBoostTarget::Bolt,
                multiplier: 1.5,
            }])
        );
    }

    // --- SpeedBoost depth and is_leaf tests ---

    #[test]
    fn speed_boost_depth_is_zero() {
        let tc = TriggerChain::SpeedBoost {
            target: SpeedBoostTarget::Bolt,
            multiplier: 1.5,
        };
        assert_eq!(tc.depth(), 0);
    }

    #[test]
    fn speed_boost_is_leaf_true() {
        let tc = TriggerChain::SpeedBoost {
            target: SpeedBoostTarget::Bolt,
            multiplier: 1.5,
        };
        assert!(tc.is_leaf());
    }

    // --- SpeedBoost convenience constructor test ---

    #[test]
    fn test_speed_boost_convenience_constructor() {
        assert_eq!(
            TriggerChain::test_speed_boost(1.5),
            TriggerChain::SpeedBoost {
                target: SpeedBoostTarget::Bolt,
                multiplier: 1.5,
            }
        );
    }

    // --- Evolution types deserialization tests ---

    #[test]
    fn evolution_ingredient_deserializes_from_ron() {
        let ron_str = r#"(chip_name: "Piercing Shot", stacks_required: 2)"#;
        let ingredient: EvolutionIngredient =
            ron::de::from_str(ron_str).expect("should parse EvolutionIngredient");
        assert_eq!(ingredient.chip_name, "Piercing Shot");
        assert_eq!(ingredient.stacks_required, 2);
    }

    #[test]
    fn evolution_recipe_deserializes_with_full_result_definition() {
        let ron_str = r#"(
            ingredients: [
                (chip_name: "Piercing Shot", stacks_required: 2),
                (chip_name: "Damage Up", stacks_required: 1),
            ],
            result_definition: (
                name: "Piercing Barrage",
                description: "Evolved piercing",
                rarity: Legendary,
                max_stacks: 1,
                effects: [Amp(Piercing(5))],
            ),
        )"#;
        let recipe: EvolutionRecipe =
            ron::de::from_str(ron_str).expect("should parse EvolutionRecipe");
        assert_eq!(recipe.ingredients.len(), 2);
        assert_eq!(recipe.ingredients[0].chip_name, "Piercing Shot");
        assert_eq!(recipe.ingredients[0].stacks_required, 2);
        assert_eq!(recipe.ingredients[1].chip_name, "Damage Up");
        assert_eq!(recipe.ingredients[1].stacks_required, 1);
        assert_eq!(recipe.result_definition.name, "Piercing Barrage");
        assert_eq!(recipe.result_definition.rarity, Rarity::Legendary);
        assert_eq!(recipe.result_definition.max_stacks, 1);
        assert_eq!(
            recipe.result_definition.effects[0],
            ChipEffect::Amp(AmpEffect::Piercing(5))
        );
    }

    #[test]
    fn evolution_recipe_with_empty_ingredients_deserializes() {
        let ron_str = r#"(
            ingredients: [],
            result_definition: (
                name: "Empty Recipe",
                description: "No ingredients",
                rarity: Common,
                max_stacks: 1,
                effects: [Amp(Piercing(1))],
            ),
        )"#;
        let recipe: EvolutionRecipe = ron::de::from_str(ron_str)
            .expect("should parse EvolutionRecipe with empty ingredients");
        assert_eq!(recipe.ingredients.len(), 0);
    }

    // --- ChainBolt variant tests ---

    #[test]
    fn trigger_chain_deserializes_chain_bolt() {
        let tc: TriggerChain =
            ron::de::from_str("ChainBolt(tether_distance: 200.0)").expect("should parse ChainBolt");
        assert_eq!(
            tc,
            TriggerChain::ChainBolt {
                tether_distance: 200.0,
            }
        );
    }

    #[test]
    fn trigger_chain_deserializes_chain_bolt_wrapped_in_on_perfect_bump() {
        let tc: TriggerChain =
            ron::de::from_str("OnPerfectBump([ChainBolt(tether_distance: 150.0)])")
                .expect("should parse OnPerfectBump(ChainBolt)");
        assert_eq!(
            tc,
            TriggerChain::OnPerfectBump(vec![TriggerChain::ChainBolt {
                tether_distance: 150.0,
            }])
        );
    }

    #[test]
    fn chain_bolt_depth_is_zero() {
        assert_eq!(
            TriggerChain::ChainBolt {
                tether_distance: 200.0
            }
            .depth(),
            0
        );
    }

    #[test]
    fn chain_bolt_is_leaf_true() {
        assert!(
            TriggerChain::ChainBolt {
                tether_distance: 200.0
            }
            .is_leaf()
        );
    }

    #[test]
    fn test_chain_bolt_convenience_constructor() {
        assert_eq!(
            TriggerChain::test_chain_bolt(200.0),
            TriggerChain::ChainBolt {
                tether_distance: 200.0,
            }
        );
    }

    #[test]
    fn chip_effect_overclock_with_chain_bolt_deserializes() {
        let e: ChipEffect = ron::de::from_str("Overclock(ChainBolt(tether_distance: 200.0))")
            .expect("should parse Overclock(ChainBolt)");
        assert_eq!(
            e,
            ChipEffect::Overclock(TriggerChain::ChainBolt {
                tether_distance: 200.0,
            })
        );
    }

    // --- ChainLightning leaf variant tests ---

    #[test]
    fn chain_lightning_is_leaf_with_depth_zero() {
        let tc = TriggerChain::ChainLightning {
            arcs: 3,
            range: 96.0,
            damage_mult: 0.5,
        };
        assert!(tc.is_leaf(), "ChainLightning should be a leaf");
        assert_eq!(tc.depth(), 0, "ChainLightning depth should be 0");
    }

    // --- SpawnPhantom leaf variant tests ---

    #[test]
    fn spawn_phantom_is_leaf_with_depth_zero() {
        let tc = TriggerChain::SpawnPhantom {
            duration: 3.0,
            max_active: 2,
        };
        assert!(tc.is_leaf(), "SpawnPhantom should be a leaf");
        assert_eq!(tc.depth(), 0, "SpawnPhantom depth should be 0");
    }

    // --- PiercingBeam leaf variant tests ---

    #[test]
    fn piercing_beam_is_leaf_with_depth_zero() {
        let tc = TriggerChain::PiercingBeam {
            damage_mult: 2.0,
            width: 20.0,
        };
        assert!(tc.is_leaf(), "PiercingBeam should be a leaf");
        assert_eq!(tc.depth(), 0, "PiercingBeam depth should be 0");
    }

    // --- GravityWell leaf variant tests ---

    #[test]
    fn gravity_well_is_leaf_with_depth_zero() {
        let tc = TriggerChain::GravityWell {
            strength: 500.0,
            duration: 5.0,
            radius: 128.0,
            max: 2,
        };
        assert!(tc.is_leaf(), "GravityWell should be a leaf");
        assert_eq!(tc.depth(), 0, "GravityWell depth should be 0");
    }

    // --- SecondWind leaf variant tests ---

    #[test]
    fn second_wind_is_leaf_with_depth_zero() {
        let tc = TriggerChain::SecondWind { invuln_secs: 2.0 };
        assert!(tc.is_leaf(), "SecondWind should be a leaf");
        assert_eq!(tc.depth(), 0, "SecondWind depth should be 0");
    }

    // --- Trigger wrapping new leaf has depth 1 ---

    #[test]
    fn on_cell_destroyed_wrapping_chain_lightning_has_depth_one() {
        let tc = TriggerChain::OnCellDestroyed(vec![TriggerChain::ChainLightning {
            arcs: 3,
            range: 96.0,
            damage_mult: 0.5,
        }]);
        assert_eq!(
            tc.depth(),
            1,
            "OnCellDestroyed wrapping ChainLightning should have depth 1"
        );
        assert!(
            !tc.is_leaf(),
            "OnCellDestroyed wrapping ChainLightning should not be a leaf"
        );
    }

    // --- AmpEffect::Attraction pattern match test ---

    #[test]
    fn amp_effect_attraction_matches_correctly() {
        let effect = ChipEffect::Amp(AmpEffect::Attraction(8.0));
        match effect {
            ChipEffect::Amp(AmpEffect::Attraction(force)) => {
                assert!(
                    (force - 8.0).abs() < f32::EPSILON,
                    "Attraction force should be 8.0, got {force}"
                );
            }
            other => panic!("expected Amp(Attraction(8.0)), got {other:?}"),
        }
    }

    // --- New leaf convenience constructor tests ---

    #[test]
    fn test_chain_lightning_convenience_constructor() {
        assert_eq!(
            TriggerChain::test_chain_lightning(3, 96.0),
            TriggerChain::ChainLightning {
                arcs: 3,
                range: 96.0,
                damage_mult: 0.5,
            }
        );
    }

    #[test]
    fn test_spawn_phantom_convenience_constructor() {
        assert_eq!(
            TriggerChain::test_spawn_phantom(3.0),
            TriggerChain::SpawnPhantom {
                duration: 3.0,
                max_active: 2,
            }
        );
    }

    #[test]
    fn test_piercing_beam_convenience_constructor() {
        assert_eq!(
            TriggerChain::test_piercing_beam(2.0),
            TriggerChain::PiercingBeam {
                damage_mult: 2.0,
                width: 20.0,
            }
        );
    }

    #[test]
    fn test_gravity_well_convenience_constructor() {
        assert_eq!(
            TriggerChain::test_gravity_well(500.0, 128.0),
            TriggerChain::GravityWell {
                strength: 500.0,
                duration: 5.0,
                radius: 128.0,
                max: 2,
            }
        );
    }

    #[test]
    fn test_second_wind_convenience_constructor() {
        assert_eq!(
            TriggerChain::test_second_wind(2.0),
            TriggerChain::SecondWind { invuln_secs: 2.0 }
        );
    }
}
