use bevy::prelude::*;
use breaker::shared::GameState;

/// Statistics collected during a scenario run.
///
/// Inserted by [`ScenarioLifecycle`] at run start. Updated by various systems.
#[derive(Resource, Default, Clone)]
pub struct ScenarioStats {
    /// Total number of actions injected via [`inject_scenario_input`].
    pub actions_injected: u32,
    /// Total number of invariant check evaluations performed.
    pub invariant_checks: u32,
    /// Highest [`ScenarioFrame`] value reached.
    pub max_frame: u32,
    /// Whether [`GameState::Playing`] was entered at least once.
    ///
    /// Defaults to `false`. Set to `true` by `tag_game_entities` on
    /// `OnEnter(Playing)`. Invariant checkers are gated on this flag
    /// to prevent false positives during `GameState::Loading`.
    pub entered_playing: bool,
    /// Number of bolt entities that were tagged with [`ScenarioTagBolt`].
    pub bolts_tagged: u32,
    /// Number of breaker entities that were tagged with [`ScenarioTagBreaker`].
    pub breakers_tagged: u32,
    /// Number of cell entities that were tagged with [`ScenarioTagCell`].
    pub cells_tagged: u32,
    /// Number of wall entities that were tagged with [`ScenarioTagWall`].
    pub walls_tagged: u32,
}

/// Marker — attached by the lifecycle plugin to the bolt entity for invariant checking.
#[derive(Component)]
pub struct ScenarioTagBolt;

/// Marker — attached by the lifecycle plugin to the breaker entity for invariant checking.
#[derive(Component)]
pub struct ScenarioTagBreaker;

/// Marker — attached by the lifecycle plugin to cell entities for invariant checking
/// and deferred effect application.
#[derive(Component)]
pub struct ScenarioTagCell;

/// Marker — attached by the lifecycle plugin to wall entities for invariant checking
/// and deferred effect application.
#[derive(Component)]
pub struct ScenarioTagWall;

/// Tracks the current fixed-update frame number for violation logging.
#[derive(Resource, Default)]
pub struct ScenarioFrame(pub u32);

/// A single invariant violation recorded during a scenario run.
#[derive(Debug, Clone)]
pub struct ViolationEntry {
    /// Fixed-update frame when the violation was detected.
    pub frame: u32,
    /// Which invariant was violated.
    pub invariant: InvariantKind,
    /// Entity involved, if applicable.
    pub entity: Option<Entity>,
    /// Human-readable description with concrete values.
    pub message: String,
}

/// Accumulated violations from all invariant checks.
#[derive(Resource, Default)]
pub struct ViolationLog(pub Vec<ViolationEntry>);

/// Stores the frozen world-space position for an entity with `disable_physics: true`.
///
/// When `ScenarioPhysicsFrozen` is present on an entity, `enforce_frozen_positions`
/// resets the entity's `Position2D` to `target` every tick, preventing physics from
/// moving it.
#[derive(Component)]
pub struct ScenarioPhysicsFrozen {
    /// The world-space position this entity is pinned to each tick.
    pub target: Vec2,
}

/// Tracks the previous [`GameState`] for transition validation.
#[derive(Resource, Default)]
pub struct PreviousGameState(pub Option<GameState>);

/// Baseline entity count for leak detection.
#[derive(Resource, Default)]
pub struct EntityLeakBaseline {
    /// Entity count sampled when [`SpawnNodeComplete`] is received.
    pub baseline: Option<usize>,
}

use crate::types::InvariantKind;
