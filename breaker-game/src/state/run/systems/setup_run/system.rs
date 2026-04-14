//! System to set up the run: spawns primary breaker and primary bolt.

use bevy::{ecs::system::SystemParam, prelude::*};
use rand::Rng;

use crate::{
    bolt::{
        messages::BoltSpawned,
        registry::BoltRegistry,
        resources::{DEFAULT_BOLT_ANGLE_SPREAD, DEFAULT_BOLT_SPAWN_OFFSET_Y},
    },
    breaker::{BreakerRegistry, SelectedBreaker, messages::BreakerSpawned},
    prelude::*,
    state::run::NodeOutcome,
};

/// Bundles read-only resources needed by [`setup_run`].
#[derive(SystemParam)]
pub(crate) struct SetupRunContext<'w> {
    selected:    Res<'w, SelectedBreaker>,
    breaker_reg: Res<'w, BreakerRegistry>,
    bolt_reg:    Res<'w, BoltRegistry>,
    run_state:   Res<'w, NodeOutcome>,
}

/// Spawns the primary breaker and primary bolt at run start.
///
/// Reads [`SelectedBreaker`] to look up the breaker definition from
/// [`BreakerRegistry`], then reads `BreakerDefinition.bolt` to look up
/// the bolt definition from [`BoltRegistry`]. Sends [`BreakerSpawned`] and
/// [`BoltSpawned`] messages after spawning.
///
/// On the first node (`NodeOutcome.node_index == 0`), the bolt spawns with
/// zero velocity and a [`BoltServing`](crate::bolt::components::BoltServing)
/// marker. On subsequent nodes it launches immediately with a random angle.
///
/// If a breaker already exists from a previous node, returns early without
/// spawning or sending any messages.
///
/// Breaker effects are handled by the breaker builder's `.spawn()` call
/// via [`EffectCommandsExt`](crate::effect_v3::commands::EffectCommandsExt).
/// Bolt effects are handled by the bolt builder's `.definition()` call which
/// stores them as [`BoundEffects`](crate::effect_v3::storage::BoundEffects) during spawn.
/// No manual effect dispatch is needed.
///
/// Runs on `OnEnter(NodeState::Loading)` (first node) alongside other node setup.
pub(crate) fn setup_run(
    mut commands: Commands,
    ctx: SetupRunContext,
    mut rng: ResMut<GameRng>,
    mut render_assets: (ResMut<Assets<Mesh>>, ResMut<Assets<ColorMaterial>>),
    existing_breakers: Query<(), With<Breaker>>,
    mut breaker_spawned: MessageWriter<BreakerSpawned>,
    mut bolt_spawned: MessageWriter<BoltSpawned>,
) {
    let (ref mut meshes, ref mut materials) = render_assets;

    // Step 1: Guard — skip if breaker already exists
    if !existing_breakers.is_empty() {
        return;
    }

    // Step 2: Look up breaker definition
    let Some(breaker_def) = ctx.breaker_reg.get(&ctx.selected.0).cloned() else {
        warn!("Breaker '{}' not found in BreakerRegistry", ctx.selected.0);
        return;
    };

    // Step 3: Spawn breaker
    Breaker::builder()
        .definition(&breaker_def)
        .rendered(meshes, materials)
        .primary()
        .spawn(&mut commands);
    breaker_spawned.write(BreakerSpawned);

    // Step 4: Look up bolt definition
    let bolt_name = &breaker_def.bolt;
    let Some(bolt_def) = ctx.bolt_reg.get(bolt_name).cloned() else {
        warn!(
            "Bolt '{bolt_name}' (from breaker '{}') not found in BoltRegistry",
            ctx.selected.0
        );
        return;
    };

    // Step 5: Compute bolt spawn position
    let breaker_y = breaker_def.y_position;
    let breaker_x = 0.0;
    let spawn_pos = Vec2::new(breaker_x, breaker_y + DEFAULT_BOLT_SPAWN_OFFSET_Y);

    // Step 6: Determine serving state
    let serving = ctx.run_state.node_index == 0;

    // Step 7: Build and spawn bolt
    if serving {
        Bolt::builder()
            .at_position(spawn_pos)
            .definition(&bolt_def)
            .serving()
            .primary()
            .rendered(meshes, materials)
            .spawn(&mut commands);
    } else {
        let random_angle = rng
            .0
            .random_range(-DEFAULT_BOLT_ANGLE_SPREAD..=DEFAULT_BOLT_ANGLE_SPREAD);
        let velocity = Velocity2D(Vec2::new(
            bolt_def.base_speed * random_angle.sin(),
            bolt_def.base_speed * random_angle.cos(),
        ));
        Bolt::builder()
            .at_position(spawn_pos)
            .definition(&bolt_def)
            .with_velocity(velocity)
            .primary()
            .rendered(meshes, materials)
            .spawn(&mut commands);
    }

    // Step 8: Send BoltSpawned message
    bolt_spawned.write(BoltSpawned);
}
