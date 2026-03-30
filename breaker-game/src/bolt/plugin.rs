//! Bolt plugin registration.

use bevy::prelude::*;

use crate::{
    bolt::{
        BoltSystems,
        messages::{BoltImpactBreaker, BoltImpactCell, BoltImpactWall, BoltLost},
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
            .add_message::<BoltSpawned>()
            .add_message::<BoltImpactBreaker>()
            .add_message::<BoltImpactCell>()
            .add_message::<BoltLost>()
            .add_message::<BoltImpactWall>()
            .add_message::<RequestBoltDestroyed>()
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
                            .after(
                            rantzsoft_physics2d::plugin::PhysicsSystems::EnforceDistanceConstraints,
                        ),
                    )
                        .after(BreakerSystems::Move),
                    spawn_bolt_lost_text,
                    // Collision systems
                    bolt_cell_collision
                        .after(BoltSystems::PrepareVelocity)
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

    // ── Regression: PrepareVelocity must run after EnforceDistanceConstraints ──

    /// Regression: `enforce_distance_constraints` redistributes velocity after
    /// speed clamp, allowing bolt speed to exceed `BoltMaxSpeed`.
    ///
    /// Given: Two tethered bolts far apart (separation=500, `max_distance=100`).
    ///   A at (0,0) with velocity (500,0), B at (0,500) with velocity (0,1200).
    ///   The constraint solver redistributes axial velocity, pushing A's speed
    ///   to ~781 (sqrt(500^2 + 600^2)).
    ///
    /// When: Both `enforce_distance_constraints` and `prepare_bolt_velocity` run in
    ///   the same `FixedUpdate` tick.
    ///
    /// Then: All bolt speeds are within [`min_speed`, `max_speed`].
    ///
    /// This test FAILS if `BoltSystems::PrepareVelocity` runs before
    /// `PhysicsSystems::EnforceDistanceConstraints` — the clamp runs first,
    /// then the constraint solver adds velocity unchecked.
    #[test]
    fn bolt_speed_stays_in_range_after_distance_constraint() {
        use rantzsoft_physics2d::constraint::DistanceConstraint;
        use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

        use crate::{
            bolt::components::{Bolt, BoltMaxSpeed, BoltMinSpeed},
            breaker::components::{Breaker, MinAngleFromHorizontal},
        };

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.add_sub_state::<PlayingState>();

        // Register BoltPlugin FIRST — this inserts prepare_bolt_velocity early
        // in the schedule. Without an explicit `.after(EnforceDistanceConstraints)`
        // constraint, the clamp runs before the constraint solver, meaning
        // velocity redistribution happens AFTER clamping and can push speed
        // above max.
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_message::<bevy::input::keyboard::KeyboardInput>();
        app.add_plugins(crate::input::InputPlugin);
        app.insert_resource(CollisionQuadtree::default());
        app.add_plugins(BoltPlugin);

        // Register physics plugin SECOND — enforce_distance_constraints is
        // inserted after prepare_bolt_velocity in the schedule.
        app.add_plugins(rantzsoft_physics2d::plugin::RantzPhysics2dPlugin);

        // Resources required by OnEnter(Playing) and FixedUpdate systems
        app.init_resource::<crate::breaker::resources::BreakerConfig>();
        app.init_resource::<crate::run::RunState>();
        app.init_resource::<crate::shared::PlayfieldConfig>();
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<ColorMaterial>>();
        app.add_message::<crate::cells::messages::DamageCell>();

        // Enter Playing state
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();

        let min_speed = 200.0_f32;
        let max_speed = 600.0_f32;

        // Spawn breaker (required by prepare_bolt_velocity)
        app.world_mut()
            .spawn((Breaker, MinAngleFromHorizontal(15.0_f32.to_radians())));

        // Two tethered bolts set up so that constraint redistribution pushes
        // bolt A's speed above max_speed.
        //
        // Setup: A at (0,0) with velocity (500,0) (moving sideways, speed=500).
        //        B at (0,500) with velocity (0,1200) (moving away from A, speed=1200).
        // Tether max_distance=100, separation=500 -> taut, needs correction.
        //
        // Constraint axis = (0,1). dot_a=0, dot_b=1200.
        // avg = midpoint(0, 1200) = 600.
        // A gets (600-0)*(0,1) = (0,600) added -> A vel = (500, 600), speed = ~781.
        // B gets (600-1200)*(0,1) = (0,-600) added -> B vel = (0, 600), speed = 600.
        //
        // If speed clamp runs AFTER constraint enforcement: A clamped to 600. PASS.
        // If speed clamp runs BEFORE: A stays at 781 (no second clamp). FAIL.
        let bolt_a = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(500.0, 0.0)), // speed=500, under max
                BoltMinSpeed(min_speed),
                BoltMaxSpeed(max_speed),
            ))
            .id();

        let bolt_b = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(0.0, 500.0)),
                Velocity2D(Vec2::new(0.0, 1200.0)), // speed=1200, moving away from A
                BoltMinSpeed(min_speed),
                BoltMaxSpeed(max_speed),
            ))
            .id();

        app.world_mut().spawn(DistanceConstraint {
            entity_a: bolt_a,
            entity_b: bolt_b,
            max_distance: 100.0,
        });

        // Tick one fixed update
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();

        let vel_a = app.world().get::<Velocity2D>(bolt_a).unwrap();
        let vel_b = app.world().get::<Velocity2D>(bolt_b).unwrap();
        let speed_a = vel_a.speed();
        let speed_b = vel_b.speed();

        assert!(
            speed_a <= max_speed + 1.0,
            "bolt A speed ({speed_a:.1}) should not exceed BoltMaxSpeed ({max_speed:.1}) \
             after distance constraint + speed clamp — PrepareVelocity must be ordered \
             after PhysicsSystems::EnforceDistanceConstraints"
        );
        assert!(
            speed_b <= max_speed + 1.0,
            "bolt B speed ({speed_b:.1}) should not exceed BoltMaxSpeed ({max_speed:.1}) \
             after distance constraint + speed clamp — PrepareVelocity must be ordered \
             after PhysicsSystems::EnforceDistanceConstraints"
        );
    }
}
