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
    /// Temporary breaker protection (charge-based).
    Shield {
        /// Current stack count (becomes charge count).
        stacks: u32,
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
        /// Damage multiplier.
        damage_mult: f32,
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
    },
}

impl EffectKind {
    /// Fire this effect on the given entity. Dispatches to the per-module `fire()` function.
    pub(crate) fn fire(&self, entity: Entity, source_chip: &str, world: &mut World) {
        match self {
            Self::Shockwave {
                base_range,
                range_per_level,
                stacks,
                speed,
            } => super::super::super::effects::shockwave::fire(
                entity,
                *base_range,
                *range_per_level,
                *stacks,
                *speed,
                source_chip,
                world,
            ),
            Self::SpeedBoost { multiplier } => {
                super::super::super::effects::speed_boost::fire(
                    entity,
                    *multiplier,
                    source_chip,
                    world,
                );
            }
            Self::DamageBoost(v) => {
                super::super::super::effects::damage_boost::fire(entity, *v, source_chip, world);
            }
            Self::Piercing(v) => {
                super::super::super::effects::piercing::fire(entity, *v, source_chip, world);
            }
            Self::SizeBoost(v) => {
                super::super::super::effects::size_boost::fire(entity, *v, source_chip, world);
            }
            Self::BumpForce(v) => {
                super::super::super::effects::bump_force::fire(entity, *v, source_chip, world);
            }
            Self::Attraction {
                attraction_type,
                force,
                max_force,
            } => {
                super::super::super::effects::attraction::fire(
                    entity,
                    *attraction_type,
                    *force,
                    *max_force,
                    source_chip,
                    world,
                );
            }
            Self::LoseLife => {
                super::super::super::effects::life_lost::fire(entity, source_chip, world);
            }
            Self::TimePenalty { seconds } => {
                super::super::super::effects::time_penalty::fire(
                    entity,
                    *seconds,
                    source_chip,
                    world,
                );
            }
            Self::SpawnBolts {
                count,
                lifespan,
                inherit,
            } => {
                super::super::super::effects::spawn_bolts::fire(
                    entity,
                    *count,
                    *lifespan,
                    *inherit,
                    source_chip,
                    world,
                );
            }
            Self::ChainBolt { tether_distance } => {
                super::super::super::effects::chain_bolt::fire(
                    entity,
                    *tether_distance,
                    source_chip,
                    world,
                );
            }
            _ => self.fire_aoe_and_spawn(entity, source_chip, world),
        }
    }

    /// Fire AOE, spawn, and utility effects -- extracted from [`fire`] for line count.
    fn fire_aoe_and_spawn(&self, entity: Entity, source_chip: &str, world: &mut World) {
        match self {
            Self::Shield { stacks } => {
                super::super::super::effects::shield::fire(entity, *stacks, source_chip, world);
            }
            Self::ChainLightning {
                arcs,
                range,
                damage_mult,
                arc_speed,
            } => super::super::super::effects::chain_lightning::fire(
                entity,
                *arcs,
                *range,
                *damage_mult,
                *arc_speed,
                source_chip,
                world,
            ),
            Self::PiercingBeam { damage_mult, width } => {
                super::super::super::effects::piercing_beam::fire(
                    entity,
                    *damage_mult,
                    *width,
                    source_chip,
                    world,
                );
            }
            Self::Pulse {
                base_range,
                range_per_level,
                stacks,
                speed,
                interval,
            } => super::super::super::effects::pulse::fire(
                entity,
                super::super::super::effects::pulse::PulseEmitter {
                    base_range: *base_range,
                    range_per_level: *range_per_level,
                    stacks: *stacks,
                    speed: *speed,
                    interval: *interval,
                    timer: 0.0,
                },
                source_chip,
                world,
            ),
            Self::SecondWind => {
                super::super::super::effects::second_wind::fire(entity, source_chip, world);
            }
            _ => self.fire_utility_and_spawn(entity, source_chip, world),
        }
    }

    /// Fire utility, random, and spawn effects -- extracted from [`fire_aoe_and_spawn`] for line count.
    fn fire_utility_and_spawn(&self, entity: Entity, source_chip: &str, world: &mut World) {
        match self {
            Self::SpawnPhantom {
                duration,
                max_active,
            } => super::super::super::effects::spawn_phantom::fire(
                entity,
                *duration,
                *max_active,
                source_chip,
                world,
            ),
            Self::GravityWell {
                strength,
                duration,
                radius,
                max,
            } => super::super::super::effects::gravity_well::fire(
                entity,
                *strength,
                *duration,
                *radius,
                *max,
                source_chip,
                world,
            ),
            Self::RandomEffect(pool) => {
                super::super::super::effects::random_effect::fire(entity, pool, source_chip, world);
            }
            Self::EntropyEngine { max_effects, pool } => {
                super::super::super::effects::entropy_engine::fire(
                    entity,
                    *max_effects,
                    pool,
                    source_chip,
                    world,
                );
            }
            Self::RampingDamage { damage_per_trigger } => {
                super::super::super::effects::ramping_damage::fire(
                    entity,
                    *damage_per_trigger,
                    source_chip,
                    world,
                );
            }
            Self::Explode { range, damage_mult } => {
                super::super::super::effects::explode::fire(
                    entity,
                    *range,
                    *damage_mult,
                    source_chip,
                    world,
                );
            }
            Self::QuickStop { multiplier } => {
                super::super::super::effects::quick_stop::fire(
                    entity,
                    *multiplier,
                    source_chip,
                    world,
                );
            }
            Self::TetherBeam { damage_mult } => {
                super::super::super::effects::tether_beam::fire(
                    entity,
                    *damage_mult,
                    source_chip,
                    world,
                );
            }
            _ => {
                // Stat effects (SpeedBoost, DamageBoost, etc.) handled in primary fire() match.
                // If this arm is reached with an unhandled variant, it's a programmer error.
            }
        }
    }

    /// Reverse this effect on the given entity. Dispatches to the per-module `reverse()` function.
    pub(crate) fn reverse(&self, entity: Entity, source_chip: &str, world: &mut World) {
        match self {
            Self::Shockwave { .. } => {
                super::super::super::effects::shockwave::reverse(entity, source_chip, world);
            }
            Self::SpeedBoost { multiplier } => {
                super::super::super::effects::speed_boost::reverse(
                    entity,
                    *multiplier,
                    source_chip,
                    world,
                );
            }
            Self::DamageBoost(v) => {
                super::super::super::effects::damage_boost::reverse(entity, *v, source_chip, world);
            }
            Self::Piercing(v) => {
                super::super::super::effects::piercing::reverse(entity, *v, source_chip, world);
            }
            Self::SizeBoost(v) => {
                super::super::super::effects::size_boost::reverse(entity, *v, source_chip, world);
            }
            Self::BumpForce(v) => {
                super::super::super::effects::bump_force::reverse(entity, *v, source_chip, world);
            }
            Self::Attraction {
                attraction_type,
                force,
                max_force,
            } => {
                super::super::super::effects::attraction::reverse(
                    entity,
                    *attraction_type,
                    *force,
                    *max_force,
                    source_chip,
                    world,
                );
            }
            Self::LoseLife => {
                super::super::super::effects::life_lost::reverse(entity, source_chip, world);
            }
            Self::TimePenalty { seconds } => {
                super::super::super::effects::time_penalty::reverse(
                    entity,
                    *seconds,
                    source_chip,
                    world,
                );
            }
            _ => self.reverse_aoe_and_spawn(entity, source_chip, world),
        }
    }

    /// Reverse AOE, spawn, and utility effects -- extracted from [`reverse`] for line count.
    fn reverse_aoe_and_spawn(&self, entity: Entity, source_chip: &str, world: &mut World) {
        match self {
            Self::SpawnBolts {
                count,
                lifespan,
                inherit,
            } => super::super::super::effects::spawn_bolts::reverse(
                entity,
                *count,
                *lifespan,
                *inherit,
                source_chip,
                world,
            ),
            Self::ChainBolt { tether_distance } => {
                super::super::super::effects::chain_bolt::reverse(
                    entity,
                    *tether_distance,
                    source_chip,
                    world,
                );
            }
            Self::Shield { .. } => {
                super::super::super::effects::shield::reverse(entity, source_chip, world);
            }
            Self::ChainLightning { .. } => {
                super::super::super::effects::chain_lightning::reverse(entity, source_chip, world);
            }
            Self::PiercingBeam { .. } => {
                super::super::super::effects::piercing_beam::reverse(entity, source_chip, world);
            }
            Self::Pulse { .. } => {
                super::super::super::effects::pulse::reverse(entity, source_chip, world);
            }
            Self::SecondWind => {
                super::super::super::effects::second_wind::reverse(entity, source_chip, world);
            }
            Self::SpawnPhantom { .. } => {
                super::super::super::effects::spawn_phantom::reverse(entity, source_chip, world);
            }
            Self::GravityWell { .. } => {
                super::super::super::effects::gravity_well::reverse(entity, source_chip, world);
            }
            Self::RandomEffect(pool) => {
                super::super::super::effects::random_effect::reverse(
                    entity,
                    pool,
                    source_chip,
                    world,
                );
            }
            Self::EntropyEngine { .. } => {
                super::super::super::effects::entropy_engine::reverse(entity, source_chip, world);
            }
            Self::RampingDamage { .. } => {
                super::super::super::effects::ramping_damage::reverse(entity, source_chip, world);
            }
            Self::Explode { .. } => {
                super::super::super::effects::explode::reverse(entity, source_chip, world);
            }
            Self::QuickStop { multiplier } => {
                super::super::super::effects::quick_stop::reverse(
                    entity,
                    *multiplier,
                    source_chip,
                    world,
                );
            }
            Self::TetherBeam { damage_mult } => {
                super::super::super::effects::tether_beam::reverse(
                    entity,
                    *damage_mult,
                    source_chip,
                    world,
                );
            }
            _ => {}
        }
    }
}

/// Permanent effect trees on an entity. Never consumed by trigger evaluation.
/// Each entry is `(chip_name, node)`.
#[derive(Component, Debug, Default, Clone)]
pub struct BoundEffects(pub Vec<(String, EffectNode)>);

/// Working set of partially-resolved chains. Consumed when matched.
/// Each entry is `(chip_name, node)`.
#[derive(Component, Debug, Default, Clone)]
pub struct StagedEffects(pub Vec<(String, EffectNode)>);
