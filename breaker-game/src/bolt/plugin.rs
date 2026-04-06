//! Bolt plugin registration.

use bevy::prelude::*;

use crate::{
    bolt::{
        BoltSystems,
        messages::{BoltImpactBreaker, BoltImpactCell, BoltImpactWall, BoltLost},
        systems::{
            bolt_breaker_collision, bolt_cell_collision, bolt_lost, bolt_wall_collision,
            clamp_bolt_to_playfield, cleanup_destroyed_bolts, dispatch_bolt_effects, hover_bolt,
            launch_bolt, normalize_bolt_speed_after_constraints, spawn_bolt_lost_text,
            sync_bolt_scale, tick_bolt_lifespan,
        },
    },
    breaker::BreakerSystems,
    effect::EffectSystems,
    shared::GameRng,
    state::{
        run::node::{
            sets::NodeSystems,
            systems::{apply_node_scale_to_bolt, apply_node_scale_to_late_bolts, reset_bolt},
        },
        types::NodeState,
    },
};

/// Plugin for the bolt domain.
///
/// Owns bolt components, velocity, speed management, and collision detection.
pub struct BoltPlugin;

impl Plugin for BoltPlugin {
    fn build(&self, app: &mut App) {
        use crate::bolt::messages::{BoltSpawned, RequestBoltDestroyed};
        app.init_resource::<GameRng>()
            .add_message::<BoltSpawned>()
            .add_message::<BoltImpactBreaker>()
            .add_message::<BoltImpactCell>()
            .add_message::<BoltLost>()
            .add_message::<BoltImpactWall>()
            .add_message::<RequestBoltDestroyed>()
            .add_systems(
                OnEnter(NodeState::Loading),
                (
                    apply_node_scale_to_bolt.after(NodeSystems::Spawn),
                    reset_bolt
                        .after(BreakerSystems::Reset)
                        .in_set(BoltSystems::Reset),
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    launch_bolt,
                    hover_bolt.after(BreakerSystems::Move),
                    spawn_bolt_lost_text,
                    // Dispatch bolt-definition effects to target entities
                    dispatch_bolt_effects.before(EffectSystems::Bridge),
                    // Renormalize bolt speed after tether constraints redistribute velocity
                    normalize_bolt_speed_after_constraints.after(
                        rantzsoft_physics2d::plugin::PhysicsSystems::EnforceDistanceConstraints,
                    ),
                    // Collision systems
                    bolt_cell_collision
                        .after(normalize_bolt_speed_after_constraints)
                        .after(BreakerSystems::Move)
                        .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree)
                        .in_set(BoltSystems::CellCollision),
                    bolt_wall_collision
                        .after(BoltSystems::CellCollision)
                        .in_set(BoltSystems::WallCollision),
                    bolt_breaker_collision
                        .after(BoltSystems::CellCollision)
                        .in_set(BoltSystems::BreakerCollision),
                    clamp_bolt_to_playfield.after(bolt_breaker_collision),
                    bolt_lost
                        .after(
                            rantzsoft_physics2d::plugin::PhysicsSystems::EnforceDistanceConstraints,
                        )
                        .after(clamp_bolt_to_playfield)
                        .in_set(BoltSystems::BoltLost),
                    // Tag late-spawned bolts with NodeScalingFactor
                    apply_node_scale_to_late_bolts,
                    // Tick bolt lifespan timers and request destruction on expiry
                    tick_bolt_lifespan.before(BoltSystems::BoltLost),
                    // Cleanup destroyed bolts after effect bridges evaluate
                    cleanup_destroyed_bolts.after(EffectSystems::Bridge),
                )
                    .run_if(in_state(NodeState::Playing)),
            )
            .add_systems(Update, sync_bolt_scale.run_if(in_state(NodeState::Playing)));
    }
}

#[cfg(test)]
mod tests {
    use rantzsoft_physics2d::resources::CollisionQuadtree;

    use super::*;
    use crate::state::types::{AppState, GameState, RunState};

    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<AppState>()
            .add_sub_state::<GameState>()
            .add_sub_state::<RunState>()
            .add_sub_state::<NodeState>()
            // InputPlugin owns InputActions
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<bevy::input::keyboard::KeyboardInput>()
            .add_plugins(crate::input::InputPlugin)
            .insert_resource(CollisionQuadtree::default())
            .add_plugins(BoltPlugin)
            .update();
    }
}
