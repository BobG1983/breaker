//! System to set up the run: spawns primary breaker and primary bolt.

use bevy::prelude::*;
use rand::Rng;
use rantzsoft_spatial2d::components::Velocity2D;

use crate::{
    bolt::{
        components::Bolt,
        messages::BoltSpawned,
        registry::BoltRegistry,
        resources::{DEFAULT_BOLT_ANGLE_SPREAD, DEFAULT_BOLT_SPAWN_OFFSET_Y},
    },
    breaker::{BreakerRegistry, SelectedBreaker, components::Breaker, messages::BreakerSpawned},
    shared::rng::GameRng,
    state::run::NodeOutcome,
};

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
/// via [`EffectCommandsExt::dispatch_initial_effects`](crate::effect::EffectCommandsExt).
/// Bolt effects are handled by the bolt builder's `.definition()` call which
/// stores them as [`BoundEffects`](crate::effect::BoundEffects) during spawn.
/// No manual effect dispatch is needed.
///
/// Runs on `OnEnter(NodeState::Loading)` (first node) alongside other node setup.
#[expect(
    clippy::too_many_arguments,
    reason = "spawns both breaker and bolt, needs both registries + assets"
)]
pub(crate) fn setup_run(
    mut commands: Commands,
    selected: Res<SelectedBreaker>,
    breaker_reg: Res<BreakerRegistry>,
    bolt_reg: Res<BoltRegistry>,
    run_state: Res<NodeOutcome>,
    mut rng: ResMut<GameRng>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    existing_breakers: Query<(), With<Breaker>>,
    mut breaker_spawned: MessageWriter<BreakerSpawned>,
    mut bolt_spawned: MessageWriter<BoltSpawned>,
) {
    // Step 1: Guard — skip if breaker already exists
    if !existing_breakers.is_empty() {
        return;
    }

    // Step 2: Look up breaker definition
    let Some(breaker_def) = breaker_reg.get(&selected.0).cloned() else {
        warn!("Breaker '{}' not found in BreakerRegistry", selected.0);
        return;
    };

    // Step 3: Spawn breaker
    Breaker::builder()
        .definition(&breaker_def)
        .rendered(&mut meshes, &mut materials)
        .primary()
        .spawn(&mut commands);
    breaker_spawned.write(BreakerSpawned);

    // Step 4: Look up bolt definition
    let bolt_name = &breaker_def.bolt;
    let Some(bolt_def) = bolt_reg.get(bolt_name).cloned() else {
        warn!(
            "Bolt '{bolt_name}' (from breaker '{}') not found in BoltRegistry",
            selected.0
        );
        return;
    };

    // Step 5: Compute bolt spawn position
    let breaker_y = breaker_def.y_position;
    let breaker_x = 0.0;
    let spawn_pos = Vec2::new(breaker_x, breaker_y + DEFAULT_BOLT_SPAWN_OFFSET_Y);

    // Step 6: Determine serving state
    let serving = run_state.node_index == 0;

    // Step 7: Build and spawn bolt
    if serving {
        Bolt::builder()
            .at_position(spawn_pos)
            .definition(&bolt_def)
            .serving()
            .primary()
            .rendered(&mut meshes, &mut materials)
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
            .rendered(&mut meshes, &mut materials)
            .spawn(&mut commands);
    }

    // Step 8: Send BoltSpawned message
    bolt_spawned.write(BoltSpawned);

    debug!(
        "setup_run: spawned breaker '{}' and bolt '{}'",
        selected.0, bolt_name
    );
}
