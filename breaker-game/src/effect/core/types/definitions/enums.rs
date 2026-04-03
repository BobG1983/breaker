//! Core effect system types -- triggers, targets, effect nodes, and effect kinds.

use bevy::prelude::*;
use serde::Deserialize;

/// Deferred chip attribution stored on spawned effect entities.
///
/// Used by shockwave, pulse ring, explode request, chain lightning request,
/// piercing beam request, and tether beam. Damage-application systems read
/// this to populate `DamageCell.source_chip`.
#[derive(Component, Debug, Clone, Default)]
pub struct EffectSourceChip(pub Option<String>);

impl EffectSourceChip {
    /// Create from a chip name: empty string -> `EffectSourceChip(None)`, non-empty -> `Some(owned)`.
    #[must_use]
    pub fn new(source_chip: &str) -> Self {
        Self(chip_attribution(source_chip))
    }

    /// Extract the chip attribution for `DamageCell.source_chip`.
    #[must_use]
    pub fn source_chip(&self) -> Option<String> {
        self.0.clone()
    }
}

/// Convert a `source_chip` string into an `Option<String>` suitable for
/// `DamageCell.source_chip`. Empty string maps to `None`; non-empty maps to
/// `Some(s.to_string())`.
#[must_use]
pub fn chip_attribution(source_chip: &str) -> Option<String> {
    if source_chip.is_empty() {
        None
    } else {
        Some(source_chip.to_string())
    }
}

/// A trigger that gates effect evaluation. Bridge systems fire triggers;
/// `When` nodes match against them.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum Trigger {
    /// Global -- a perfect bump occurred.
    PerfectBump,
    /// Global -- an early bump occurred.
    EarlyBump,
    /// Global -- a late bump occurred.
    LateBump,
    /// Global -- any successful bump (perfect, early, or late).
    Bump,
    /// Global -- bump attempt missed timing window.
    BumpWhiff,
    /// Global -- bolt hit breaker with no bump input.
    NoBump,
    /// Targeted on bolt -- bolt was in a perfect bump.
    PerfectBumped,
    /// Targeted on bolt -- bolt was in an early bump.
    EarlyBumped,
    /// Targeted on bolt -- bolt was in a late bump.
    LateBumped,
    /// Targeted on bolt -- bolt was in any successful bump.
    Bumped,
    /// Global -- an impact involving the specified entity type occurred.
    Impact(ImpactTarget),
    /// Targeted on both participants -- you were in an impact with the specified type.
    Impacted(ImpactTarget),
    /// Global -- something died.
    Death,
    /// Targeted -- this entity died.
    Died,
    /// Global -- a bolt was lost.
    BoltLost,
    /// Global -- a cell was destroyed.
    CellDestroyed,
    /// Global -- a new node started.
    NodeStart,
    /// Global -- the current node ended.
    NodeEnd,
    /// Global -- node timer crossed a ratio threshold.
    NodeTimerThreshold(f32),
    /// Special -- timer system ticks this down; fires when remaining reaches zero.
    TimeExpires(f32),
}

/// Entity type involved in a collision, used by [`Trigger::Impact`] and [`Trigger::Impacted`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum ImpactTarget {
    /// A cell entity.
    Cell,
    /// A bolt entity.
    Bolt,
    /// A wall entity.
    Wall,
    /// The breaker entity.
    Breaker,
}

/// Target entity for effect dispatch via [`RootEffect::On`] and [`EffectNode::On`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum Target {
    /// The primary bolt entity (singular at runtime, all bolts at dispatch).
    Bolt,
    /// All bolt entities (desugared at dispatch time).
    AllBolts,
    /// The breaker entity.
    Breaker,
    /// A single cell entity (context-sensitive at runtime).
    Cell,
    /// All cell entities (desugared at dispatch time).
    AllCells,
    /// A single wall entity (context-sensitive at runtime).
    Wall,
    /// All wall entities (desugared at dispatch time).
    AllWalls,
}

/// Context entities from the trigger event that caused effect evaluation.
///
/// Bridges populate the fields they know about. `On(Cell)` reads `context.cell`,
/// `On(Bolt)` reads `context.bolt`, etc. Fields are `None` when the trigger
/// doesn't involve that entity type.
#[derive(Clone, Copy, Debug, Default)]
pub struct TriggerContext {
    /// The bolt entity involved in this trigger, if any.
    pub bolt: Option<Entity>,
    /// The breaker entity involved in this trigger, if any.
    pub breaker: Option<Entity>,
    /// The cell entity involved in this trigger, if any.
    pub cell: Option<Entity>,
    /// The wall entity involved in this trigger, if any.
    pub wall: Option<Entity>,
}

/// Type of entity to attract toward, used by [`EffectKind::Attraction`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum AttractionType {
    /// Attract toward nearest cell.
    Cell,
    /// Attract toward nearest wall.
    Wall,
    /// Attract toward the breaker.
    Breaker,
}

/// Top-level effect wrapper for RON definitions. Ensures every effect chain
/// names its target entity.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum RootEffect {
    /// Dispatch effects to a target entity.
    On {
        /// The target entity type.
        target: Target,
        /// Children to dispatch.
        then: Vec<EffectNode>,
    },
}

impl From<RootEffect> for EffectNode {
    fn from(r: RootEffect) -> Self {
        let RootEffect::On { target, then } = r;
        Self::On {
            target,
            permanent: false,
            then,
        }
    }
}

/// A node in the effect tree. Evaluated by trigger bridge systems and the
/// until desugaring system.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum EffectNode {
    /// Gate -- evaluate children only if trigger matches.
    When {
        /// The trigger to match against.
        trigger: Trigger,
        /// Children to evaluate on match.
        then: Vec<Self>,
    },
    /// Terminal -- fire the effect on the entity.
    Do(EffectKind),
    /// One-shot -- evaluate children; if any match, consume the `Once`.
    Once(Vec<Self>),
    /// Redirect -- transfer children to the target entity.
    On {
        /// Target entity to transfer to.
        target: Target,
        /// If true, non-`Do` children go to `BoundEffects`; otherwise `StagedEffects` (except during dispatch, where they ALWAYS go to `BoundEffects`).
        #[serde(default)]
        permanent: bool,
        /// Children to transfer.
        then: Vec<Self>,
    },
    /// Duration-scoped -- apply effects now, undo when trigger fires.
    Until {
        /// The trigger that ends the duration.
        trigger: Trigger,
        /// Effects to apply (`Do`) and chains to install (non-`Do`).
        then: Vec<Self>,
    },
    /// Internal -- created by `Until` desugaring. Carries reversal data.
    Reverse {
        /// Effects that were fired and need reversing.
        effects: Vec<EffectKind>,
        /// Chains that were pushed to `BoundEffects` and need removing.
        chains: Vec<Self>,
    },
}

/// Serde default helper for [`EffectKind::SpawnBolts::count`].
const fn one() -> u32 {
    1
}

/// Serde default helper for [`EffectKind::Pulse::interval`].
const fn default_pulse_interval() -> f32 {
    0.5
}

/// Serde default helper for [`EffectKind::ChainLightning::arc_speed`].
const fn default_chain_lightning_arc_speed() -> f32 {
    200.0
}

/// The action an effect performs. Each variant maps to a per-module `fire()`
/// and `reverse()` function.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum EffectKind {
    /// Expanding ring of area damage.
    Shockwave {
        /// Base radius before stacking.
        base_range: f32,
        /// Extra radius per stack beyond the first.
        range_per_level: f32,
        /// Current stack count.
        stacks: u32,
        /// Expansion speed in world units/sec.
        speed: f32,
    },
    /// Multiplicative speed scaling.
    SpeedBoost {
        /// Speed multiplier (1.x format).
        multiplier: f32,
    },
    /// Multiplicative damage bonus.
    DamageBoost(f32),
    /// Pass through cells instead of bouncing.
    Piercing(u32),
    /// Size increase (bolt radius or breaker width).
    SizeBoost(f32),
    /// Flat bump force increase.
    BumpForce(f32),
    /// Steer toward nearest entity of a type.
    Attraction {
        /// Which entity type to attract toward.
        attraction_type: AttractionType,
        /// Attraction strength.
        force: f32,
        /// Optional maximum force magnitude per tick. Clamps velocity delta.
        #[serde(default)]
        max_force: Option<f32>,
    },
    /// Decrement lives.
    LoseLife,
    /// Subtract time from node timer.
    TimePenalty {
        /// Seconds to subtract.
        seconds: f32,
    },
    /// Spawn additional bolts.
    SpawnBolts {
        /// Number of bolts to spawn.
        #[serde(default = "one")]
        count: u32,
        /// Optional lifespan in seconds.
        #[serde(default)]
        lifespan: Option<f32>,
        /// If true, spawned bolts inherit parent's `BoundEffects`.
        #[serde(default)]
        inherit: bool,
    },
    /// Spawn two bolts chained together via distance constraint.
    ChainBolt {
        /// Maximum distance between the two chained bolts.
        tether_distance: f32,
    },
    /// Temporary visible floor wall (timed).
    Shield {
        /// Duration in seconds before the wall despawns.
        duration: f32,
        /// Time subtracted from `ShieldWallTimer` per bolt reflection.
        #[serde(default)]
        reflection_cost: f32,
    },
    /// Arc damage jumping between nearby cells.
    ChainLightning {
        /// Number of jumps.
        arcs: u32,
        /// Maximum jump distance.
        range: f32,
        /// Damage multiplier per arc.
        damage_mult: f32,
        /// Arc travel speed in world units per second.
        #[serde(default = "default_chain_lightning_arc_speed")]
        arc_speed: f32,
    },
    /// Beam through cells in velocity direction.
    PiercingBeam {
        /// Damage multiplier.
        damage_mult: f32,
        /// Beam width in world units.
        width: f32,
    },
    /// Shockwave at every active bolt position.
    Pulse {
        /// Base radius per shockwave.
        base_range: f32,
        /// Extra radius per stack.
        range_per_level: f32,
        /// Current stack count.
        stacks: u32,
        /// Expansion speed.
        speed: f32,
        /// Seconds between ring emissions.
        #[serde(default = "default_pulse_interval")]
        interval: f32,
    },
    /// Invisible bottom wall that bounces bolt once.
    SecondWind,
    /// Breaker dash teleport on reverse-direction input.
    FlashStep,
    /// Temporary phantom bolt with infinite piercing.
    SpawnPhantom {
        /// Lifespan in seconds.
        duration: f32,
        /// Maximum phantoms alive at once.
        max_active: u32,
    },
    /// Gravity well that attracts bolts within radius.
    GravityWell {
        /// Pull strength.
        strength: f32,
        /// Duration in seconds.
        duration: f32,
        /// Attraction radius.
        radius: f32,
        /// Maximum active wells.
        max: u32,
    },
    /// Weighted random selection from a pool.
    RandomEffect(Vec<(f32, EffectNode)>),
    /// Escalating chaos -- fires multiple random effects per cell destroyed.
    EntropyEngine {
        /// Maximum effects fired per cell destroyed.
        max_effects: u32,
        /// Weighted pool of effects to choose from.
        pool: Vec<(f32, EffectNode)>,
    },
    /// Stacking damage bonus on consecutive cell hits.
    RampingDamage {
        /// Damage bonus added per trigger activation.
        damage_per_trigger: f32,
    },
    /// Instant area damage burst.
    Explode {
        /// Blast radius.
        range: f32,
        /// Flat damage dealt to each cell in range.
        damage: f32,
    },
    /// Breaker deceleration multiplier for precise stops.
    QuickStop {
        /// Deceleration multiplier (1.x format: 2.0 = 2x faster deceleration).
        multiplier: f32,
    },
    /// Two free-moving bolts connected by a damaging beam.
    TetherBeam {
        /// Damage multiplier for beam contact (1.x format).
        damage_mult: f32,
        /// If true, chain mode connects all existing bolts instead of spawning new ones.
        #[serde(default)]
        chain: bool,
    },
    /// Spawn a mirrored bolt reflected across the last impact surface.
    MirrorProtocol {
        /// If true, spawned bolt inherits parent's `BoundEffects`.
        #[serde(default)]
        inherit: bool,
    },
    /// Breaker plants after stationary delay, modifying bump behavior.
    Anchor {
        /// Bump force multiplier when planted.
        bump_force_multiplier: f32,
        /// Perfect window multiplier when planted.
        perfect_window_multiplier: f32,
        /// Seconds breaker must remain stationary before planting.
        plant_delay: f32,
    },
    /// Marks an entity as vulnerable, increasing damage taken.
    ///
    /// Applied to cells. The damage system multiplies incoming damage by
    /// `multiplier` when the cell has an active vulnerability.
    Vulnerable {
        /// Damage multiplier applied to incoming hits (e.g., 2.0 = double damage).
        multiplier: f32,
    },
    /// Charge-and-release: count bumps, then spawn bolts + shockwave.
    CircuitBreaker {
        /// Number of bumps required per cycle.
        bumps_required: u32,
        /// Number of extra bolts to spawn on reward.
        #[serde(default = "one")]
        spawn_count: u32,
        /// Whether spawned bolts inherit parent's `BoundEffects`.
        #[serde(default)]
        inherit: bool,
        /// Shockwave maximum radius.
        shockwave_range: f32,
        /// Shockwave expansion speed.
        shockwave_speed: f32,
    },
}

/// Permanent effect trees on an entity. Never consumed by trigger evaluation.
/// Each entry is `(chip_name, node)`.
#[derive(Component, Debug, Default, Clone)]
pub struct BoundEffects(pub Vec<(String, EffectNode)>);

/// Working set of partially-resolved chains. Consumed when matched.
/// Each entry is `(chip_name, node)`.
#[derive(Component, Debug, Default, Clone)]
pub struct StagedEffects(pub Vec<(String, EffectNode)>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_root_effect_on_for_effect_node_sets_permanent_false() {
        let root = RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
        };

        let node = EffectNode::from(root);

        assert_eq!(
            node,
            EffectNode::On {
                target: Target::Bolt,
                permanent: false,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
            },
            "From<RootEffect> should produce On with permanent=false"
        );
    }

    #[test]
    fn from_root_effect_on_all_cells_with_empty_then() {
        let root = RootEffect::On {
            target: Target::AllCells,
            then: vec![],
        };

        let node = EffectNode::from(root);

        assert_eq!(
            node,
            EffectNode::On {
                target: Target::AllCells,
                permanent: false,
                then: vec![],
            },
            "From<RootEffect> with AllCells and empty then should produce On with permanent=false"
        );
    }

    #[test]
    fn from_root_effect_on_breaker_with_nested_children_preserves_structure() {
        let nested_children = vec![EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }];
        let root = RootEffect::On {
            target: Target::Breaker,
            then: nested_children.clone(),
        };

        let node = EffectNode::from(root);

        assert_eq!(
            node,
            EffectNode::On {
                target: Target::Breaker,
                permanent: false,
                then: nested_children,
            },
            "From<RootEffect> with Breaker should preserve nested children with permanent=false"
        );
    }

    // -- TetherBeam chain field serde tests --

    #[test]
    fn tether_beam_serde_with_chain_true() {
        let ron_str = "TetherBeam(damage_mult: 1.5, chain: true)";
        let effect: EffectKind =
            ron::from_str(ron_str).expect("should deserialize TetherBeam with chain: true");

        match &effect {
            EffectKind::TetherBeam { damage_mult, chain } => {
                assert!(
                    (*damage_mult - 1.5).abs() < f32::EPSILON,
                    "expected damage_mult 1.5, got {damage_mult}"
                );
                assert!(*chain, "expected chain true, got {chain}");
            }
            other => panic!("expected TetherBeam variant, got {other:?}"),
        }
    }

    #[test]
    fn tether_beam_serde_defaults_chain_to_false_when_omitted() {
        let ron_str = "TetherBeam(damage_mult: 2.0)";
        let effect: EffectKind =
            ron::from_str(ron_str).expect("should deserialize TetherBeam with omitted chain");

        match &effect {
            EffectKind::TetherBeam { damage_mult, chain } => {
                assert!(
                    (*damage_mult - 2.0).abs() < f32::EPSILON,
                    "expected damage_mult 2.0, got {damage_mult}"
                );
                assert!(!*chain, "expected chain false (serde default), got {chain}");
            }
            other => panic!("expected TetherBeam variant, got {other:?}"),
        }
    }
}
