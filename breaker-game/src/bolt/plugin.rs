//! Bolt plugin registration.

use bevy::prelude::*;

use crate::{
    bolt::{
        BoltSystems,
        messages::{
            BoltImpactBreaker, BoltImpactCell, BoltImpactWall, BoltLost, SpawnAdditionalBolt,
            SpawnChainBolt,
        },
        resources::BoltConfig,
        systems::{
            apply_entity_scale_to_bolt, bolt_breaker_collision, bolt_cell_collision, bolt_lost,
            bolt_scale_visual, bolt_wall_collision, clamp_bolt_to_playfield,
            cleanup_destroyed_bolts, hover_bolt, init_bolt_params, launch_bolt,
            prepare_bolt_velocity, reset_bolt, spawn_bolt, spawn_bolt_lost_text,
            tick_bolt_lifespan,
        },
    },
    breaker::BreakerSystems,
    effect::EffectSystems,
    run::node::sets::NodeSystems,
    shared::{GameRng, GameState, PlayingState},
};

/// Plugin for the bolt domain.
///
/// Owns bolt components, velocity, speed management, and collision detection.
pub struct BoltPlugin;

impl Plugin for BoltPlugin {
    fn build(&self, app: &mut App) {
        use crate::bolt::messages::{BoltSpawned, RequestBoltDestroyed};
        app.init_resource::<BoltConfig>()
            .init_resource::<GameRng>()
            .add_message::<SpawnAdditionalBolt>()
            .add_message::<BoltSpawned>()
            .add_message::<BoltImpactBreaker>()
            .add_message::<BoltImpactCell>()
            .add_message::<BoltLost>()
            .add_message::<BoltImpactWall>()
            .add_message::<RequestBoltDestroyed>()
            .add_message::<SpawnChainBolt>()
            .add_systems(
                OnEnter(GameState::Playing),
                (
                    spawn_bolt,
                    init_bolt_params
                        .after(spawn_bolt)
                        .in_set(BoltSystems::InitParams),
                    apply_entity_scale_to_bolt
                        .after(BoltSystems::InitParams)
                        .after(NodeSystems::Spawn),
                    reset_bolt
                        .after(BoltSystems::InitParams)
                        .after(BreakerSystems::Reset)
                        .in_set(BoltSystems::Reset),
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    launch_bolt,
                    (
                        hover_bolt,
                        prepare_bolt_velocity
                            .in_set(BoltSystems::PrepareVelocity)
                            .after(EffectSystems::Recalculate),
                    )
                        .after(BreakerSystems::Move),
                    spawn_bolt_lost_text,
                    // Collision systems
                    bolt_cell_collision
                        .after(BoltSystems::PrepareVelocity)
                        .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree)
                        .in_set(BoltSystems::CellCollision),
                    bolt_wall_collision.after(BoltSystems::CellCollision),
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
                    // Tick bolt lifespan timers and request destruction on expiry
                    tick_bolt_lifespan.before(BoltSystems::BoltLost),
                    // Cleanup destroyed bolts after effect bridges evaluate
                    cleanup_destroyed_bolts.after(EffectSystems::Bridge),
                )
                    .run_if(in_state(PlayingState::Active)),
            )
            .add_systems(
                Update,
                bolt_scale_visual.run_if(in_state(PlayingState::Active)),
            );
    }
}

#[cfg(test)]
mod tests {
    use rantzsoft_physics2d::resources::CollisionQuadtree;

    use super::*;

    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<GameState>()
            .add_sub_state::<PlayingState>()
            // InputPlugin owns InputActions
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<bevy::input::keyboard::KeyboardInput>()
            .add_plugins(crate::input::InputPlugin)
            .insert_resource(CollisionQuadtree::default())
            .add_plugins(BoltPlugin)
            .update();
    }
}
