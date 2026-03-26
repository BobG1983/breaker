//! Breaker and effect type definitions — RON-deserialized data structures.

use bevy::prelude::*;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Shared enums
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
    /// Target the specific cell entity that was hit.
    Cell,
    /// Target the wall entity that was hit.
    Wall,
    /// Target all cell entities in the current node.
    AllCells,
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

/// Discriminates what type of entity attraction targets.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttractionType {
    /// Attract nearby cells.
    Cell,
    /// Attract toward walls.
    Wall,
    /// Attract toward breaker.
    Breaker,
}

/// The trigger condition that gates an effect subtree.
///
/// Note: `Trigger` does NOT derive `Eq` because `TimeExpires(f32)` contains an f32.
/// It DOES derive `Copy` because f32 is `Copy`.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Trigger {
    /// Fires on a perfect bump.
    #[serde(rename = "OnPerfectBump")]
    PerfectBump,
    /// Fires on any non-whiff bump (Early, Late, or Perfect).
    #[serde(rename = "OnBump")]
    Bump,
    /// Fires on an early bump.
    #[serde(rename = "OnEarlyBump")]
    EarlyBump,
    /// Fires on a late bump.
    #[serde(rename = "OnLateBump")]
    LateBump,
    /// Fires when a bump whiffs (misses).
    #[serde(rename = "OnBumpWhiff")]
    BumpWhiff,
    /// Fires on bolt impact with a specific surface.
    #[serde(rename = "OnImpact")]
    Impact(ImpactTarget),
    /// Fires when a cell is destroyed.
    #[serde(rename = "OnCellDestroyed")]
    CellDestroyed,
    /// Fires when a bolt is lost.
    #[serde(rename = "OnBoltLost")]
    BoltLost,
    /// Fires when the breaker dies (all lives lost or timer expired).
    #[serde(rename = "OnDeath")]
    Death,
    /// Fires when the bolt passes the breaker without being bumped.
    NoBump,
    /// Fires after a perfect bump completes (post-bump event, not the bump itself).
    PerfectBumped,
    /// Fires after any non-whiff bump completes (post-bump event).
    Bumped,
    /// Fires after an early bump completes (post-bump event).
    EarlyBumped,
    /// Fires after a late bump completes (post-bump event).
    LateBumped,
    /// Passive effects: evaluated immediately on chip selection.
    #[serde(rename = "OnSelected")]
    Selected,
    /// Timer-based expiry trigger (duration in seconds).
    TimeExpires(f32),
    /// Fires when the node timer ratio crosses below the threshold.
    #[serde(rename = "OnNodeTimerThreshold")]
    NodeTimerThreshold(f32),
}

// ---------------------------------------------------------------------------
// Effect enum
// ---------------------------------------------------------------------------

/// Default count for `SpawnBolts` serde deserialization.
fn default_spawn_bolts_count() -> u32 {
    1
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
    /// Damage multiplier per stack (1.x format: 2.0 = 2x damage).
    DamageBoost(f32),
    /// Scales a target's speed by a multiplier, clamped within base/max bounds.
    SpeedBoost {
        /// Multiplier applied to the current velocity magnitude (1.x format).
        multiplier: f32,
    },
    /// Bolt chains to N additional cells on hit.
    ChainHit(u32),
    /// Size boost: adjusts radius for bolts, width for breakers.
    SizeBoost(f32),
    /// Bolt attracts nearby entities of the specified type (force per stack).
    Attraction(AttractionType, f32),
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
    /// Spawns additional bolts with configurable parameters.
    SpawnBolts {
        /// Number of bolts to spawn.
        #[serde(default = "default_spawn_bolts_count")]
        count: u32,
        /// Optional lifespan in seconds (temporary bolts).
        #[serde(default)]
        lifespan: Option<f32>,
        /// Whether spawned bolts inherit the parent bolt's velocity.
        #[serde(default)]
        inherit: bool,
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
    /// Ramping damage bonus that accumulates per cell hit and resets on breaker bounce.
    RampingDamage {
        /// Damage bonus added per cell hit.
        bonus_per_hit: f32,
    },
    /// Selects a random effect from a weighted pool of `EffectNode` entries.
    RandomEffect(Vec<(f32, EffectNode)>),
    /// Counts cell destructions and fires a random effect from the pool when threshold reached.
    EntropyEngine {
        /// Number of cell destructions needed before firing.
        threshold: u32,
        /// Weighted pool of `EffectNode` entries to select from on trigger.
        pool: Vec<(f32, EffectNode)>,
    },
    /// Shockwave at every active bolt position simultaneously.
    Pulse {
        /// Base radius of each shockwave.
        base_range: f32,
        /// Additional radius per stack beyond the first.
        range_per_level: f32,
        /// Current stack count.
        stacks: u32,
        /// Expansion speed in world units per second.
        speed: f32,
    },
}

// ---------------------------------------------------------------------------
// EffectNode — the effect tree shape
// ---------------------------------------------------------------------------

/// A node in the effect tree — `When`/`Do`/`Until`/`Once`/`On`.
///
/// Replaces the old `Trigger(Trigger, Vec<EffectNode>)` / `Leaf(Effect)` shape.
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub enum EffectNode {
    /// A trigger condition with child nodes evaluated when the trigger fires.
    When {
        /// The trigger that gates this subtree.
        trigger: Trigger,
        /// Child nodes evaluated when the trigger fires.
        then: Vec<EffectNode>,
    },
    /// A terminal effect action.
    Do(Effect),
    /// A removal condition — child effects are active until the trigger fires.
    Until {
        /// The trigger that removes these effects.
        until: Trigger,
        /// Child nodes active until the trigger fires.
        then: Vec<EffectNode>,
    },
    /// A one-shot wrapper — children fire once and are consumed.
    Once(Vec<EffectNode>),
    /// A target scope — children are dispatched against the specified target entity.
    ///
    /// `On` nodes are not evaluated by trigger matching; they are resolved at
    /// dispatch time to determine the entity context for child effects.
    On {
        /// Which entity type the child effects target.
        target: Target,
        /// Child nodes dispatched against the target entity.
        then: Vec<EffectNode>,
    },
}

// ---------------------------------------------------------------------------
// RootEffect — breaker-definition entry point (always starts with On)
// ---------------------------------------------------------------------------

/// A root-level effect declaration — always scoped to a target entity.
///
/// `RootEffect` constrains breaker definitions so that every top-level effect
/// chain explicitly names its target. It converts into an [`EffectNode::On`]
/// for tree evaluation.
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub enum RootEffect {
    /// A target-scoped effect chain.
    On {
        /// Which entity type the child effects target.
        target: Target,
        /// Child nodes dispatched against the target entity.
        then: Vec<EffectNode>,
    },
}

impl From<RootEffect> for EffectNode {
    fn from(r: RootEffect) -> Self {
        let RootEffect::On { target, then } = r;
        EffectNode::On { target, then }
    }
}

// ---------------------------------------------------------------------------
// EffectEntity marker (renamed from EffectTarget)
// ---------------------------------------------------------------------------

/// Marker component for entities that can have effects armed on them.
///
/// Added to `Bolt` and `Breaker` entities via `#[require(EffectEntity)]`.
/// Effect queries filter `With<EffectEntity>`.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct EffectEntity;

// ---------------------------------------------------------------------------
// EffectTarget runtime enum (NOT in RON — runtime only)
// ---------------------------------------------------------------------------

/// Runtime target for a triggered effect — either a specific entity or a location.
///
/// Does NOT derive `Deserialize` — this is runtime-only, not loaded from RON.
#[derive(Clone, Debug, PartialEq)]
pub enum EffectTarget {
    /// Target a specific entity.
    Entity(Entity),
    /// Target a world-space location.
    Location(Vec2),
}

// ---------------------------------------------------------------------------
// EffectChains component
// ---------------------------------------------------------------------------

/// Entity-local effect chains — the primary source of truth for all trigger chains.
///
/// Coexists with `ArmedEffects` (bolt component for partially-resolved chains).
/// Each entry is `(chip_name, chain)` where `chip_name` is `None` for
/// breaker-originating chains and `Some(name)` for chip/evolution chains.
/// Populated by `init_breaker`, `dispatch_chip_effects`, and `On` node dispatch.
#[derive(Component, Debug, Default, Clone)]
pub struct EffectChains(pub Vec<(Option<String>, EffectNode)>);

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

    /// Build a `SpawnBolts` leaf with default parameters.
    pub(crate) fn test_spawn_bolts() -> Self {
        Self::SpawnBolts {
            count: 1,
            lifespan: None,
            inherit: false,
        }
    }

    /// Build a `SpeedBoost` leaf with the given multiplier.
    pub(crate) fn test_speed_boost(multiplier: f32) -> Self {
        Self::SpeedBoost { multiplier }
    }

    /// Build a `Piercing` leaf with the given count.
    pub(crate) fn test_piercing(count: u32) -> Self {
        Self::Piercing(count)
    }

    /// Build a `DamageBoost` leaf with the given boost value.
    pub(crate) fn test_damage_boost(boost: f32) -> Self {
        Self::DamageBoost(boost)
    }

    /// Build a `RampingDamage` leaf with the given per-hit bonus.
    pub(crate) fn test_ramping_damage(bonus_per_hit: f32) -> Self {
        Self::RampingDamage { bonus_per_hit }
    }

    /// Build a `RandomEffect` variant with the given pool.
    pub(crate) fn test_random_effect(pool: Vec<(f32, EffectNode)>) -> Self {
        Self::RandomEffect(pool)
    }

    /// Build an `EntropyEngine` variant with the given threshold and pool.
    pub(crate) fn test_entropy_engine(threshold: u32, pool: Vec<(f32, EffectNode)>) -> Self {
        Self::EntropyEngine { threshold, pool }
    }

    /// Build a `Pulse` leaf with `range_per_level: 0.0`, `stacks: 1`, and `speed: 400.0`.
    pub(crate) fn test_pulse(range: f32) -> Self {
        Self::Pulse {
            base_range: range,
            range_per_level: 0.0,
            stacks: 1,
            speed: 400.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Test constructors for EffectNode
// ---------------------------------------------------------------------------

#[cfg(test)]
impl EffectNode {
    /// Build a `When` + `Do` node — convenience for single-leaf triggered effects.
    pub(crate) fn trigger_leaf(trigger: Trigger, effect: Effect) -> Self {
        Self::When {
            trigger,
            then: vec![Self::Do(effect)],
        }
    }
}

// ---------------------------------------------------------------------------
// Breaker definition re-exports (canonical location: breaker/definition.rs)
// ---------------------------------------------------------------------------

pub(crate) use crate::breaker::definition::{BreakerDefinition, BreakerStatOverrides};

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // C7 Wave 1 Part A: EffectNode construction (behaviors 1-6)
    // =========================================================================

    #[test]
    fn effect_node_when_wraps_trigger_and_children() {
        let node = EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(Effect::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            })],
        };
        match &node {
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                then,
            } => {
                assert_eq!(then.len(), 1);
            }
            other => panic!("expected When(PerfectBump, _), got {other:?}"),
        }
    }

    #[test]
    fn effect_node_when_empty_then_is_valid() {
        let node = EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![],
        };
        match &node {
            EffectNode::When {
                trigger: Trigger::Bump,
                then,
            } => {
                assert!(then.is_empty());
            }
            other => panic!("expected When(Bump, []), got {other:?}"),
        }
    }

    #[test]
    fn effect_node_do_wraps_effect_leaf() {
        let node = EffectNode::Do(Effect::LoseLife);
        assert!(matches!(node, EffectNode::Do(Effect::LoseLife)));
    }

    #[test]
    fn effect_node_do_wraps_spawn_bolts() {
        let node = EffectNode::Do(Effect::SpawnBolts {
            count: 1,
            lifespan: None,
            inherit: false,
        });
        assert!(matches!(
            node,
            EffectNode::Do(Effect::SpawnBolts { count: 1, .. })
        ));
    }

    #[test]
    fn effect_node_until_wraps_trigger_and_children() {
        let node = EffectNode::Until {
            until: Trigger::TimeExpires(3.0),
            then: vec![EffectNode::Do(Effect::DamageBoost(2.0))],
        };
        match &node {
            EffectNode::Until { until, then } => {
                assert_eq!(*until, Trigger::TimeExpires(3.0));
                assert_eq!(then.len(), 1);
            }
            other => panic!("expected Until(TimeExpires(3.0), _), got {other:?}"),
        }
    }

    #[test]
    fn effect_node_until_with_impact_breaker_removal() {
        let node = EffectNode::Until {
            until: Trigger::Impact(ImpactTarget::Breaker),
            then: vec![],
        };
        assert!(matches!(
            node,
            EffectNode::Until {
                until: Trigger::Impact(ImpactTarget::Breaker),
                ..
            }
        ));
    }

    #[test]
    fn effect_node_once_wraps_children() {
        let node = EffectNode::Once(vec![EffectNode::Do(Effect::SecondWind {
            invuln_secs: 3.0,
        })]);
        match &node {
            EffectNode::Once(children) => {
                assert_eq!(children.len(), 1);
            }
            other => panic!("expected Once(_), got {other:?}"),
        }
    }

    #[test]
    fn effect_node_once_empty_is_valid() {
        let node = EffectNode::Once(vec![]);
        match &node {
            EffectNode::Once(children) => {
                assert!(children.is_empty());
            }
            other => panic!("expected Once([]), got {other:?}"),
        }
    }

    #[test]
    fn effect_node_nests_when_inside_when_two_deep() {
        let node = EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::When {
                trigger: Trigger::Impact(ImpactTarget::Cell),
                then: vec![EffectNode::Do(Effect::Shockwave {
                    base_range: 64.0,
                    range_per_level: 0.0,
                    stacks: 1,
                    speed: 400.0,
                })],
            }],
        };
        match &node {
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                then,
            } => match &then[0] {
                EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Cell),
                    then: inner,
                } => {
                    assert!(matches!(inner[0], EffectNode::Do(Effect::Shockwave { .. })));
                }
                other => panic!("expected inner When(Impact(Cell), _), got {other:?}"),
            },
            other => panic!("expected outer When(PerfectBump, _), got {other:?}"),
        }
    }

    #[test]
    fn effect_node_nests_three_deep() {
        let node = EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::When {
                trigger: Trigger::Impact(ImpactTarget::Cell),
                then: vec![EffectNode::When {
                    trigger: Trigger::CellDestroyed,
                    then: vec![EffectNode::Do(Effect::test_shockwave(64.0))],
                }],
            }],
        };
        assert!(matches!(
            node,
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                ..
            }
        ));
    }

    #[test]
    fn effect_node_when_containing_until_with_do_leaves() {
        let node = EffectNode::When {
            trigger: Trigger::BumpWhiff,
            then: vec![EffectNode::When {
                trigger: Trigger::Impact(ImpactTarget::Cell),
                then: vec![EffectNode::Until {
                    until: Trigger::Impact(ImpactTarget::Breaker),
                    then: vec![
                        EffectNode::Do(Effect::DamageBoost(1.5)),
                        EffectNode::Do(Effect::Shockwave {
                            base_range: 64.0,
                            range_per_level: 0.0,
                            stacks: 1,
                            speed: 400.0,
                        }),
                    ],
                }],
            }],
        };
        // Verify the nested structure has 2 Do leaves inside Until
        match &node {
            EffectNode::When { then, .. } => match &then[0] {
                EffectNode::When { then: inner, .. } => match &inner[0] {
                    EffectNode::Until {
                        then: until_kids, ..
                    } => {
                        assert_eq!(until_kids.len(), 2, "Until node should contain 2 Do leaves");
                    }
                    other => panic!("expected Until, got {other:?}"),
                },
                other => panic!("expected inner When, got {other:?}"),
            },
            other => panic!("expected outer When, got {other:?}"),
        }
    }

    // =========================================================================
    // C7 Wave 1 Part A: EffectNode RON deserialization (behaviors 7-10)
    // =========================================================================

    #[test]
    fn effect_node_ron_when_with_do_leaf() {
        let ron_str = "When(trigger: OnPerfectBump, then: [Do(Shockwave(base_range: 64.0, range_per_level: 0.0, stacks: 1, speed: 400.0))])";
        let node: EffectNode =
            ron::de::from_str(ron_str).expect("EffectNode When+Do RON should parse");
        assert_eq!(
            node,
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(Effect::Shockwave {
                    base_range: 64.0,
                    range_per_level: 0.0,
                    stacks: 1,
                    speed: 400.0,
                })]
            }
        );
    }

    #[test]
    fn effect_node_ron_bare_do_lose_life() {
        let ron_str = "Do(LoseLife)";
        let node: EffectNode = ron::de::from_str(ron_str).expect("bare Do(LoseLife) should parse");
        assert_eq!(node, EffectNode::Do(Effect::LoseLife));
    }

    #[test]
    fn effect_node_ron_until_node() {
        let ron_str = "Until(until: TimeExpires(3.0), then: [Do(DamageBoost(2.0))])";
        let node: EffectNode = ron::de::from_str(ron_str).expect("Until node RON should parse");
        assert_eq!(
            node,
            EffectNode::Until {
                until: Trigger::TimeExpires(3.0),
                then: vec![EffectNode::Do(Effect::DamageBoost(2.0))]
            }
        );
    }

    #[test]
    fn effect_node_ron_until_empty_then() {
        let ron_str = "Until(until: OnImpact(Breaker), then: [])";
        let node: EffectNode =
            ron::de::from_str(ron_str).expect("Until with empty then should parse");
        assert_eq!(
            node,
            EffectNode::Until {
                until: Trigger::Impact(ImpactTarget::Breaker),
                then: vec![]
            }
        );
    }

    #[test]
    fn effect_node_ron_once_node() {
        let ron_str = "Once([Do(SecondWind(invuln_secs: 3.0))])";
        let node: EffectNode = ron::de::from_str(ron_str).expect("Once node RON should parse");
        assert_eq!(
            node,
            EffectNode::Once(vec![EffectNode::Do(Effect::SecondWind {
                invuln_secs: 3.0
            })])
        );
    }

    #[test]
    fn effect_node_ron_once_empty() {
        let ron_str = "Once([])";
        let node: EffectNode = ron::de::from_str(ron_str).expect("Once([]) should parse");
        assert_eq!(node, EffectNode::Once(vec![]));
    }

    #[test]
    fn effect_node_ron_nested_when_until_do_combo() {
        let ron_str = "When(trigger: OnBumpWhiff, then: [When(trigger: OnImpact(Cell), then: [Until(until: OnImpact(Breaker), then: [Do(DamageBoost(1.5)), Do(Shockwave(base_range: 64.0, range_per_level: 0.0, stacks: 1, speed: 400.0))])])])";
        let node: EffectNode =
            ron::de::from_str(ron_str).expect("nested When/Until/Do RON should parse");
        // Verify outer When
        match &node {
            EffectNode::When {
                trigger: Trigger::BumpWhiff,
                then,
            } => match &then[0] {
                EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Cell),
                    then: inner,
                } => match &inner[0] {
                    EffectNode::Until {
                        until: Trigger::Impact(ImpactTarget::Breaker),
                        then: until_kids,
                    } => {
                        assert_eq!(until_kids.len(), 2, "Until should have 2 Do children");
                    }
                    other => panic!("expected Until, got {other:?}"),
                },
                other => panic!("expected inner When(Impact(Cell)), got {other:?}"),
            },
            other => panic!("expected outer When(BumpWhiff), got {other:?}"),
        }
    }

    // =========================================================================
    // C7 Wave 1 Part A: trigger_leaf helper (behavior 11)
    // =========================================================================

    #[test]
    fn effect_node_trigger_leaf_builds_when_do() {
        let node = EffectNode::trigger_leaf(Trigger::BoltLost, Effect::LoseLife);
        assert_eq!(
            node,
            EffectNode::When {
                trigger: Trigger::BoltLost,
                then: vec![EffectNode::Do(Effect::LoseLife)]
            }
        );
    }

    #[test]
    fn effect_node_trigger_leaf_on_selected() {
        let node = EffectNode::trigger_leaf(Trigger::Selected, Effect::Piercing(1));
        assert_eq!(
            node,
            EffectNode::When {
                trigger: Trigger::Selected,
                then: vec![EffectNode::Do(Effect::Piercing(1))]
            }
        );
    }

    // =========================================================================
    // C7 Wave 1 Part B: Trigger enum new variants (behaviors 12-16)
    // =========================================================================

    #[test]
    fn trigger_time_expires_constructs_and_clones() {
        let t = Trigger::TimeExpires(3.0);
        let cloned = t;
        assert_eq!(t, cloned);
        assert_eq!(t, Trigger::TimeExpires(3.0));
    }

    #[test]
    fn trigger_time_expires_zero_is_valid() {
        let t = Trigger::TimeExpires(0.0);
        assert_eq!(t, Trigger::TimeExpires(0.0));
    }

    #[test]
    fn trigger_on_death_constructs() {
        let t = Trigger::Death;
        assert!(matches!(t, Trigger::Death));
    }

    #[test]
    fn trigger_on_death_distinct_from_bolt_lost() {
        assert_ne!(Trigger::Death, Trigger::BoltLost);
    }

    #[test]
    fn trigger_ron_time_expires() {
        let t: Trigger =
            ron::de::from_str("TimeExpires(3.0)").expect("TimeExpires RON should parse");
        assert_eq!(t, Trigger::TimeExpires(3.0));
    }

    #[test]
    fn trigger_ron_time_expires_zero() {
        let t: Trigger =
            ron::de::from_str("TimeExpires(0.0)").expect("TimeExpires(0.0) RON should parse");
        assert_eq!(t, Trigger::TimeExpires(0.0));
    }

    #[test]
    fn trigger_ron_on_death() {
        let t: Trigger = ron::de::from_str("OnDeath").expect("OnDeath RON should parse");
        assert_eq!(t, Trigger::Death);
    }

    // =========================================================================
    // C7 Wave 2a: OnNodeTimerThreshold RON deserialization (behavior 12)
    // =========================================================================

    #[test]
    fn trigger_ron_on_node_timer_threshold() {
        let t: Trigger = ron::de::from_str("OnNodeTimerThreshold(0.25)")
            .expect("OnNodeTimerThreshold(0.25) RON should parse");
        assert_eq!(t, Trigger::NodeTimerThreshold(0.25));
    }

    #[test]
    fn trigger_ron_on_node_timer_threshold_zero() {
        let t: Trigger = ron::de::from_str("OnNodeTimerThreshold(0.0)")
            .expect("OnNodeTimerThreshold(0.0) RON should parse");
        assert_eq!(t, Trigger::NodeTimerThreshold(0.0));
    }

    #[test]
    fn trigger_ron_on_node_timer_threshold_one() {
        let t: Trigger = ron::de::from_str("OnNodeTimerThreshold(1.0)")
            .expect("OnNodeTimerThreshold(1.0) RON should parse");
        assert_eq!(t, Trigger::NodeTimerThreshold(1.0));
    }

    #[test]
    fn trigger_ron_invalid_variant_fails() {
        let result = ron::de::from_str::<Trigger>("OnGameEnd");
        assert!(
            result.is_err(),
            "invalid trigger variant should fail to parse"
        );
    }

    #[test]
    fn trigger_enum_has_all_fourteen_patterns() {
        let triggers = [
            Trigger::PerfectBump,
            Trigger::Bump,
            Trigger::EarlyBump,
            Trigger::LateBump,
            Trigger::BumpWhiff,
            Trigger::Impact(ImpactTarget::Cell),
            Trigger::Impact(ImpactTarget::Breaker),
            Trigger::Impact(ImpactTarget::Wall),
            Trigger::CellDestroyed,
            Trigger::BoltLost,
            Trigger::Death,
            Trigger::Selected,
            Trigger::TimeExpires(1.0),
            Trigger::NodeTimerThreshold(0.25),
        ];
        assert_eq!(
            triggers.len(),
            14,
            "all 14 distinguishable trigger patterns"
        );
    }

    #[test]
    fn trigger_is_copy_but_not_eq() {
        // Verify Copy works (f32 is Copy)
        let t = Trigger::TimeExpires(3.0);
        let copied = t; // Copy, not move
        let also = t; // still usable — proves Copy
        assert_eq!(copied, also);

        // Eq is NOT derived because f32 doesn't implement Eq.
        // We can only verify PartialEq works:
        assert_eq!(t, t);
    }

    // =========================================================================
    // C7 Wave 1 Part C: Effect enum changes (behaviors 17-22)
    // =========================================================================

    #[test]
    fn effect_spawn_bolts_full_construction() {
        let e = Effect::SpawnBolts {
            count: 2,
            lifespan: Some(5.0),
            inherit: true,
        };
        match e {
            Effect::SpawnBolts {
                count,
                lifespan,
                inherit,
            } => {
                assert_eq!(count, 2);
                assert_eq!(lifespan, Some(5.0));
                assert!(inherit);
            }
            other => panic!("expected SpawnBolts, got {other:?}"),
        }
    }

    #[test]
    fn effect_spawn_bolts_default_values() {
        let e = Effect::SpawnBolts {
            count: 1,
            lifespan: None,
            inherit: false,
        };
        match e {
            Effect::SpawnBolts {
                count,
                lifespan,
                inherit,
            } => {
                assert_eq!(count, 1);
                assert!(lifespan.is_none());
                assert!(!inherit);
            }
            other => panic!("expected SpawnBolts, got {other:?}"),
        }
    }

    #[test]
    fn effect_spawn_bolts_ron_with_serde_defaults() {
        let ron_str = "SpawnBolts(count: 3)";
        let e: Effect =
            ron::de::from_str(ron_str).expect("SpawnBolts with partial fields should parse");
        match e {
            Effect::SpawnBolts {
                count,
                lifespan,
                inherit,
            } => {
                assert_eq!(count, 3, "count should be 3");
                assert!(lifespan.is_none(), "lifespan should default to None");
                assert!(!inherit, "inherit should default to false");
            }
            other => panic!("expected SpawnBolts, got {other:?}"),
        }
    }

    #[test]
    fn effect_spawn_bolts_ron_full_form() {
        let ron_str = "SpawnBolts(count: 2, lifespan: Some(5.0), inherit: true)";
        let e: Effect = ron::de::from_str(ron_str).expect("SpawnBolts full form should parse");
        assert_eq!(
            e,
            Effect::SpawnBolts {
                count: 2,
                lifespan: Some(5.0),
                inherit: true,
            }
        );
    }

    #[test]
    fn effect_spawn_bolts_ron_bare_parens_defaults_count_to_one() {
        let ron_str = "SpawnBolts()";
        let e: Effect = ron::de::from_str(ron_str).expect("SpawnBolts() bare parens should parse");
        match e {
            Effect::SpawnBolts {
                count,
                lifespan,
                inherit,
            } => {
                assert_eq!(count, 1, "count should default to 1");
                assert!(lifespan.is_none(), "lifespan should default to None");
                assert!(!inherit, "inherit should default to false");
            }
            other => panic!("expected SpawnBolts, got {other:?}"),
        }
    }

    #[test]
    fn effect_spawn_bolts_ron_count_override() {
        let ron_str = "SpawnBolts(count: 5)";
        let e: Effect = ron::de::from_str(ron_str).expect("SpawnBolts(count: 5) should parse");
        match e {
            Effect::SpawnBolts { count, .. } => {
                assert_eq!(count, 5, "count should be overridden to 5");
            }
            other => panic!("expected SpawnBolts, got {other:?}"),
        }
    }

    #[test]
    fn effect_attraction_with_attraction_type() {
        let e = Effect::Attraction(AttractionType::Cell, 1.0);
        assert!(matches!(
            e,
            Effect::Attraction(AttractionType::Cell, v) if (v - 1.0).abs() < f32::EPSILON
        ));
    }

    #[test]
    fn effect_attraction_wall_variant() {
        let e = Effect::Attraction(AttractionType::Wall, 0.5);
        assert!(matches!(e, Effect::Attraction(AttractionType::Wall, _)));
    }

    #[test]
    fn effect_attraction_breaker_variant() {
        let e = Effect::Attraction(AttractionType::Breaker, 2.0);
        assert!(matches!(e, Effect::Attraction(AttractionType::Breaker, _)));
    }

    #[test]
    fn attraction_type_ron_deserialization() {
        let e: Effect =
            ron::de::from_str("Attraction(Cell, 1.0)").expect("Attraction(Cell, 1.0) should parse");
        assert_eq!(e, Effect::Attraction(AttractionType::Cell, 1.0));
    }

    #[test]
    fn attraction_type_ron_wall() {
        let e: Effect =
            ron::de::from_str("Attraction(Wall, 0.5)").expect("Attraction(Wall, 0.5) should parse");
        assert_eq!(e, Effect::Attraction(AttractionType::Wall, 0.5));
    }

    #[test]
    fn attraction_type_ron_breaker() {
        let e: Effect = ron::de::from_str("Attraction(Breaker, 2.0)")
            .expect("Attraction(Breaker, 2.0) should parse");
        assert_eq!(e, Effect::Attraction(AttractionType::Breaker, 2.0));
    }

    #[test]
    fn effect_enum_has_all_twenty_three_variants() {
        let effects: Vec<Effect> = vec![
            Effect::Shockwave {
                base_range: 64.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 400.0,
            },
            Effect::Piercing(1),
            Effect::DamageBoost(0.5),
            Effect::SpeedBoost { multiplier: 1.2 },
            Effect::ChainHit(2),
            Effect::SizeBoost(5.0),
            Effect::Attraction(AttractionType::Cell, 0.3),
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
            Effect::SpawnBolts {
                count: 1,
                lifespan: None,
                inherit: false,
            },
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
            Effect::RampingDamage {
                bonus_per_hit: 0.04,
            },
            Effect::RandomEffect(vec![(
                1.0,
                EffectNode::Do(Effect::SpawnBolts {
                    count: 1,
                    lifespan: None,
                    inherit: false,
                }),
            )]),
            Effect::EntropyEngine {
                threshold: 5,
                pool: vec![(
                    1.0,
                    EffectNode::Do(Effect::SpawnBolts {
                        count: 1,
                        lifespan: None,
                        inherit: false,
                    }),
                )],
            },
        ];
        assert_eq!(effects.len(), 23, "all 23 Effect variants");
    }

    // =========================================================================
    // C7 Wave 1 Part D: EffectChains component (behaviors 23-25)
    // =========================================================================

    #[test]
    fn effect_chains_default_is_empty() {
        let chains = EffectChains::default();
        assert!(chains.0.is_empty());
    }

    #[test]
    fn effect_chains_stores_mixed_node_types() {
        let chains = EffectChains(vec![
            (
                None,
                EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(Effect::Shockwave {
                        base_range: 64.0,
                        range_per_level: 0.0,
                        stacks: 1,
                        speed: 400.0,
                    })],
                },
            ),
            (None, EffectNode::Do(Effect::Piercing(1))),
        ]);
        assert_eq!(chains.0.len(), 2);
    }

    #[test]
    fn effect_chains_single_do_is_valid() {
        let chains = EffectChains(vec![(None, EffectNode::Do(Effect::Piercing(1)))]);
        assert_eq!(chains.0.len(), 1, "single Do node in chains is valid");
    }

    #[test]
    fn effect_chains_is_queryable_component() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn(EffectChains::default()).id();
        let found = app
            .world()
            .entity(entity)
            .get::<EffectChains>()
            .expect("EffectChains should be queryable as Component");
        assert!(found.0.is_empty());
    }

    #[test]
    fn effect_chains_not_present_returns_none() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn_empty().id();
        assert!(
            app.world().entity(entity).get::<EffectChains>().is_none(),
            "entity without EffectChains should return None"
        );
    }

    // =========================================================================
    // C7 Wave 1 Part E: EffectEntity rename + new EffectTarget (behaviors 26-30)
    // =========================================================================

    #[test]
    fn effect_entity_is_queryable_component() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn(EffectEntity).id();
        assert!(
            app.world().entity(entity).contains::<EffectEntity>(),
            "EffectEntity should be queryable via With<EffectEntity>"
        );
    }

    #[test]
    fn effect_target_entity_variant() {
        let entity = Entity::PLACEHOLDER;
        let target = EffectTarget::Entity(entity);
        match target {
            EffectTarget::Entity(e) => assert_eq!(e, entity),
            other @ EffectTarget::Location(_) => {
                panic!("expected EffectTarget::Entity, got {other:?}")
            }
        }
    }

    #[test]
    fn effect_target_location_variant() {
        let target = EffectTarget::Location(Vec2::new(100.0, 200.0));
        match target {
            EffectTarget::Location(pos) => {
                assert_eq!(pos, Vec2::new(100.0, 200.0));
            }
            other @ EffectTarget::Entity(_) => {
                panic!("expected EffectTarget::Location, got {other:?}")
            }
        }
    }

    #[test]
    fn effect_target_location_zero_is_valid() {
        let target = EffectTarget::Location(Vec2::ZERO);
        assert_eq!(target, EffectTarget::Location(Vec2::ZERO));
    }

    #[test]
    fn effect_target_empty_vec_is_valid() {
        let targets: Vec<EffectTarget> = Vec::new();
        assert!(targets.is_empty());
    }

    #[test]
    fn effect_target_multiple_entities() {
        let targets = [
            EffectTarget::Entity(Entity::PLACEHOLDER),
            EffectTarget::Entity(Entity::PLACEHOLDER),
        ];
        assert_eq!(targets.len(), 2);
    }

    // =========================================================================
    // C7 Wave 1 Part I: BreakerDefinition migration (behaviors 44-46)
    // =========================================================================

    #[test]
    fn breaker_definition_fields_use_effect_node() {
        let def = BreakerDefinition {
            name: "Aegis".to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: Some(3),
            effects: vec![
                RootEffect::On {
                    target: Target::Breaker,
                    then: vec![EffectNode::When {
                        trigger: Trigger::BoltLost,
                        then: vec![EffectNode::Do(Effect::LoseLife)],
                    }],
                },
                RootEffect::On {
                    target: Target::Bolt,
                    then: vec![EffectNode::When {
                        trigger: Trigger::PerfectBumped,
                        then: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.5 })],
                    }],
                },
                RootEffect::On {
                    target: Target::Bolt,
                    then: vec![EffectNode::When {
                        trigger: Trigger::EarlyBumped,
                        then: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.1 })],
                    }],
                },
                RootEffect::On {
                    target: Target::Bolt,
                    then: vec![EffectNode::When {
                        trigger: Trigger::LateBumped,
                        then: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.1 })],
                    }],
                },
            ],
        };
        // Verify BoltLost effect is present
        assert!(matches!(
            &def.effects[0],
            RootEffect::On { target: Target::Breaker, then } if matches!(&then[0], EffectNode::When { trigger: Trigger::BoltLost, then: inner } if matches!(&inner[0], EffectNode::Do(Effect::LoseLife)))
        ));
        // Verify PerfectBumped SpeedBoost is present
        assert!(matches!(
            &def.effects[1],
            RootEffect::On { target: Target::Bolt, then } if matches!(&then[0], EffectNode::When { trigger: Trigger::PerfectBumped, then: inner } if matches!(&inner[0], EffectNode::Do(Effect::SpeedBoost { .. })))
        ));
    }

    #[test]
    fn breaker_definition_none_early_late_bump_is_valid() {
        let def = BreakerDefinition {
            name: "Prism".to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: None,
            effects: vec![RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(Effect::SpawnBolts {
                        count: 1,
                        lifespan: None,
                        inherit: false,
                    })],
                }],
            }],
        };
        // Only one effect — no early/late bump entries
        assert_eq!(def.effects.len(), 1);
    }

    #[test]
    fn breaker_definition_chains_holds_nested_when_tree() {
        let def = BreakerDefinition {
            name: "Test".to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: None,
            effects: vec![RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::When {
                        trigger: Trigger::Impact(ImpactTarget::Cell),
                        then: vec![EffectNode::Do(Effect::Shockwave {
                            base_range: 64.0,
                            range_per_level: 0.0,
                            stacks: 1,
                            speed: 400.0,
                        })],
                    }],
                }],
            }],
        };
        assert_eq!(def.effects.len(), 1);
    }

    #[test]
    fn breaker_definition_empty_chains_is_valid() {
        let def = BreakerDefinition {
            name: "Test".to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: None,
            effects: vec![],
        };
        assert!(def.effects.is_empty());
    }

    #[test]
    fn breaker_definition_ron_with_effect_node_syntax() {
        let ron_str = r#"
        (
            name: "Aegis",
            stat_overrides: (),
            life_pool: Some(3),
            effects: [
                On(target: Breaker, then: [When(trigger: OnBoltLost, then: [Do(LoseLife)])]),
                On(target: Bolt, then: [When(trigger: PerfectBumped, then: [Do(SpeedBoost(multiplier: 1.5))])]),
                On(target: Bolt, then: [When(trigger: EarlyBumped, then: [Do(SpeedBoost(multiplier: 1.1))])]),
                On(target: Bolt, then: [When(trigger: LateBumped, then: [Do(SpeedBoost(multiplier: 1.1))])]),
            ],
        )
        "#;
        let def: BreakerDefinition =
            ron::de::from_str(ron_str).expect("BreakerDefinition with EffectNode RON should parse");
        assert_eq!(def.name, "Aegis");
        assert_eq!(def.effects.len(), 4);
    }

    #[test]
    fn breaker_definition_ron_prism_style_none_early_late() {
        let ron_str = r#"
        (
            name: "Prism",
            stat_overrides: (),
            life_pool: None,
            effects: [
                On(target: Breaker, then: [When(trigger: OnBoltLost, then: [Do(TimePenalty(seconds: 7.0))])]),
                On(target: Breaker, then: [When(trigger: OnPerfectBump, then: [Do(SpawnBolts())])]),
            ],
        )
        "#;
        let def: BreakerDefinition =
            ron::de::from_str(ron_str).expect("Prism-style BreakerDefinition should parse");
        assert_eq!(def.name, "Prism");
        assert_eq!(def.effects.len(), 2);
    }

    // =========================================================================
    // C7 Wave 1 Part J: Multiplier standardization (behaviors 47-48)
    // =========================================================================

    #[test]
    fn damage_boost_uses_multiplier_format() {
        // 2.0 means 2x damage (double), 0.5 means 50% damage (half)
        let double = Effect::DamageBoost(2.0);
        let half = Effect::DamageBoost(0.5);
        assert_eq!(double, Effect::DamageBoost(2.0));
        assert_eq!(half, Effect::DamageBoost(0.5));
    }

    #[test]
    fn speed_boost_uses_multiplier_format() {
        // 1.5 means 1.5x speed, 0.5 means 50% speed
        let fast = Effect::SpeedBoost { multiplier: 1.5 };
        let slow = Effect::SpeedBoost { multiplier: 0.5 };
        assert!(
            matches!(fast, Effect::SpeedBoost { multiplier, .. } if (multiplier - 1.5).abs() < f32::EPSILON)
        );
        assert!(
            matches!(slow, Effect::SpeedBoost { multiplier, .. } if (multiplier - 0.5).abs() < f32::EPSILON)
        );
    }

    // =========================================================================
    // Preserved tests
    // =========================================================================

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

    #[test]
    fn effect_zero_damage_boost_is_valid() {
        let e = Effect::DamageBoost(0.0);
        assert_eq!(e, Effect::DamageBoost(0.0));
    }

    #[test]
    fn effect_speed_boost_all_bolts_target() {
        let e = Effect::SpeedBoost { multiplier: 0.5 };
        assert!(matches!(e, Effect::SpeedBoost { multiplier, .. } if (multiplier - 0.5).abs() < f32::EPSILON));
    }

    #[test]
    fn effect_random_effect_round_trips() {
        let effect = Effect::RandomEffect(vec![
            (
                0.6,
                EffectNode::Do(Effect::SpawnBolts {
                    count: 1,
                    lifespan: None,
                    inherit: false,
                }),
            ),
            (0.4, EffectNode::Do(Effect::test_speed_boost(1.2))),
        ]);
        let cloned = effect.clone();
        assert_eq!(effect, cloned);
    }

    #[test]
    fn effect_entropy_engine_round_trips() {
        let effect = Effect::EntropyEngine {
            threshold: 5,
            pool: vec![
                (
                    0.5,
                    EffectNode::Do(Effect::SpawnBolts {
                        count: 1,
                        lifespan: None,
                        inherit: false,
                    }),
                ),
                (0.5, EffectNode::Do(Effect::test_speed_boost(1.3))),
            ],
        };
        let cloned = effect.clone();
        assert_eq!(effect, cloned);
    }

    // =========================================================================
    // EffectNode::On — construction and serde (Part A)
    // =========================================================================

    #[test]
    fn effect_node_on_wraps_target_and_children() {
        let node = EffectNode::On {
            target: Target::Bolt,
            then: vec![EffectNode::Do(Effect::LoseLife)],
        };
        match &node {
            EffectNode::On { target, then } => {
                assert_eq!(*target, Target::Bolt);
                assert_eq!(then.len(), 1);
                assert!(matches!(then[0], EffectNode::Do(Effect::LoseLife)));
            }
            other => panic!("expected On(Bolt, _), got {other:?}"),
        }
    }

    #[test]
    fn effect_node_on_deserializes_from_ron() {
        let ron_str = "On(target: Bolt, then: [Do(LoseLife)])";
        let node: EffectNode =
            ron::de::from_str(ron_str).expect("EffectNode On RON should parse");
        assert_eq!(
            node,
            EffectNode::On {
                target: Target::Bolt,
                then: vec![EffectNode::Do(Effect::LoseLife)],
            }
        );
    }

    #[test]
    fn effect_node_on_converts_from_root_effect() {
        let root = RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::BoltLost,
                then: vec![EffectNode::Do(Effect::LoseLife)],
            }],
        };
        let node: EffectNode = root.into();
        assert_eq!(
            node,
            EffectNode::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::BoltLost,
                    then: vec![EffectNode::Do(Effect::LoseLife)],
                }],
            }
        );
    }

    // =========================================================================
    // RootEffect — construction and serde (Part B)
    // =========================================================================

    #[test]
    fn root_effect_on_deserializes_from_ron() {
        let ron_str =
            "On(target: Breaker, then: [When(trigger: OnBoltLost, then: [Do(LoseLife)])])";
        let root: RootEffect =
            ron::de::from_str(ron_str).expect("RootEffect On RON should parse");
        match &root {
            RootEffect::On { target, then } => {
                assert_eq!(*target, Target::Breaker);
                assert_eq!(then.len(), 1);
                assert!(matches!(
                    &then[0],
                    EffectNode::When {
                        trigger: Trigger::BoltLost,
                        ..
                    }
                ));
            }
        }
    }

    #[test]
    fn root_effect_rejects_non_on_variant() {
        let ron_str = "When(trigger: OnPerfectBump, then: [Do(LoseLife)])";
        let result = ron::de::from_str::<RootEffect>(ron_str);
        assert!(
            result.is_err(),
            "RootEffect should reject non-On variants like When"
        );
    }

    // =========================================================================
    // Target expansion — new variants (Part C)
    // =========================================================================

    #[test]
    fn target_cell_deserializes() {
        let t: Target = ron::de::from_str("Cell").expect("Target::Cell RON should parse");
        assert_eq!(t, Target::Cell);
    }

    #[test]
    fn target_wall_deserializes() {
        let t: Target = ron::de::from_str("Wall").expect("Target::Wall RON should parse");
        assert_eq!(t, Target::Wall);
    }

    #[test]
    fn target_all_cells_deserializes() {
        let t: Target =
            ron::de::from_str("AllCells").expect("Target::AllCells RON should parse");
        assert_eq!(t, Target::AllCells);
    }
}
