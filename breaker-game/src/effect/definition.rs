//! Archetype and effect type definitions — RON-deserialized data structures.

use bevy::prelude::*;
use serde::Deserialize;

use crate::chips::definition::TriggerChain;

// ---------------------------------------------------------------------------
// New split types (B12b stubs — production logic added by writer-code)
// ---------------------------------------------------------------------------

/// Discriminates which entity an effect targets.
///
/// Moved from `chips/definition.rs` to canonical location in the effect domain.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// Target the specific bolt that triggered the effect.
    Bolt,
    /// Target the breaker entity.
    Breaker,
    /// Target all bolt entities in play.
    AllBolts,
}

/// Discriminates which surface triggered an impact event.
///
/// Moved from `chips/definition.rs` to canonical location in the effect domain.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImpactTarget {
    /// Bolt hit a cell.
    Cell,
    /// Bolt bounced off the breaker.
    Breaker,
    /// Bolt bounced off a wall.
    Wall,
}

/// The trigger condition that gates an effect subtree.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trigger {
    /// Fires on a perfect bump.
    OnPerfectBump,
    /// Fires on any non-whiff bump (Early, Late, or Perfect).
    OnBump,
    /// Fires on an early bump.
    OnEarlyBump,
    /// Fires on a late bump.
    OnLateBump,
    /// Fires when a bump whiffs (misses).
    OnBumpWhiff,
    /// Fires on bolt impact with a specific surface.
    OnImpact(ImpactTarget),
    /// Fires when a cell is destroyed.
    OnCellDestroyed,
    /// Fires when a bolt is lost.
    OnBoltLost,
    /// Passive effects: evaluated immediately on chip selection.
    OnSelected,
}

/// A leaf effect action — the terminal node in an effect tree.
///
/// Does NOT derive `Copy` to allow future addition of non-Copy fields.
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub enum Effect {
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
    /// Bolt passes through N cells before stopping.
    Piercing(u32),
    /// Adds fractional bonus damage per stack.
    DamageBoost(f32),
    /// Scales a target's speed by a multiplier, clamped within base/max bounds.
    SpeedBoost {
        /// Which entity to apply the speed change to.
        target: Target,
        /// Multiplier applied to the current velocity magnitude.
        multiplier: f32,
    },
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
    /// Spawns a tethered chain bolt at the anchor bolt's position.
    ChainBolt {
        /// Maximum distance the chain bolt can travel from its anchor.
        tether_distance: f32,
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
}

/// A node in the effect tree — either a trigger gate wrapping children,
/// or a terminal leaf action.
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub enum EffectNode {
    /// A trigger condition with child nodes evaluated when the trigger fires.
    Trigger(Trigger, Vec<EffectNode>),
    /// A terminal effect action.
    Leaf(Effect),
}

/// Marker component for entities that can have effects armed on them.
///
/// Added to `Bolt` and `Breaker` entities via `#[require(EffectTarget)]`.
/// `ArmedEffects` queries filter `With<EffectTarget>`.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct EffectTarget;

// ---------------------------------------------------------------------------
// Test constructors for Effect
// ---------------------------------------------------------------------------

#[cfg(test)]
impl Effect {
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
}

// ---------------------------------------------------------------------------
// Test constructors for EffectNode
// ---------------------------------------------------------------------------

#[cfg(test)]
impl EffectNode {
    /// Build a trigger-wrapped single-leaf node.
    pub(crate) fn trigger_leaf(trigger: Trigger, effect: Effect) -> Self {
        Self::Trigger(trigger, vec![Self::Leaf(effect)])
    }
}

// ---------------------------------------------------------------------------
// Archetype definition (still uses TriggerChain — writer-code will migrate)
// ---------------------------------------------------------------------------

/// A breaker archetype definition loaded from a RON file.
///
/// Uses unified `TriggerChain` for all behavior bindings.
/// Adding a new archetype = new RON file. Adding a new behavior type =
/// new `TriggerChain` variant + handler.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub(crate) struct ArchetypeDefinition {
    /// Display name of the archetype.
    pub name: String,
    /// Optional stat overrides applied on top of `BreakerDefaults`.
    pub stat_overrides: BreakerStatOverrides,
    /// Number of lives, if the archetype uses a life pool.
    pub life_pool: Option<u32>,
    /// Chain fired when a bolt is lost.
    pub on_bolt_lost: Option<TriggerChain>,
    /// Chain fired on a perfect bump.
    pub on_perfect_bump: Option<TriggerChain>,
    /// Chain fired on an early bump.
    pub on_early_bump: Option<TriggerChain>,
    /// Chain fired on a late bump.
    pub on_late_bump: Option<TriggerChain>,
    /// Additional trigger chains (overclock-style multi-step chains).
    pub chains: Vec<TriggerChain>,
}

/// Optional overrides for `BreakerDefaults` fields.
///
/// Each `Some` field replaces the corresponding base value.
#[derive(Deserialize, Clone, Debug, Default)]
pub(crate) struct BreakerStatOverrides {
    /// Override breaker width.
    pub width: Option<f32>,
    /// Override breaker height.
    pub height: Option<f32>,
    /// Override maximum horizontal speed.
    pub max_speed: Option<f32>,
    /// Override horizontal acceleration.
    pub acceleration: Option<f32>,
    /// Override horizontal deceleration.
    pub deceleration: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // B12b: EffectNode, Trigger, and Effect type construction (behaviors 1-6)
    // =========================================================================

    #[test]
    fn effect_node_leaf_wraps_spawn_bolt() {
        let node = EffectNode::Leaf(Effect::SpawnBolt);
        assert!(matches!(node, EffectNode::Leaf(Effect::SpawnBolt)));
    }

    #[test]
    fn effect_node_leaf_wraps_lose_life() {
        let node = EffectNode::Leaf(Effect::LoseLife);
        assert!(matches!(node, EffectNode::Leaf(Effect::LoseLife)));
    }

    #[test]
    fn effect_node_trigger_wraps_trigger_and_children() {
        let node = EffectNode::Trigger(
            Trigger::OnPerfectBump,
            vec![EffectNode::Leaf(Effect::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })],
        );
        match &node {
            EffectNode::Trigger(Trigger::OnPerfectBump, children) => {
                assert_eq!(children.len(), 1);
            }
            other => panic!("expected Trigger(OnPerfectBump, _), got {other:?}"),
        }
    }

    #[test]
    fn effect_node_trigger_empty_children_is_valid() {
        let node = EffectNode::Trigger(Trigger::OnBump, vec![]);
        match &node {
            EffectNode::Trigger(Trigger::OnBump, children) => {
                assert!(children.is_empty());
            }
            other => panic!("expected Trigger(OnBump, []), got {other:?}"),
        }
    }

    #[test]
    fn effect_node_nests_triggers_two_deep() {
        let node = EffectNode::Trigger(
            Trigger::OnPerfectBump,
            vec![EffectNode::Trigger(
                Trigger::OnImpact(ImpactTarget::Cell),
                vec![EffectNode::Leaf(Effect::test_shockwave(64.0))],
            )],
        );
        match &node {
            EffectNode::Trigger(Trigger::OnPerfectBump, children) => match &children[0] {
                EffectNode::Trigger(Trigger::OnImpact(ImpactTarget::Cell), inner) => {
                    assert!(matches!(
                        inner[0],
                        EffectNode::Leaf(Effect::Shockwave { .. })
                    ));
                }
                other => panic!("expected inner Trigger(OnImpact(Cell), _), got {other:?}"),
            },
            other => panic!("expected outer Trigger(OnPerfectBump, _), got {other:?}"),
        }
    }

    #[test]
    fn effect_node_nests_three_deep() {
        let node = EffectNode::Trigger(
            Trigger::OnPerfectBump,
            vec![EffectNode::Trigger(
                Trigger::OnImpact(ImpactTarget::Cell),
                vec![EffectNode::Trigger(
                    Trigger::OnCellDestroyed,
                    vec![EffectNode::Leaf(Effect::test_shockwave(64.0))],
                )],
            )],
        );
        // Just assert it constructs without panic — 3-deep nesting is valid
        assert!(matches!(
            node,
            EffectNode::Trigger(Trigger::OnPerfectBump, _)
        ));
    }

    #[test]
    fn trigger_enum_has_all_nine_variants() {
        // Construct all 9 Trigger variants (11 distinguishable patterns
        // including ImpactTarget discrimination)
        let triggers = [
            Trigger::OnPerfectBump,
            Trigger::OnBump,
            Trigger::OnEarlyBump,
            Trigger::OnLateBump,
            Trigger::OnBumpWhiff,
            Trigger::OnImpact(ImpactTarget::Cell),
            Trigger::OnImpact(ImpactTarget::Breaker),
            Trigger::OnImpact(ImpactTarget::Wall),
            Trigger::OnCellDestroyed,
            Trigger::OnBoltLost,
            Trigger::OnSelected,
        ];
        assert_eq!(
            triggers.len(),
            11,
            "all 11 distinguishable trigger patterns"
        );
        // Verify OnSelected is distinct from runtime triggers
        assert_ne!(Trigger::OnSelected, Trigger::OnPerfectBump);
    }

    #[test]
    fn effect_enum_has_all_twenty_variants() {
        let effects: Vec<Effect> = vec![
            Effect::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            },
            Effect::Piercing(1),
            Effect::DamageBoost(0.5),
            Effect::SpeedBoost {
                target: Target::Bolt,
                multiplier: 1.2,
            },
            Effect::ChainHit(2),
            Effect::SizeBoost(Target::Bolt, 5.0),
            Effect::Attraction(0.3),
            Effect::BumpForce(0.2),
            Effect::TiltControl(0.1),
            Effect::ChainBolt {
                tether_distance: 150.0,
            },
            Effect::MultiBolt {
                base_count: 2,
                count_per_level: 0,
                stacks: 1,
            },
            Effect::Shield {
                base_duration: 3.0,
                duration_per_level: 0.0,
                stacks: 1,
            },
            Effect::LoseLife,
            Effect::TimePenalty { seconds: 3.0 },
            Effect::SpawnBolt,
            Effect::ChainLightning {
                arcs: 3,
                range: 64.0,
                damage_mult: 0.5,
            },
            Effect::SpawnPhantom {
                duration: 5.0,
                max_active: 2,
            },
            Effect::PiercingBeam {
                damage_mult: 1.5,
                width: 20.0,
            },
            Effect::GravityWell {
                strength: 1.0,
                duration: 5.0,
                radius: 100.0,
                max: 2,
            },
            Effect::SecondWind { invuln_secs: 3.0 },
        ];
        assert_eq!(effects.len(), 20, "all 20 Effect variants");
    }

    #[test]
    fn effect_zero_damage_boost_is_valid() {
        let e = Effect::DamageBoost(0.0);
        assert_eq!(e, Effect::DamageBoost(0.0));
    }

    #[test]
    fn effect_speed_boost_all_bolts_target() {
        let e = Effect::SpeedBoost {
            target: Target::AllBolts,
            multiplier: 0.5,
        };
        assert!(matches!(
            e,
            Effect::SpeedBoost {
                target: Target::AllBolts,
                ..
            }
        ));
    }

    #[test]
    fn effect_target_marker_is_unit_struct() {
        let _ = EffectTarget;
        // EffectTarget is a unit struct marker — no data fields
    }

    // =========================================================================
    // B12b: RON deserialization of EffectNode (behaviors 7-9)
    // =========================================================================

    #[test]
    fn effect_node_ron_trigger_wrapping_leaf() {
        let ron_str = "Trigger(OnPerfectBump, [Leaf(Shockwave(base_range: 64.0, range_per_level: 0.0, stacks: 1, speed: 400.0))])";
        let node: EffectNode =
            ron::de::from_str(ron_str).expect("EffectNode RON trigger wrapping leaf should parse");
        assert_eq!(
            node,
            EffectNode::Trigger(
                Trigger::OnPerfectBump,
                vec![EffectNode::Leaf(Effect::Shockwave {
                    base_range: 64.0,
                    range_per_level: 0.0,
                    stacks: 1,
                    speed: 400.0,
                })]
            )
        );
    }

    #[test]
    fn effect_node_ron_bare_leaf() {
        let ron_str = "Leaf(SpawnBolt)";
        let node: EffectNode =
            ron::de::from_str(ron_str).expect("bare Leaf(SpawnBolt) should parse");
        assert_eq!(node, EffectNode::Leaf(Effect::SpawnBolt));
    }

    #[test]
    fn effect_node_ron_nested_triggers() {
        let ron_str = "Trigger(OnPerfectBump, [Trigger(OnImpact(Cell), [Leaf(Shockwave(base_range: 64.0, range_per_level: 0.0, stacks: 1, speed: 400.0))])])";
        let node: EffectNode =
            ron::de::from_str(ron_str).expect("nested EffectNode RON should parse");
        assert_eq!(
            node,
            EffectNode::Trigger(
                Trigger::OnPerfectBump,
                vec![EffectNode::Trigger(
                    Trigger::OnImpact(ImpactTarget::Cell),
                    vec![EffectNode::Leaf(Effect::test_shockwave(64.0))],
                )]
            )
        );
    }

    #[test]
    fn effect_node_ron_multiple_children() {
        let ron_str =
            "Trigger(OnBump, [Leaf(SpawnBolt), Leaf(SpeedBoost(target: Bolt, multiplier: 1.2))])";
        let node: EffectNode =
            ron::de::from_str(ron_str).expect("EffectNode with multiple children should parse");
        match &node {
            EffectNode::Trigger(Trigger::OnBump, children) => {
                assert_eq!(children.len(), 2);
            }
            other => panic!("expected Trigger(OnBump, 2 children), got {other:?}"),
        }
    }

    #[test]
    fn effect_node_ron_on_selected_with_passive() {
        let ron_str = "Trigger(OnSelected, [Leaf(Piercing(1))])";
        let node: EffectNode =
            ron::de::from_str(ron_str).expect("OnSelected EffectNode should parse");
        assert_eq!(
            node,
            EffectNode::Trigger(
                Trigger::OnSelected,
                vec![EffectNode::Leaf(Effect::Piercing(1))]
            )
        );
    }

    #[test]
    fn effect_node_ron_on_selected_multiple_passives() {
        let ron_str = "Trigger(OnSelected, [Leaf(Piercing(1)), Leaf(DamageBoost(0.5))])";
        let node: EffectNode =
            ron::de::from_str(ron_str).expect("OnSelected with multiple passives should parse");
        assert_eq!(
            node,
            EffectNode::Trigger(
                Trigger::OnSelected,
                vec![
                    EffectNode::Leaf(Effect::Piercing(1)),
                    EffectNode::Leaf(Effect::DamageBoost(0.5)),
                ]
            )
        );
    }

    // =========================================================================
    // B12b: ArchetypeDefinition EffectNode RON format (behavior 21)
    // These tests verify the EffectNode shapes that ArchetypeDefinition
    // fields will hold after migration. They exercise evaluate_node which
    // fails with todo!().
    // =========================================================================

    #[test]
    fn effect_node_archetype_bolt_lost_evaluates_correctly() {
        use super::super::evaluate::{NodeEvalResult, TriggerKind, evaluate_node};

        // After migration: on_bolt_lost: Some(Trigger(OnBoltLost, [Leaf(LoseLife)]))
        let node = EffectNode::Trigger(
            Trigger::OnBoltLost,
            vec![EffectNode::Leaf(Effect::LoseLife)],
        );
        let result = evaluate_node(TriggerKind::BoltLost, &node);
        assert_eq!(
            result,
            vec![NodeEvalResult::Fire(Effect::LoseLife)],
            "archetype on_bolt_lost EffectNode should fire LoseLife on BoltLost"
        );
    }

    #[test]
    fn effect_node_archetype_ron_format_deserializes() {
        // Verify the new RON format for ArchetypeDefinition fields
        let ron_str = "Trigger(OnBoltLost, [Leaf(LoseLife)])";
        let node: EffectNode =
            ron::de::from_str(ron_str).expect("archetype EffectNode RON should parse");
        assert_eq!(
            node,
            EffectNode::Trigger(
                Trigger::OnBoltLost,
                vec![EffectNode::Leaf(Effect::LoseLife)]
            )
        );
    }

    #[test]
    fn effect_node_archetype_chains_ron_format_deserializes() {
        // Verify chains field new format: full EffectNode trees
        let ron_str = "Trigger(OnPerfectBump, [Leaf(SpeedBoost(target: Bolt, multiplier: 1.3))])";
        let node: EffectNode =
            ron::de::from_str(ron_str).expect("archetype chains EffectNode RON should parse");
        assert_eq!(
            node,
            EffectNode::Trigger(
                Trigger::OnPerfectBump,
                vec![EffectNode::Leaf(Effect::SpeedBoost {
                    target: Target::Bolt,
                    multiplier: 1.3,
                })]
            )
        );
    }

    // =========================================================================
    // Existing tests (preserved — will be updated by writer-code when
    // ArchetypeDefinition migrates from TriggerChain to EffectNode)
    // =========================================================================

    #[test]
    fn aegis_ron_parses() {
        let ron_str = include_str!("../../assets/breakers/aegis.breaker.ron");
        let def: ArchetypeDefinition =
            ron::de::from_str(ron_str).expect("aegis archetype RON should parse");
        assert_eq!(def.name, "Aegis");
        assert_eq!(def.life_pool, Some(3));
        assert!(matches!(def.on_bolt_lost, Some(TriggerChain::LoseLife)));
        assert!(matches!(
            def.on_perfect_bump,
            Some(TriggerChain::SpeedBoost { multiplier, .. }) if (multiplier - 1.5).abs() < f32::EPSILON
        ));
    }

    #[test]
    fn chrono_ron_file_parses() {
        let ron_str = include_str!("../../assets/breakers/chrono.breaker.ron");
        let def: ArchetypeDefinition =
            ron::de::from_str(ron_str).expect("chrono archetype RON should parse");
        assert_eq!(def.name, "Chrono");
        assert!(def.life_pool.is_none());
        assert!(matches!(
            def.on_bolt_lost,
            Some(TriggerChain::TimePenalty { seconds }) if (seconds - 5.0).abs() < f32::EPSILON
        ));
    }

    #[test]
    fn prism_ron_file_parses() {
        let ron_str = include_str!("../../assets/breakers/prism.breaker.ron");
        let def: ArchetypeDefinition =
            ron::de::from_str(ron_str).expect("prism archetype RON should parse");
        assert_eq!(def.name, "Prism");
        assert!(def.life_pool.is_none());
        assert!(matches!(def.on_perfect_bump, Some(TriggerChain::SpawnBolt)));
        assert!(matches!(
            def.on_bolt_lost,
            Some(TriggerChain::TimePenalty { seconds }) if (seconds - 7.0).abs() < f32::EPSILON
        ));
        assert!(def.on_early_bump.is_none());
        assert!(def.on_late_bump.is_none());
    }

    #[test]
    fn prism_ron_parses() {
        let ron_str = r#"
        (
            name: "Prism",
            stat_overrides: (),
            life_pool: None,
            on_bolt_lost: None,
            on_perfect_bump: Some(SpawnBolt),
            on_early_bump: None,
            on_late_bump: None,
            chains: [],
        )
        "#;
        let def: ArchetypeDefinition =
            ron::de::from_str(ron_str).expect("prism archetype RON should parse");
        assert_eq!(def.name, "Prism");
        assert!(def.life_pool.is_none());
        assert!(matches!(def.on_perfect_bump, Some(TriggerChain::SpawnBolt)));
    }

    #[test]
    fn default_stat_overrides_are_all_none() {
        let overrides = BreakerStatOverrides::default();
        assert!(overrides.width.is_none());
        assert!(overrides.height.is_none());
        assert!(overrides.max_speed.is_none());
        assert!(overrides.acceleration.is_none());
        assert!(overrides.deceleration.is_none());
    }

    #[test]
    fn chrono_ron_parses() {
        let ron_str = r#"
        (
            name: "Chrono",
            stat_overrides: (),
            life_pool: None,
            on_bolt_lost: Some(TimePenalty(seconds: 5.0)),
            on_perfect_bump: Some(SpeedBoost(target: Bolt, multiplier: 1.5)),
            on_early_bump: Some(SpeedBoost(target: Bolt, multiplier: 1.1)),
            on_late_bump: Some(SpeedBoost(target: Bolt, multiplier: 1.1)),
            chains: [],
        )
        "#;
        let def: ArchetypeDefinition =
            ron::de::from_str(ron_str).expect("chrono archetype RON should parse");
        assert_eq!(def.name, "Chrono");
        assert!(def.life_pool.is_none());
        assert!(matches!(
            def.on_bolt_lost,
            Some(TriggerChain::TimePenalty { seconds }) if (seconds - 5.0).abs() < f32::EPSILON
        ));
    }

    #[test]
    fn archetype_with_chains_parses() {
        let ron_str = r#"
        (
            name: "Test",
            stat_overrides: (),
            life_pool: None,
            on_bolt_lost: None,
            on_perfect_bump: None,
            on_early_bump: None,
            on_late_bump: None,
            chains: [
                OnPerfectBump([OnImpact(Cell, [Shockwave(base_range: 64.0, range_per_level: 0.0, stacks: 1, speed: 400.0)])]),
            ],
        )
        "#;
        let def: ArchetypeDefinition =
            ron::de::from_str(ron_str).expect("archetype with chains should parse");
        assert_eq!(def.chains.len(), 1);
    }

    #[test]
    fn apply_stat_overrides_partial() {
        use super::super::init::apply_stat_overrides;
        use crate::breaker::resources::BreakerConfig;

        let mut config = BreakerConfig::default();
        let original_max_speed = config.max_speed;

        let overrides = BreakerStatOverrides {
            width: Some(200.0),
            height: Some(30.0),
            ..default()
        };

        apply_stat_overrides(&mut config, &overrides);

        assert!((config.width - 200.0).abs() < f32::EPSILON);
        assert!((config.height - 30.0).abs() < f32::EPSILON);
        assert!(
            (config.max_speed - original_max_speed).abs() < f32::EPSILON,
            "unset fields should remain unchanged"
        );
    }
}
