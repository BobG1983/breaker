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
    /// Targeted impact trigger — fires on the impacted entity (cell, wall, breaker).
    Impacted(ImpactTarget),
    /// Targeted death trigger — fires on the entity that died.
    Died,
    /// Targeted cell-destroyed trigger — fires on the destroyed cell entity.
    DestroyedCell,
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
        /// Base radius before stacking.
        base_range: f32,
        /// Extra radius per stack.
        range_per_level: f32,
        /// Current stack count (starts at 1, incremented at runtime).
        stacks: u32,
        /// Expansion speed (world units/sec).
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
        /// Max distance from anchor bolt.
        tether_distance: f32,
    },
    /// Spawns additional bolts on trigger.
    MultiBolt {
        /// Base bolt count before stacking.
        base_count: u32,
        /// Extra bolts per stack.
        count_per_level: u32,
        /// Stack count (starts at 1, incremented at runtime).
        stacks: u32,
    },
    /// Temporary shield protecting the breaker.
    Shield {
        /// Base duration (seconds).
        base_duration: f32,
        /// Extra duration per stack (seconds).
        duration_per_level: f32,
        /// Stack count (starts at 1, incremented at runtime).
        stacks: u32,
    },
    /// Deducts a life from the breaker.
    LoseLife,
    /// Applies a time penalty in seconds.
    TimePenalty {
        /// Penalty duration (seconds).
        seconds: f32,
    },
    /// Spawns additional bolts with configurable parameters.
    SpawnBolts {
        /// Bolt spawn count.
        #[serde(default = "default_spawn_bolts_count")]
        count: u32,
        /// Optional lifespan (seconds); `None` = permanent.
        #[serde(default)]
        lifespan: Option<f32>,
        /// Inherit parent bolt velocity.
        #[serde(default)]
        inherit: bool,
    },
    /// Chain lightning arcing between nearby cells.
    ChainLightning {
        /// Arc count from origin cell.
        arcs: u32,
        /// Max arc range (world units).
        range: f32,
        /// Damage multiplier per arc (applied to base bolt damage).
        damage_mult: f32,
    },
    /// Spawns a temporary phantom breaker entity.
    SpawnPhantom {
        /// Duration (seconds).
        duration: f32,
        /// Max simultaneous instances.
        max_active: u32,
    },
    /// Fires a piercing beam through cells in a line.
    PiercingBeam {
        /// Damage multiplier per arc.
        damage_mult: f32,
        /// Beam width (world units).
        width: f32,
    },
    /// Creates a gravity well that attracts bolts.
    GravityWell {
        /// Pull force magnitude.
        strength: f32,
        /// Duration (seconds).
        duration: f32,
        /// Effect radius (world units).
        radius: f32,
        /// Maximum active wells at once.
        max: u32,
    },
    /// Temporary invulnerability after bolt loss.
    SecondWind {
        /// Invulnerability duration (seconds).
        invuln_secs: f32,
    },
    /// Ramping damage bonus that accumulates per cell hit and resets on breaker bounce.
    RampingDamage {
        /// Damage bonus per cell hit.
        bonus_per_hit: f32,
    },
    /// Selects a random effect from a weighted pool of `EffectNode` entries.
    RandomEffect(Vec<(f32, EffectNode)>),
    /// Counts cell destructions and fires a random effect from the pool when threshold reached.
    EntropyEngine {
        /// Cell destructions before firing.
        threshold: u32,
        /// Weighted pool of effects to choose from.
        pool: Vec<(f32, EffectNode)>,
    },
    /// Shockwave at every active bolt position simultaneously.
    Pulse {
        /// Base radius before stacking.
        base_range: f32,
        /// Extra radius per stack.
        range_per_level: f32,
        /// Stack count (starts at 1, incremented at runtime).
        stacks: u32,
        /// Expansion speed (world units/sec).
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
        /// Condition that gates evaluation.
        trigger: Trigger,
        /// Child nodes evaluated when the trigger fires.
        then: Vec<EffectNode>,
    },
    /// A terminal effect action.
    Do(Effect),
    /// A removal condition — child effects are active until the trigger fires.
    Until {
        /// Condition that removes the child effects.
        until: Trigger,
        /// Child effects active until the removal trigger fires.
        then: Vec<EffectNode>,
    },
    /// A one-shot wrapper — children fire once and are consumed.
    Once(Vec<EffectNode>),
    /// A target scope — children are dispatched against the specified target entity.
    ///
    /// `On` nodes are not evaluated by trigger matching; they are resolved at
    /// dispatch time to determine the entity context for child effects.
    On {
        /// Entity type this scope targets.
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
        /// Entity type this chain targets.
        target: Target,
        /// Effect nodes in this chain.
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
    #[must_use]
    pub fn test_shockwave(range: f32) -> Self {
        Self::Shockwave {
            base_range: range,
            range_per_level: 0.0,
            stacks: 1,
            speed: 400.0,
        }
    }

    /// Build a `MultiBolt` leaf with `count_per_level: 0` and `stacks: 1`.
    #[must_use]
    pub fn test_multi_bolt(count: u32) -> Self {
        Self::MultiBolt {
            base_count: count,
            count_per_level: 0,
            stacks: 1,
        }
    }

    /// Build a `Shield` leaf with `duration_per_level: 0.0` and `stacks: 1`.
    #[must_use]
    pub fn test_shield(duration: f32) -> Self {
        Self::Shield {
            base_duration: duration,
            duration_per_level: 0.0,
            stacks: 1,
        }
    }

    /// Build a `LoseLife` leaf.
    #[must_use]
    pub fn test_lose_life() -> Self {
        Self::LoseLife
    }

    /// Build a `TimePenalty` leaf with the given duration in seconds.
    #[must_use]
    pub fn test_time_penalty(seconds: f32) -> Self {
        Self::TimePenalty { seconds }
    }

    /// Build a `SpawnBolts` leaf with default parameters.
    #[must_use]
    pub fn test_spawn_bolts() -> Self {
        Self::SpawnBolts {
            count: 1,
            lifespan: None,
            inherit: false,
        }
    }

    /// Build a `SpeedBoost` leaf with the given multiplier.
    #[must_use]
    pub fn test_speed_boost(multiplier: f32) -> Self {
        Self::SpeedBoost { multiplier }
    }

    /// Build a `Piercing` leaf with the given count.
    #[must_use]
    pub fn test_piercing(count: u32) -> Self {
        Self::Piercing(count)
    }

    /// Build a `DamageBoost` leaf with the given boost value.
    #[must_use]
    pub fn test_damage_boost(boost: f32) -> Self {
        Self::DamageBoost(boost)
    }

    /// Build a `RampingDamage` leaf with the given per-hit bonus.
    #[must_use]
    pub fn test_ramping_damage(bonus_per_hit: f32) -> Self {
        Self::RampingDamage { bonus_per_hit }
    }

    /// Build a `RandomEffect` variant with the given pool.
    #[must_use]
    pub fn test_random_effect(pool: Vec<(f32, EffectNode)>) -> Self {
        Self::RandomEffect(pool)
    }

    /// Build an `EntropyEngine` variant with the given threshold and pool.
    #[must_use]
    pub fn test_entropy_engine(threshold: u32, pool: Vec<(f32, EffectNode)>) -> Self {
        Self::EntropyEngine { threshold, pool }
    }

    /// Build a `Pulse` leaf with `range_per_level: 0.0`, `stacks: 1`, and `speed: 400.0`.
    #[must_use]
    pub fn test_pulse(range: f32) -> Self {
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
    #[must_use]
    pub fn trigger_leaf(trigger: Trigger, effect: Effect) -> Self {
        Self::When {
            trigger,
            then: vec![Self::Do(effect)],
        }
    }
}

// ---------------------------------------------------------------------------
// Breaker definition re-exports (canonical location: breaker/definition.rs)
// ---------------------------------------------------------------------------

pub use crate::breaker::definition::{BreakerDefinition, BreakerStatOverrides};
