//! Query aliases, resources, constants, and sentinel types for the scenario lifecycle.

use bevy::prelude::*;
use breaker::{
    breaker::components::BaseWidth,
    effect_v3::types::Tree,
    state::run::{NodeLayout, node::definition::NodePool},
};
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use crate::invariants::{ScenarioTagBolt, ScenarioTagBreaker};

/// Query alias for bolt entities in [`super::debug_setup::apply_debug_setup`] and
/// [`super::debug_setup::deferred_debug_setup`].
pub type BoltDebugQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static mut Position2D, &'static mut Velocity2D),
    With<ScenarioTagBolt>,
>;

/// Query alias for breaker entities in [`super::debug_setup::apply_debug_setup`] and
/// [`super::debug_setup::deferred_debug_setup`].
pub type BreakerDebugQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static mut Position2D),
    (With<ScenarioTagBreaker>, Without<ScenarioTagBolt>),
>;

/// Query alias for breaker entities in [`super::perfect_tracking::apply_perfect_tracking`].
pub type BreakerTrackingQuery<'w, 's> = Query<
    'w,
    's,
    (&'static mut Position2D, &'static BaseWidth),
    (With<ScenarioTagBreaker>, Without<ScenarioTagBolt>),
>;

/// Loaded scenario configuration, inserted before the app runs.
#[derive(Resource)]
pub struct ScenarioConfig {
    /// The full scenario definition loaded from RON.
    pub definition: crate::types::ScenarioDefinition,
}

/// Bevy resource wrapping an [`crate::input::InputDriver`] for the current scenario run.
///
/// Inserted by [`super::input::init_scenario_input`] when the scenario starts.
/// Queried each `FixedPreUpdate` tick by [`super::input::inject_scenario_input`].
#[derive(Resource)]
pub struct ScenarioInputDriver(pub crate::input::InputDriver);

/// Tracks which `chip_selections` entry to use next.
///
/// Reset to `0` by [`super::menu_bypass::bypass_menu_to_playing`] on each run restart.
#[derive(Resource, Default)]
pub struct ChipSelectionIndex(pub usize);

/// Holds bolt-targeted initial effects until bolt entities are spawned and tagged.
///
/// Inserted by [`super::menu_bypass::bypass_menu_to_playing`] when `initial_effects` contains a
/// `Target::Bolt` entry. Applied once by [`super::pending_effects::apply_pending_bolt_effects`]
/// after tagged bolt entities exist; cleared after application.
#[derive(Resource, Default)]
pub struct PendingBoltEffects(pub Vec<(String, Tree)>);

/// Holds cell-targeted initial effects until cell entities are spawned and tagged.
///
/// Inserted by [`super::menu_bypass::bypass_menu_to_playing`] when `initial_effects` contains a
/// `Target::Cell` or `Target::AllCells` entry. Applied once by
/// [`super::pending_effects::apply_pending_cell_effects`] after tagged cell entities exist; cleared
/// after application.
#[derive(Resource, Default)]
pub struct PendingCellEffects(pub Vec<(String, Tree)>);

/// Holds wall-targeted initial effects until wall entities are spawned and tagged.
///
/// Inserted by [`super::menu_bypass::bypass_menu_to_playing`] when `initial_effects` contains a
/// `Target::Wall` or `Target::AllWalls` entry. Applied once by
/// [`super::pending_effects::apply_pending_wall_effects`] after tagged wall entities exist; cleared
/// after application.
#[derive(Resource, Default)]
pub struct PendingWallEffects(pub Vec<(String, Tree)>);

/// Holds breaker-targeted initial effects until breaker entities are spawned and tagged.
///
/// Inserted by [`super::menu_bypass::bypass_menu_to_playing`] when `initial_effects` contains a
/// `Target::Breaker` entry. Applied once by
/// [`super::pending_effects::apply_pending_breaker_effects`] after tagged breaker entities exist;
/// cleared after application.
#[derive(Resource, Default)]
pub struct PendingBreakerEffects(pub Vec<(String, Tree)>);

/// Sentinel breaker name for scenarios that need an indestructible breaker.
///
/// Case-sensitive -- RON `breaker` field must match exactly.
pub const GODMODE_BREAKER_SENTINEL: &str = "godmode";

/// Sentinel layout name for scenarios that need a single-cell quick-clear layout.
///
/// Case-sensitive -- RON `layout` field must match exactly.
pub const QUICK_CLEAR_LAYOUT_SENTINEL: &str = "quick_clear";

/// Sentinel layout name for scenarios that need a single-cell Boss-pool layout.
///
/// Identical to `quick_clear` but with `pool: NodePool::Boss`, so the chip
/// selection system treats the node as a boss and offers evolutions.
pub const QUICK_BOSS_LAYOUT_SENTINEL: &str = "quick_boss";

/// Builds an empty layout that clears instantly (no required cells).
///
/// `track_node_completion` fires `NodeCleared` when `ClearRemainingCount` is 0,
/// so an all-empty grid triggers immediate node completion -> `TransitionOut`.
#[must_use]
pub fn quick_clear_layout(pool: NodePool) -> NodeLayout {
    let name = if pool == NodePool::Boss {
        "quick_boss"
    } else {
        "quick_clear"
    };
    NodeLayout {
        name: name.to_owned(),
        timer_secs: 999.0,
        cols: 1,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec![".".to_owned()]],
        pool,
        entity_scale: 1.0,
        locks: None,
        sequences: None,
    }
}

/// Distance threshold (world units) for bolt-breaker proximity to trigger bump.
///
/// Must exceed the bolt spawn offset (default 54) so that a freshly-spawned or
/// respawned bolt (placed at `breaker_y + spawn_offset`) is caught immediately.
/// Also must exceed the bolt-breaker collision distance (breaker half-height +
/// bolt radius ≈ 24) plus one bolt step per tick (≈ 11 at 700 u/s @ 64 Hz)
/// because `apply_perfect_tracking` runs in `FixedPreUpdate` — before the
/// physics systems in `FixedUpdate` move the bolt and resolve collisions.
pub const PERFECT_TRACKING_BUMP_THRESHOLD: f32 = 60.0;

/// Factor of breaker half-width used for random x offset.
pub const PERFECT_TRACKING_WIDTH_FACTOR: f32 = 0.8;
