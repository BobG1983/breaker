//! Wave 4C — `afterimage_spawn_phantom_breaker` builder-migration tests.
//!
//! Pins the contract that the afterimage spawn path uses the builder API
//! (not the raw `commands.spawn` tuple). These tests RED-fail against the
//! unmigrated production code:
//!   - Behavior 1 fails because the raw spawn doesn't insert `Breaker`,
//!     `Lifespan`, `PhantomFlicker`, mesh/material, draw layer, bump stats,
//!     collision layers, or `ExtraBreaker`.
//!   - Behavior 2 fails because `BumpState` / `CollisionLayers` are absent
//!     on the raw-spawn phantom (the builder-only marker sub-assertion).
//!   - Behavior 3 fails because the raw spawn doesn't insert
//!     `MeshMaterial2d<ColorMaterial>` (unwrap panics).
//!   - Behavior 5 fails because the raw spawn doesn't insert `Lifespan`.
//!
//! Behaviors 4, 6, 7, 8 are regression guards that already pass against the
//! current system — they fire if the migration drops an existing invariant.

mod component_set;
mod spawn_lifecycle;

use bevy::{ecs::system::SystemState, prelude::*};

use super::super::system::AfterimageConfig;
use crate::{
    breaker::{components::DashState, definition::BreakerDefinition},
    prelude::*,
};

// ── Local fixture ────────────────────────────────────────────────────────────

pub(super) type BuilderSpawnState<'w, 's> = SystemState<(
    Commands<'w, 's>,
    ResMut<'w, Assets<Mesh>>,
    ResMut<'w, Assets<ColorMaterial>>,
)>;

pub(super) fn test_breaker_definition() -> BreakerDefinition {
    ron::de::from_str(
        r#"(
            name: "TestBreaker",
            salvo_hit: Stamp(Breaker, When(Impacted(Salvo), Fire(LoseLife(())))),
            effects: [],
        )"#,
    )
    .expect("test RON should parse")
}

/// Spawns the real breaker via the builder (Rendered + Primary) using the
/// canonical `test_breaker_definition()` — default dimensions `120.0 / 20.0`.
///
/// Returns the spawned entity ID. Must be called after `init_asset::<Mesh>()`
/// and `init_asset::<ColorMaterial>()`.
pub(super) fn spawn_real_breaker_via_builder(app: &mut App) -> Entity {
    let world = app.world_mut();
    let mut state: BuilderSpawnState = SystemState::new(world);
    let (mut commands, mut meshes, mut materials) = state.get_mut(world);
    let entity = Breaker::builder()
        .definition(&test_breaker_definition())
        .rendered(&mut meshes, &mut materials)
        .primary()
        .spawn(&mut commands);
    state.apply(world);
    entity
}

/// Overrides the `AfterimageConfig` to `phantom_duration: 2.5` so the
/// `Lifespan { remaining: 2.5 }` assertion pins the specific value and
/// cannot be fooled by a hardcoded `2.0` from the canonical config.
pub(super) fn override_config_to_2_5(app: &mut App) {
    app.insert_resource(AfterimageConfig {
        phantom_duration:      2.5,
        phantom_bolt_duration: 1.0,
    });
}

/// Drives the rising edge: ticks once (records `DashState::Idle`), mutates
/// the breaker to `DashState::Dashing`, then ticks again (rising edge fires,
/// phantom spawned). Returns the real breaker entity.
pub(super) fn drive_rising_edge(app: &mut App, real: Entity) {
    tick(app);
    *app.world_mut().get_mut::<DashState>(real).unwrap() = DashState::Dashing;
    tick(app);
}
