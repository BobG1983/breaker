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
        /// Maximum cumulative damage bonus before capping.
        max_bonus: f32,
    },
    /// Temporary speed burst applied to a bolt, decaying over time.
    TimedSpeedBurst {
        /// Multiplier applied to bolt velocity.
        speed_mult: f32,
        /// Duration of the burst in seconds.
        duration_secs: f32,
    },
    /// Time-pressure speed boost applied to bolts when node timer drops below threshold.
    TimePressureBoost {
        /// Speed multiplier applied to bolt velocity when active.
        speed_mult: f32,
        /// Timer ratio threshold (remaining/total) below which boost activates.
        threshold_pct: f32,
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
            | Self::TimedSpeedBurst { .. }
            | Self::TimePressureBoost { .. }
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
    pub effects: Vec<TriggerChain>,
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
    pub effects: Vec<TriggerChain>,
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
    pub(crate) fn test(name: &str, effect: TriggerChain, max_stacks: u32) -> Self {
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
        Self::test(
            name,
            TriggerChain::OnPerfectBump(vec![TriggerChain::test_shockwave(64.0)]),
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

    /// Build a `RampingDamage` leaf with the given per-hit bonus and max bonus.
    pub(crate) fn test_ramping_damage(bonus_per_hit: f32, max_bonus: f32) -> Self {
        Self::RampingDamage {
            bonus_per_hit,
            max_bonus,
        }
    }

    /// Build a `TimedSpeedBurst` leaf with the given speed multiplier and duration.
    pub(crate) fn test_timed_speed_burst(speed_mult: f32, duration_secs: f32) -> Self {
        Self::TimedSpeedBurst {
            speed_mult,
            duration_secs,
        }
    }

    /// Build a `TimePressureBoost` leaf with the given speed multiplier and threshold.
    pub(crate) fn test_time_pressure_boost(speed_mult: f32, threshold_pct: f32) -> Self {
        Self::TimePressureBoost {
            speed_mult,
            threshold_pct,
        }
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

    #[test]
    fn chip_definition_deserializes_with_on_selected_piercing() {
        let ron_str = r#"(name: "Piercing Shot", description: "Bolt passes through", rarity: Common, max_stacks: 3, effects: [OnSelected([Piercing(1)])])"#;
        let def: ChipDefinition = ron::de::from_str(ron_str).expect("should parse ChipDefinition");
        assert_eq!(def.name, "Piercing Shot");
        assert_eq!(def.description, "Bolt passes through");
        assert_eq!(def.rarity, Rarity::Common);
        assert_eq!(def.max_stacks, 3);
        assert_eq!(
            def.effects[0],
            TriggerChain::OnSelected(vec![TriggerChain::Piercing(1)])
        );
    }

    // --- B1: Target enum deserialization (behavior 1) ---

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
    fn target_rejects_invalid_variant() {
        let result = ron::de::from_str::<Target>("Cell");
        assert!(result.is_err(), "Target should not accept Cell variant");
    }

    // --- B1: OnSelected deserialization (behaviors 2-4) ---

    #[test]
    fn on_selected_deserializes_with_inner_leaf() {
        let tc: TriggerChain = ron::de::from_str("OnSelected([Piercing(1)])")
            .expect("should parse OnSelected([Piercing(1)])");
        assert_eq!(
            tc,
            TriggerChain::OnSelected(vec![TriggerChain::Piercing(1)])
        );
    }

    #[test]
    fn on_selected_deserializes_empty_vec() {
        let tc: TriggerChain =
            ron::de::from_str("OnSelected([])").expect("should parse OnSelected([])");
        assert_eq!(tc, TriggerChain::OnSelected(vec![]));
    }

    #[test]
    fn on_selected_deserializes_with_multiple_leaves() {
        let tc: TriggerChain = ron::de::from_str("OnSelected([Piercing(1), DamageBoost(0.5)])")
            .expect("should parse OnSelected with multiple leaves");
        assert_eq!(
            tc,
            TriggerChain::OnSelected(vec![
                TriggerChain::Piercing(1),
                TriggerChain::DamageBoost(0.5),
            ])
        );
    }

    #[test]
    fn on_selected_with_size_boost_breaker_deserializes() {
        let tc: TriggerChain = ron::de::from_str("OnSelected([SizeBoost(Breaker, 20.0)])")
            .expect("should parse OnSelected([SizeBoost(Breaker, 20.0)])");
        assert_eq!(
            tc,
            TriggerChain::OnSelected(vec![TriggerChain::SizeBoost(Target::Breaker, 20.0)])
        );
    }

    #[test]
    fn on_selected_with_size_boost_bolt_deserializes() {
        let tc: TriggerChain = ron::de::from_str("OnSelected([SizeBoost(Bolt, 0.3)])")
            .expect("should parse SizeBoost(Bolt, 0.3)");
        assert_eq!(
            tc,
            TriggerChain::OnSelected(vec![TriggerChain::SizeBoost(Target::Bolt, 0.3)])
        );
    }

    // --- B1: New leaf variants deserialize standalone (behavior 5) ---

    #[test]
    fn trigger_chain_deserializes_piercing_leaf() {
        let tc: TriggerChain = ron::de::from_str("Piercing(1)").expect("should parse Piercing(1)");
        assert_eq!(tc, TriggerChain::Piercing(1));
    }

    #[test]
    fn trigger_chain_deserializes_piercing_zero() {
        let tc: TriggerChain = ron::de::from_str("Piercing(0)").expect("should parse Piercing(0)");
        assert_eq!(tc, TriggerChain::Piercing(0));
    }

    #[test]
    fn trigger_chain_deserializes_damage_boost_leaf() {
        let tc: TriggerChain =
            ron::de::from_str("DamageBoost(0.5)").expect("should parse DamageBoost(0.5)");
        assert_eq!(tc, TriggerChain::DamageBoost(0.5));
    }

    #[test]
    fn trigger_chain_deserializes_chain_hit_leaf() {
        let tc: TriggerChain = ron::de::from_str("ChainHit(2)").expect("should parse ChainHit(2)");
        assert_eq!(tc, TriggerChain::ChainHit(2));
    }

    #[test]
    fn trigger_chain_deserializes_size_boost_bolt() {
        let tc: TriggerChain =
            ron::de::from_str("SizeBoost(Bolt, 0.3)").expect("should parse SizeBoost(Bolt, 0.3)");
        assert_eq!(tc, TriggerChain::SizeBoost(Target::Bolt, 0.3));
    }

    #[test]
    fn trigger_chain_deserializes_attraction_leaf() {
        let tc: TriggerChain =
            ron::de::from_str("Attraction(8.0)").expect("should parse Attraction(8.0)");
        assert_eq!(tc, TriggerChain::Attraction(8.0));
    }

    #[test]
    fn trigger_chain_deserializes_bump_force_leaf() {
        let tc: TriggerChain =
            ron::de::from_str("BumpForce(10.0)").expect("should parse BumpForce(10.0)");
        assert_eq!(tc, TriggerChain::BumpForce(10.0));
    }

    #[test]
    fn trigger_chain_deserializes_tilt_control_leaf() {
        let tc: TriggerChain =
            ron::de::from_str("TiltControl(5.0)").expect("should parse TiltControl(5.0)");
        assert_eq!(tc, TriggerChain::TiltControl(5.0));
    }

    // --- B1: SpeedBoost now uses Target (behavior 6) ---

    #[test]
    fn speed_boost_uses_target_instead_of_speed_boost_target() {
        let tc: TriggerChain = ron::de::from_str("SpeedBoost(target: Bolt, multiplier: 1.5)")
            .expect("should parse SpeedBoost with Target");
        assert_eq!(
            tc,
            TriggerChain::SpeedBoost {
                target: Target::Bolt,
                multiplier: 1.5,
            }
        );
    }

    #[test]
    fn speed_boost_all_bolts_identity() {
        let tc: TriggerChain = ron::de::from_str("SpeedBoost(target: AllBolts, multiplier: 1.0)")
            .expect("should parse SpeedBoost AllBolts identity");
        assert_eq!(
            tc,
            TriggerChain::SpeedBoost {
                target: Target::AllBolts,
                multiplier: 1.0,
            }
        );
    }

    // --- B1: OnBump replaces OnBumpSuccess (behavior 7) ---

    #[test]
    fn on_bump_deserializes_with_spawn_bolt() {
        let tc: TriggerChain =
            ron::de::from_str("OnBump([SpawnBolt])").expect("should parse OnBump([SpawnBolt])");
        assert_eq!(tc, TriggerChain::OnBump(vec![TriggerChain::SpawnBolt]));
    }

    #[test]
    fn on_bump_deserializes_nested_depth_two() {
        let tc: TriggerChain = ron::de::from_str(
            "OnBump([OnImpact(Cell, [Shockwave(base_range: 64.0, range_per_level: 0.0, stacks: 1, speed: 400.0)])])",
        )
        .expect("should parse OnBump nested depth 2");
        assert_eq!(
            tc,
            TriggerChain::OnBump(vec![TriggerChain::OnImpact(
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

    // --- B1: New leaf variants depth (behavior 8) ---

    #[test]
    fn new_chip_leaf_variants_have_depth_zero() {
        assert_eq!(TriggerChain::Piercing(1).depth(), 0);
        assert_eq!(TriggerChain::DamageBoost(0.5).depth(), 0);
        assert_eq!(TriggerChain::ChainHit(2).depth(), 0);
        assert_eq!(TriggerChain::SizeBoost(Target::Bolt, 0.3).depth(), 0);
        assert_eq!(TriggerChain::Attraction(8.0).depth(), 0);
        assert_eq!(TriggerChain::BumpForce(10.0).depth(), 0);
        assert_eq!(TriggerChain::TiltControl(5.0).depth(), 0);
    }

    // --- B1: New leaf variants are leaves (behavior 9) ---

    #[test]
    fn new_chip_leaf_variants_are_leaves() {
        assert!(TriggerChain::Piercing(1).is_leaf());
        assert!(TriggerChain::DamageBoost(0.5).is_leaf());
        assert!(TriggerChain::ChainHit(2).is_leaf());
        assert!(TriggerChain::SizeBoost(Target::Bolt, 0.3).is_leaf());
        assert!(TriggerChain::Attraction(8.0).is_leaf());
        assert!(TriggerChain::BumpForce(10.0).is_leaf());
        assert!(TriggerChain::TiltControl(5.0).is_leaf());
    }

    // --- B1: OnSelected is NOT a leaf (behavior 10) ---

    #[test]
    fn on_selected_is_not_a_leaf() {
        assert!(!TriggerChain::OnSelected(vec![TriggerChain::Piercing(1)]).is_leaf());
    }

    #[test]
    fn on_selected_empty_is_not_a_leaf() {
        assert!(!TriggerChain::OnSelected(vec![]).is_leaf());
    }

    // --- B1: OnSelected depth (behavior 11) ---

    #[test]
    fn on_selected_depth_is_one_plus_inner() {
        assert_eq!(
            TriggerChain::OnSelected(vec![TriggerChain::Piercing(1)]).depth(),
            1
        );
    }

    #[test]
    fn on_selected_nested_depth_is_two() {
        assert_eq!(
            TriggerChain::OnSelected(vec![TriggerChain::OnPerfectBump(vec![
                TriggerChain::SpawnBolt
            ])])
            .depth(),
            2
        );
    }

    // --- B1: OnBump depth and is_leaf (behavior 12) ---

    #[test]
    fn on_bump_depth_is_one() {
        let tc = TriggerChain::OnBump(vec![TriggerChain::test_shield(3.0)]);
        assert_eq!(tc.depth(), 1);
        assert!(!tc.is_leaf());
    }

    // --- B1: ChipDefinition with TriggerChain effects (behavior 13) ---

    #[test]
    fn chip_definition_with_trigger_chain_effects_deserializes() {
        let ron_str = r#"(name: "Piercing Shot", description: "Bolt passes through", rarity: Common, max_stacks: 3, effects: [OnSelected([Piercing(1)])])"#;
        let def: ChipDefinition =
            ron::de::from_str(ron_str).expect("should parse ChipDefinition with TriggerChain");
        assert_eq!(
            def.effects[0],
            TriggerChain::OnSelected(vec![TriggerChain::Piercing(1)])
        );
    }

    #[test]
    fn chip_definition_multi_effect_deserializes() {
        let ron_str = r#"(
            name: "Hybrid",
            description: "Two effects",
            rarity: Rare,
            max_stacks: 2,
            effects: [OnSelected([SizeBoost(Breaker, 20.0)]), OnPerfectBump([SpawnBolt])]
        )"#;
        let def: ChipDefinition = ron::de::from_str(ron_str)
            .expect("should parse ChipDefinition with multiple TriggerChain effects");
        assert_eq!(def.effects.len(), 2);
        assert_eq!(
            def.effects[0],
            TriggerChain::OnSelected(vec![TriggerChain::SizeBoost(Target::Breaker, 20.0)])
        );
        assert_eq!(
            def.effects[1],
            TriggerChain::OnPerfectBump(vec![TriggerChain::SpawnBolt])
        );
    }

    // --- B1: ChipDefinition with triggered chain (behavior 14) ---

    #[test]
    fn chip_definition_triggered_chain_no_on_selected_wrapper() {
        let ron_str = r#"(name: "Surge", description: "...", rarity: Rare, max_stacks: 1, effects: [OnPerfectBump([OnImpact(Cell, [Shockwave(base_range: 64.0, range_per_level: 32.0, stacks: 1, speed: 400.0)])])])"#;
        let def: ChipDefinition =
            ron::de::from_str(ron_str).expect("should parse ChipDefinition with triggered chain");
        assert_eq!(
            def.effects[0],
            TriggerChain::OnPerfectBump(vec![TriggerChain::OnImpact(
                ImpactTarget::Cell,
                vec![TriggerChain::Shockwave {
                    base_range: 64.0,
                    range_per_level: 32.0,
                    stacks: 1,
                    speed: 400.0,
                }],
            )])
        );
    }

    // --- B1: Representative RON chip files (behavior 15) ---
    // These will fail until RON files are updated by writer-code

    #[test]
    fn piercing_chip_template_ron_parses() {
        let ron_str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/chips/piercing.chip.ron"
        ));
        let template: ChipTemplate =
            ron::de::from_str(ron_str).expect("chip template RON should parse");
        assert_eq!(template.name, "Piercing Shot");
        assert_eq!(template.max_taken, 3);
        assert!(template.common.is_some());
        let defs = expand_template(&template);
        assert_eq!(defs.len(), 3);
        assert_eq!(defs[0].name, "Basic Piercing Shot");
        assert_eq!(
            defs[0].effects[0],
            TriggerChain::OnSelected(vec![TriggerChain::Piercing(1)])
        );
    }

    #[test]
    fn wide_breaker_chip_template_ron_parses() {
        let ron_str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/chips/wide_breaker.chip.ron"
        ));
        let template: ChipTemplate =
            ron::de::from_str(ron_str).expect("chip template RON should parse");
        assert_eq!(template.name, "Wide Breaker");
        let defs = expand_template(&template);
        assert_eq!(defs.len(), 3);
        assert_eq!(
            defs[0].effects[0],
            TriggerChain::OnSelected(vec![TriggerChain::SizeBoost(Target::Breaker, 10.0)])
        );
    }

    #[test]
    fn surge_chip_template_ron_parses() {
        let ron_str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/chips/surge.chip.ron"
        ));
        let template: ChipTemplate =
            ron::de::from_str(ron_str).expect("chip template RON should parse");
        assert_eq!(template.name, "Surge");
        let defs = expand_template(&template);
        assert_eq!(defs.len(), 3);
        assert_eq!(
            defs[0].effects[0],
            TriggerChain::OnPerfectBump(vec![TriggerChain::SpeedBoost {
                target: Target::Bolt,
                multiplier: 1.2,
            }])
        );
    }

    #[test]
    fn phantom_breaker_evolution_ron_parses_with_on_bump() {
        let ron_str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/chips/evolution/phantom_breaker.evolution.ron"
        ));
        let def: ChipDefinition =
            ron::de::from_str(ron_str).expect("phantom_breaker evolution RON should parse");
        assert_eq!(def.name, "Phantom Breaker");
        assert_eq!(def.rarity, Rarity::Evolution);
        // The RON file should use OnBump (not OnBumpSuccess) wrapping SpawnPhantom
        assert_eq!(
            def.effects[0],
            TriggerChain::OnBump(vec![TriggerChain::SpawnPhantom {
                duration: 5.0,
                max_active: 1,
            }])
        );
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
        assert!(!TriggerChain::OnBoltLost(vec![leaf]).is_leaf());
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
    fn trigger_chain_deserializes_speed_boost_bolt() {
        let tc: TriggerChain = ron::de::from_str("SpeedBoost(target: Bolt, multiplier: 1.5)")
            .expect("should parse SpeedBoost(target: Bolt)");
        assert_eq!(
            tc,
            TriggerChain::SpeedBoost {
                target: Target::Bolt,
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
                target: Target::Bolt,
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
                target: Target::Bolt,
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
                target: Target::Bolt,
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
                target: Target::Bolt,
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

    // --- SpeedBoost variant deserialization tests ---

    #[test]
    fn trigger_chain_deserializes_speed_boost() {
        let tc: TriggerChain = ron::de::from_str("SpeedBoost(target: Bolt, multiplier: 1.5)")
            .expect("should parse SpeedBoost");
        assert_eq!(
            tc,
            TriggerChain::SpeedBoost {
                target: Target::Bolt,
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
                target: Target::Bolt,
                multiplier: 1.5,
            }])
        );
    }

    // --- SpeedBoost depth and is_leaf tests ---

    #[test]
    fn speed_boost_depth_is_zero() {
        let tc = TriggerChain::SpeedBoost {
            target: Target::Bolt,
            multiplier: 1.5,
        };
        assert_eq!(tc.depth(), 0);
    }

    #[test]
    fn speed_boost_is_leaf_true() {
        let tc = TriggerChain::SpeedBoost {
            target: Target::Bolt,
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
                target: Target::Bolt,
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

    // ======================================================================
    // B4: ChipTemplate + RaritySlot deserialization (spec behaviors 1-3)
    // ======================================================================

    // --- Behavior 1: ChipTemplate deserializes with all four rarity slots ---

    #[test]
    fn chip_template_deserializes_with_all_rarity_slots() {
        let ron_str = r#"(name: "Piercing", max_taken: 3, common: Some((prefix: "Basic", effects: [OnSelected([Piercing(1)])])), uncommon: Some((prefix: "Keen", effects: [OnSelected([Piercing(2)])])), rare: Some((prefix: "Brutal", effects: [OnSelected([Piercing(3), DamageBoost(0.1)])])), legendary: None)"#;
        let template: ChipTemplate =
            ron::de::from_str(ron_str).expect("should parse ChipTemplate with all slots");
        assert_eq!(template.name, "Piercing");
        assert_eq!(template.max_taken, 3);
        assert!(template.common.is_some());
        assert!(template.uncommon.is_some());
        assert!(template.rare.is_some());
        assert!(template.legendary.is_none());
    }

    #[test]
    fn chip_template_deserializes_with_all_none_slots() {
        let ron_str = r#"(name: "Empty", max_taken: 1, common: None, uncommon: None, rare: None, legendary: None)"#;
        let template: ChipTemplate =
            ron::de::from_str(ron_str).expect("should parse ChipTemplate with all None slots");
        assert_eq!(template.name, "Empty");
        assert_eq!(template.max_taken, 1);
        assert!(template.common.is_none());
        assert!(template.uncommon.is_none());
        assert!(template.rare.is_none());
        assert!(template.legendary.is_none());
    }

    #[test]
    fn chip_template_max_taken_zero_is_valid_ron() {
        let ron_str = r#"(name: "Zero", max_taken: 0, common: Some((prefix: "X", effects: [])), uncommon: None, rare: None, legendary: None)"#;
        let template: ChipTemplate =
            ron::de::from_str(ron_str).expect("should parse ChipTemplate with max_taken 0");
        assert_eq!(template.max_taken, 0);
    }

    // --- Behavior 2: ChipTemplate with only legendary slot ---

    #[test]
    fn chip_template_deserializes_legendary_only() {
        let ron_str = r#"(name: "Glass Cannon", max_taken: 1, common: None, uncommon: None, rare: None, legendary: Some((prefix: "", effects: [OnSelected([DamageBoost(1.0), SizeBoost(Breaker, -0.3)])])))"#;
        let template: ChipTemplate =
            ron::de::from_str(ron_str).expect("should parse legendary-only ChipTemplate");
        assert_eq!(template.name, "Glass Cannon");
        assert_eq!(template.max_taken, 1);
        assert!(template.common.is_none());
        assert!(template.uncommon.is_none());
        assert!(template.rare.is_none());
        assert!(template.legendary.is_some());
        let slot = template.legendary.unwrap();
        assert_eq!(slot.prefix, "");
    }

    // --- Behavior 3: RaritySlot stores prefix and effects ---

    #[test]
    fn rarity_slot_deserializes_prefix_and_effects() {
        let ron_str = r#"(prefix: "Basic", effects: [OnSelected([Piercing(1)])])"#;
        let slot: RaritySlot = ron::de::from_str(ron_str).expect("should parse RaritySlot");
        assert_eq!(slot.prefix, "Basic");
        assert_eq!(
            slot.effects,
            vec![TriggerChain::OnSelected(vec![TriggerChain::Piercing(1)])]
        );
    }

    // ======================================================================
    // B4: expand_template pure function (spec behaviors 4-9)
    // ======================================================================

    /// Helper: build a `ChipTemplate` with specific slots for testing.
    fn make_template(
        name: &str,
        max_taken: u32,
        common: Option<RaritySlot>,
        uncommon: Option<RaritySlot>,
        rare: Option<RaritySlot>,
        legendary: Option<RaritySlot>,
    ) -> ChipTemplate {
        ChipTemplate {
            name: name.to_owned(),
            max_taken,
            common,
            uncommon,
            rare,
            legendary,
        }
    }

    fn slot(prefix: &str, effects: Vec<TriggerChain>) -> RaritySlot {
        RaritySlot {
            prefix: prefix.to_owned(),
            effects,
        }
    }

    // --- Behavior 4: expand_template produces one ChipDefinition per non-None slot ---

    #[test]
    fn expand_template_produces_one_def_per_non_none_slot() {
        let template = make_template(
            "Piercing",
            3,
            Some(slot("Basic", vec![TriggerChain::Piercing(1)])),
            Some(slot("Keen", vec![TriggerChain::Piercing(2)])),
            None,
            None,
        );
        let defs = expand_template(&template);
        assert_eq!(defs.len(), 2);
    }

    #[test]
    fn expand_template_all_none_returns_empty() {
        let template = make_template("Empty", 1, None, None, None, None);
        let defs = expand_template(&template);
        assert!(defs.is_empty());
    }

    // --- Behavior 5: Expanded name is "{prefix} {template_name}" ---

    #[test]
    fn expanded_chip_name_is_prefix_space_template_name() {
        let template = make_template(
            "Piercing",
            3,
            Some(slot(
                "Basic",
                vec![TriggerChain::OnSelected(vec![TriggerChain::Piercing(1)])],
            )),
            None,
            None,
            None,
        );
        let defs = expand_template(&template);
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "Basic Piercing");
        assert_eq!(defs[0].rarity, Rarity::Common);
        assert_eq!(defs[0].max_stacks, 3);
        assert_eq!(
            defs[0].effects,
            vec![TriggerChain::OnSelected(vec![TriggerChain::Piercing(1)])]
        );
        assert_eq!(defs[0].template_name, Some("Piercing".to_owned()));
        assert_eq!(defs[0].description, "");
    }

    // --- Behavior 6: Empty prefix uses template name directly ---

    #[test]
    fn expanded_chip_empty_prefix_uses_template_name() {
        let template = make_template(
            "Glass Cannon",
            1,
            None,
            None,
            None,
            Some(slot("", vec![TriggerChain::DamageBoost(1.0)])),
        );
        let defs = expand_template(&template);
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "Glass Cannon");
        assert_eq!(defs[0].rarity, Rarity::Legendary);
        assert_eq!(defs[0].template_name, Some("Glass Cannon".to_owned()));
    }

    #[test]
    fn expanded_chip_whitespace_prefix_uses_template_name() {
        let template = make_template(
            "Glass Cannon",
            1,
            None,
            None,
            None,
            Some(slot("  ", vec![TriggerChain::DamageBoost(1.0)])),
        );
        let defs = expand_template(&template);
        assert_eq!(defs.len(), 1);
        assert_eq!(
            defs[0].name, "Glass Cannon",
            "whitespace-only prefix should be treated as empty"
        );
    }

    // --- Behavior 7: Each expanded definition gets the correct rarity ---

    #[test]
    fn expanded_defs_have_correct_rarities() {
        let template = make_template(
            "AllSlots",
            5,
            Some(slot("C", vec![TriggerChain::Piercing(1)])),
            Some(slot("U", vec![TriggerChain::Piercing(2)])),
            Some(slot("R", vec![TriggerChain::Piercing(3)])),
            Some(slot("L", vec![TriggerChain::Piercing(4)])),
        );
        let defs = expand_template(&template);
        assert_eq!(defs.len(), 4);
        assert_eq!(defs[0].rarity, Rarity::Common);
        assert_eq!(defs[1].rarity, Rarity::Uncommon);
        assert_eq!(defs[2].rarity, Rarity::Rare);
        assert_eq!(defs[3].rarity, Rarity::Legendary);
    }

    // --- Behavior 8: All expanded definitions share max_stacks ---

    #[test]
    fn expanded_defs_share_max_stacks_from_template() {
        let template = make_template(
            "Piercing",
            3,
            Some(slot("Basic", vec![TriggerChain::Piercing(1)])),
            Some(slot("Keen", vec![TriggerChain::Piercing(2)])),
            Some(slot("Brutal", vec![TriggerChain::Piercing(3)])),
            None,
        );
        let defs = expand_template(&template);
        assert_eq!(defs.len(), 3);
        for def in &defs {
            assert_eq!(
                def.max_stacks, 3,
                "all expanded defs should share max_stacks=3, got {} for {}",
                def.max_stacks, def.name
            );
        }
    }

    // --- Behavior 9: All expanded definitions share the same template_name ---

    #[test]
    fn expanded_defs_share_template_name() {
        let template = make_template(
            "Piercing",
            3,
            Some(slot("Basic", vec![TriggerChain::Piercing(1)])),
            Some(slot("Keen", vec![TriggerChain::Piercing(2)])),
            Some(slot("Brutal", vec![TriggerChain::Piercing(3)])),
            None,
        );
        let defs = expand_template(&template);
        assert_eq!(defs.len(), 3);
        for def in &defs {
            assert_eq!(
                def.template_name,
                Some("Piercing".to_owned()),
                "all expanded defs should have template_name 'Piercing', got {:?} for {}",
                def.template_name,
                def.name
            );
        }
    }

    // =========================================================================
    // B12b: EffectNode type construction for chip effect patterns (behaviors 19-20)
    // These tests verify the EffectNode shapes that ChipDefinition.effects
    // will hold after migration. They exercise evaluate_node which fails
    // with todo!().
    // =========================================================================

    #[test]
    fn effect_node_surge_chip_pattern_evaluates_correctly() {
        use crate::effect::{
            definition::{Effect, EffectNode, Trigger},
            evaluate::{NodeEvalResult, TriggerKind, evaluate_node},
        };

        // After migration: ChipDefinition.effects[0] will be this EffectNode
        let node = EffectNode::Trigger(
            Trigger::OnPerfectBump,
            vec![EffectNode::Trigger(
                Trigger::OnImpact(crate::effect::definition::ImpactTarget::Cell),
                vec![EffectNode::Leaf(Effect::Shockwave {
                    base_range: 64.0,
                    range_per_level: 0.0,
                    stacks: 1,
                    speed: 400.0,
                })],
            )],
        );
        let result = evaluate_node(TriggerKind::PerfectBump, &node);
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], NodeEvalResult::Arm(_)));
    }

    #[test]
    fn effect_node_passive_chip_pattern_on_selected_no_match() {
        use crate::effect::{
            definition::{Effect, EffectNode, Trigger},
            evaluate::{NodeEvalResult, TriggerKind, evaluate_node},
        };

        // After migration: passive ChipDefinition.effects[0] will be:
        // EffectNode::Trigger(Trigger::OnSelected, [Leaf(Piercing(1))])
        let node = EffectNode::Trigger(
            Trigger::OnSelected,
            vec![EffectNode::Leaf(Effect::Piercing(1))],
        );
        // OnSelected has no TriggerKind mapping — should always return NoMatch
        let result = evaluate_node(TriggerKind::PerfectBump, &node);
        assert_eq!(result, vec![NodeEvalResult::NoMatch]);
    }

    #[test]
    fn effect_node_ron_deserialization_for_chip_definition() {
        use crate::effect::{
            definition::{Effect, EffectNode, Trigger},
            evaluate::{NodeEvalResult, TriggerKind, evaluate_node},
        };

        // After migration, ChipDefinition RON will use EffectNode syntax.
        let ron_str = "Trigger(OnSelected, [Leaf(Piercing(1))])";
        let node: EffectNode =
            ron::de::from_str(ron_str).expect("EffectNode for chip RON should parse");
        assert_eq!(
            node,
            EffectNode::Trigger(
                Trigger::OnSelected,
                vec![EffectNode::Leaf(Effect::Piercing(1))]
            )
        );
        // Verify evaluate_node behavior for OnSelected (fails with todo!)
        let result = evaluate_node(TriggerKind::PerfectBump, &node);
        assert_eq!(result, vec![NodeEvalResult::NoMatch]);
    }

    // B12b: ChipEffectApplied will carry Effect (behavior 18)

    #[test]
    fn effect_piercing_matches_and_evaluates_correctly() {
        use crate::effect::{
            definition::{Effect, EffectNode, Trigger},
            evaluate::{NodeEvalResult, TriggerKind, evaluate_node},
        };

        let effect = Effect::Piercing(1);
        let tc = TriggerChain::Piercing(1);
        // Both should carry the same value
        match (&effect, &tc) {
            (Effect::Piercing(e), TriggerChain::Piercing(t)) => assert_eq!(e, t),
            _ => panic!("variant mismatch"),
        }
        // Verify evaluate_node with a Piercing leaf (fails with todo!)
        let node = EffectNode::trigger_leaf(Trigger::OnBump, effect.clone());
        let result = evaluate_node(TriggerKind::BumpSuccess, &node);
        assert_eq!(result, vec![NodeEvalResult::Fire(effect)]);
    }

    #[test]
    fn effect_damage_boost_matches_and_evaluates_correctly() {
        use crate::effect::{
            definition::{Effect, EffectNode, Trigger},
            evaluate::{NodeEvalResult, TriggerKind, evaluate_node},
        };

        let effect = Effect::DamageBoost(0.5);
        let tc = TriggerChain::DamageBoost(0.5);
        match (&effect, &tc) {
            (Effect::DamageBoost(e), TriggerChain::DamageBoost(t)) => {
                assert!((e - t).abs() < f32::EPSILON);
            }
            _ => panic!("variant mismatch"),
        }
        // Verify evaluate_node with DamageBoost leaf (fails with todo!)
        let node = EffectNode::trigger_leaf(Trigger::OnBump, effect.clone());
        let result = evaluate_node(TriggerKind::BumpSuccess, &node);
        assert_eq!(result, vec![NodeEvalResult::Fire(effect)]);
    }

    // B12b: EffectNode RON for triggered chip

    #[test]
    fn effect_node_ron_triggered_chip_format() {
        use crate::effect::{
            definition::{Effect, EffectNode, Trigger},
            evaluate::{NodeEvalResult, TriggerKind, evaluate_node},
        };

        let ron_str = "Trigger(OnPerfectBump, [Trigger(OnImpact(Cell), [Leaf(Shockwave(base_range: 64.0, range_per_level: 32.0, stacks: 1, speed: 400.0))])])";
        let node: EffectNode =
            ron::de::from_str(ron_str).expect("triggered chip EffectNode RON should parse");
        assert_eq!(
            node,
            EffectNode::Trigger(
                Trigger::OnPerfectBump,
                vec![EffectNode::Trigger(
                    Trigger::OnImpact(crate::effect::definition::ImpactTarget::Cell),
                    vec![EffectNode::Leaf(Effect::Shockwave {
                        base_range: 64.0,
                        range_per_level: 32.0,
                        stacks: 1,
                        speed: 400.0,
                    })],
                )]
            )
        );
        // Verify evaluate_node arms the inner trigger (fails with todo!)
        let result = evaluate_node(TriggerKind::PerfectBump, &node);
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], NodeEvalResult::Arm(_)));
    }

    // ======================================================================
    // C2-C4: Augment RON template tests
    // ======================================================================

    #[test]
    fn augment_chip_template_ron_parses() {
        let ron_str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/chips/augment.chip.ron"
        ));
        let template: ChipTemplate =
            ron::de::from_str(ron_str).expect("augment chip template RON should parse");
        assert_eq!(template.name, "Augment");
        assert_eq!(template.max_taken, 2);
        assert!(template.common.is_some());
        assert!(template.uncommon.is_some());
        assert!(template.rare.is_some());
        assert!(template.legendary.is_none());

        let defs = expand_template(&template);
        assert_eq!(defs.len(), 3, "Augment should expand to 3 definitions");
        assert_eq!(defs[0].name, "Basic Augment");
        assert_eq!(defs[0].rarity, Rarity::Common);
        assert_eq!(defs[1].name, "Sturdy Augment");
        assert_eq!(defs[1].rarity, Rarity::Uncommon);
        assert_eq!(defs[2].name, "Fortified Augment");
        assert_eq!(defs[2].rarity, Rarity::Rare);
        for def in &defs {
            assert_eq!(def.max_stacks, 2);
        }
    }

    #[test]
    fn augment_rare_has_three_effects() {
        let ron_str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/chips/augment.chip.ron"
        ));
        let template: ChipTemplate =
            ron::de::from_str(ron_str).expect("augment chip template RON should parse");
        let rare_slot = template.rare.as_ref().expect("rare slot should exist");
        assert_eq!(rare_slot.prefix, "Fortified");

        // The rare slot has OnSelected with 3 inner effects
        assert_eq!(rare_slot.effects.len(), 1, "should have 1 top-level effect");
        match &rare_slot.effects[0] {
            TriggerChain::OnSelected(inner) => {
                assert_eq!(inner.len(), 3, "rare OnSelected should contain 3 effects");
                assert_eq!(inner[0], TriggerChain::SizeBoost(Target::Breaker, 16.0));
                assert_eq!(inner[1], TriggerChain::BumpForce(25.0));
                assert_eq!(
                    inner[2],
                    TriggerChain::SpeedBoost {
                        target: Target::Breaker,
                        multiplier: 1.15,
                    }
                );
            }
            other => panic!("expected OnSelected, got {other:?}"),
        }
    }

    // ======================================================================
    // C2-C4: RampingDamage TriggerChain leaf tests (Amp)
    // ======================================================================

    #[test]
    fn ramping_damage_trigger_chain_deserializes() {
        let tc: TriggerChain =
            ron::de::from_str("RampingDamage(bonus_per_hit: 0.02, max_bonus: 0.2)")
                .expect("should parse RampingDamage");
        assert_eq!(
            tc,
            TriggerChain::RampingDamage {
                bonus_per_hit: 0.02,
                max_bonus: 0.2,
            }
        );
    }

    #[test]
    fn ramping_damage_trigger_chain_deserializes_zero_values() {
        let tc: TriggerChain =
            ron::de::from_str("RampingDamage(bonus_per_hit: 0.0, max_bonus: 0.0)")
                .expect("should parse RampingDamage with zero values");
        assert_eq!(
            tc,
            TriggerChain::RampingDamage {
                bonus_per_hit: 0.0,
                max_bonus: 0.0,
            }
        );
    }

    #[test]
    fn ramping_damage_is_leaf_with_depth_zero() {
        let tc = TriggerChain::RampingDamage {
            bonus_per_hit: 0.04,
            max_bonus: 0.4,
        };
        assert_eq!(tc.depth(), 0);
        assert!(tc.is_leaf());
    }

    #[test]
    fn amp_chip_template_ron_parses() {
        let ron_str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/chips/amp.chip.ron"
        ));
        let template: ChipTemplate =
            ron::de::from_str(ron_str).expect("amp chip template RON should parse");
        assert_eq!(template.name, "Amp");
        assert_eq!(template.max_taken, 2);
        assert!(template.common.is_some());
        let common = template.common.as_ref().unwrap();
        assert_eq!(common.prefix, "Basic");
        assert_eq!(
            common.effects,
            vec![TriggerChain::OnSelected(vec![
                TriggerChain::RampingDamage {
                    bonus_per_hit: 0.02,
                    max_bonus: 0.2,
                }
            ])]
        );

        let defs = expand_template(&template);
        assert_eq!(defs.len(), 3, "Amp should expand to 3 definitions");
        assert_eq!(defs[0].name, "Basic Amp");
        assert_eq!(defs[0].rarity, Rarity::Common);
        assert_eq!(defs[1].name, "Potent Amp");
        assert_eq!(defs[1].rarity, Rarity::Uncommon);
        assert_eq!(defs[2].name, "Savage Amp");
        assert_eq!(defs[2].rarity, Rarity::Rare);
        for def in &defs {
            assert_eq!(def.max_stacks, 2);
        }
    }

    // ======================================================================
    // C2-C4: TimedSpeedBurst TriggerChain leaf tests (Overclock)
    // ======================================================================

    #[test]
    fn timed_speed_burst_trigger_chain_deserializes() {
        let tc: TriggerChain =
            ron::de::from_str("TimedSpeedBurst(speed_mult: 1.3, duration_secs: 2.0)")
                .expect("should parse TimedSpeedBurst");
        assert_eq!(
            tc,
            TriggerChain::TimedSpeedBurst {
                speed_mult: 1.3,
                duration_secs: 2.0,
            }
        );
    }

    #[test]
    fn timed_speed_burst_trigger_chain_edge_values() {
        // Identity multiplier and zero duration should deserialize
        let tc: TriggerChain =
            ron::de::from_str("TimedSpeedBurst(speed_mult: 1.0, duration_secs: 0.0)")
                .expect("should parse TimedSpeedBurst with edge values");
        assert_eq!(
            tc,
            TriggerChain::TimedSpeedBurst {
                speed_mult: 1.0,
                duration_secs: 0.0,
            }
        );
    }

    #[test]
    fn timed_speed_burst_is_leaf_with_depth_zero() {
        let tc = TriggerChain::TimedSpeedBurst {
            speed_mult: 1.5,
            duration_secs: 3.0,
        };
        assert_eq!(tc.depth(), 0);
        assert!(tc.is_leaf());
    }

    #[test]
    fn overclock_chip_template_ron_parses() {
        let ron_str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/chips/overclock.chip.ron"
        ));
        let template: ChipTemplate =
            ron::de::from_str(ron_str).expect("overclock chip template RON should parse");
        assert_eq!(template.name, "Overclock");
        assert_eq!(template.max_taken, 2);
        assert!(template.common.is_some());
        let common = template.common.as_ref().unwrap();
        assert_eq!(common.prefix, "Basic");
        assert_eq!(
            common.effects,
            vec![TriggerChain::OnPerfectBump(vec![
                TriggerChain::TimedSpeedBurst {
                    speed_mult: 1.3,
                    duration_secs: 2.0,
                }
            ])]
        );

        let defs = expand_template(&template);
        assert_eq!(defs.len(), 3, "Overclock should expand to 3 definitions");
        assert_eq!(defs[0].name, "Basic Overclock");
        assert_eq!(defs[0].rarity, Rarity::Common);
        assert_eq!(defs[1].name, "Charged Overclock");
        assert_eq!(defs[1].rarity, Rarity::Uncommon);
        assert_eq!(defs[2].name, "Supercharged Overclock");
        assert_eq!(defs[2].rarity, Rarity::Rare);
        for def in &defs {
            assert_eq!(def.max_stacks, 2);
        }
    }

    // ======================================================================
    // C5-C6: TimePressureBoost (Deadline) TriggerChain tests
    // ======================================================================

    #[test]
    fn time_pressure_boost_trigger_chain_deserializes() {
        let tc: TriggerChain =
            ron::de::from_str("TimePressureBoost(speed_mult: 2.0, threshold_pct: 0.25)")
                .expect("should parse TimePressureBoost");
        assert_eq!(
            tc,
            TriggerChain::TimePressureBoost {
                speed_mult: 2.0,
                threshold_pct: 0.25,
            }
        );
    }

    #[test]
    fn time_pressure_boost_trigger_chain_zero_threshold_edge_case() {
        let tc: TriggerChain =
            ron::de::from_str("TimePressureBoost(speed_mult: 2.0, threshold_pct: 0.0)")
                .expect("should parse TimePressureBoost with threshold_pct: 0.0");
        assert_eq!(
            tc,
            TriggerChain::TimePressureBoost {
                speed_mult: 2.0,
                threshold_pct: 0.0,
            }
        );
    }

    #[test]
    fn time_pressure_boost_is_leaf_with_depth_zero() {
        let tc = TriggerChain::TimePressureBoost {
            speed_mult: 2.0,
            threshold_pct: 0.25,
        };
        assert!(tc.is_leaf(), "TimePressureBoost should be a leaf");
        assert_eq!(tc.depth(), 0, "TimePressureBoost depth should be 0");
    }

    // ======================================================================
    // C5-C6: RandomEffect (Flux) TriggerChain tests
    // ======================================================================

    #[test]
    fn random_effect_trigger_chain_deserializes() {
        let ron_str = "RandomEffect([(0.5, SpeedBoost(target: Bolt, multiplier: 1.1)), (0.5, Shockwave(base_range: 24.0, range_per_level: 0.0, stacks: 1, speed: 400.0))])";
        let tc: TriggerChain =
            ron::de::from_str(ron_str).expect("should parse RandomEffect with pool entries");
        match &tc {
            TriggerChain::RandomEffect(pool) => {
                assert_eq!(pool.len(), 2, "pool should contain 2 entries");
                assert!((pool[0].0 - 0.5).abs() < f32::EPSILON);
                assert!((pool[1].0 - 0.5).abs() < f32::EPSILON);
            }
            other => panic!("expected RandomEffect, got {other:?}"),
        }
    }

    #[test]
    fn random_effect_trigger_chain_single_entry_deserializes() {
        let ron_str = "RandomEffect([(1.0, SpawnBolt)])";
        let tc: TriggerChain =
            ron::de::from_str(ron_str).expect("should parse RandomEffect with single entry");
        match &tc {
            TriggerChain::RandomEffect(pool) => {
                assert_eq!(pool.len(), 1);
                assert!((pool[0].0 - 1.0).abs() < f32::EPSILON);
                assert_eq!(pool[0].1, TriggerChain::SpawnBolt);
            }
            other => panic!("expected RandomEffect, got {other:?}"),
        }
    }

    #[test]
    fn random_effect_is_leaf_with_depth_zero() {
        let tc = TriggerChain::RandomEffect(vec![(1.0, TriggerChain::SpawnBolt)]);
        assert!(tc.is_leaf(), "RandomEffect should be a leaf");
        assert_eq!(tc.depth(), 0, "RandomEffect depth should be 0");
    }

    #[test]
    fn random_effect_with_non_leaf_inner_is_still_leaf() {
        let tc = TriggerChain::RandomEffect(vec![(
            1.0,
            TriggerChain::OnImpact(ImpactTarget::Cell, vec![TriggerChain::test_shockwave(32.0)]),
        )]);
        assert!(
            tc.is_leaf(),
            "RandomEffect wrapper is a leaf; inner chains may be trigger wrappers for arming"
        );
    }

    // ======================================================================
    // C5-C6: EntropyEngine TriggerChain tests
    // ======================================================================

    #[test]
    fn entropy_engine_trigger_chain_deserializes() {
        let ron_str = "EntropyEngine(5, [(0.3, SpawnBolt), (0.7, SpeedBoost(target: Bolt, multiplier: 1.3))])";
        let tc: TriggerChain = ron::de::from_str(ron_str).expect("should parse EntropyEngine");
        match &tc {
            TriggerChain::EntropyEngine(threshold, pool) => {
                assert_eq!(*threshold, 5);
                assert_eq!(pool.len(), 2);
                assert!((pool[0].0 - 0.3).abs() < f32::EPSILON);
                assert!((pool[1].0 - 0.7).abs() < f32::EPSILON);
            }
            other => panic!("expected EntropyEngine, got {other:?}"),
        }
    }

    #[test]
    fn entropy_engine_trigger_chain_threshold_one_edge_case() {
        let ron_str = "EntropyEngine(1, [(1.0, SpawnBolt)])";
        let tc: TriggerChain =
            ron::de::from_str(ron_str).expect("should parse EntropyEngine with threshold 1");
        match &tc {
            TriggerChain::EntropyEngine(threshold, pool) => {
                assert_eq!(*threshold, 1);
                assert_eq!(pool.len(), 1);
            }
            other => panic!("expected EntropyEngine, got {other:?}"),
        }
    }

    #[test]
    fn entropy_engine_is_leaf_with_depth_zero() {
        let tc = TriggerChain::EntropyEngine(5, vec![(1.0, TriggerChain::SpawnBolt)]);
        assert!(tc.is_leaf(), "EntropyEngine should be a leaf");
        assert_eq!(tc.depth(), 0, "EntropyEngine depth should be 0");
    }
}
