//! Effect modules — one per effect, each with `fire()`, `reverse()`, `register()`.

use bevy::prelude::*;
use rand::Rng;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{
    Position2D, PreviousPosition, PreviousScale, Scale2D, Velocity2D,
};

use crate::{
    bolt::{
        components::{Bolt, BoltBaseSpeed, BoltMaxSpeed, BoltMinSpeed, BoltRadius, ExtraBolt},
        resources::BoltConfig,
    },
    shared::{
        BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, CleanupOnNodeExit, GameDrawLayer, WALL_LAYER,
        rng::GameRng,
    },
};

/// Compute effective range from base, per-level scaling, and stack count.
///
/// Stacks > 1 add `range_per_level` per additional stack.
/// Formula: `base_range + (stacks - 1).clamp(0, u16::MAX) * range_per_level`
pub(crate) fn effective_range(base_range: f32, range_per_level: f32, stacks: u32) -> f32 {
    let extra = u16::try_from(stacks.saturating_sub(1)).unwrap_or(u16::MAX);
    base_range + f32::from(extra) * range_per_level
}

/// Spawn an extra bolt entity with full physics components at the given position.
///
/// Reads [`BoltConfig`] for radius/speed values and [`GameRng`] for a random
/// velocity direction. Returns the spawned entity ID. Callers insert
/// effect-specific markers (e.g., `ChainBoltMarker`, `PhantomBoltMarker`) after.
pub(crate) fn spawn_extra_bolt(world: &mut World, spawn_pos: Vec2) -> Entity {
    let config = world.resource::<BoltConfig>();
    let radius = config.radius;
    let base_speed = config.base_speed;
    let min_speed = config.min_speed;
    let max_speed = config.max_speed;

    let angle = {
        let mut rng = world.resource_mut::<GameRng>();
        rng.0.random_range(0.0..std::f32::consts::TAU)
    };
    let direction = Vec2::new(angle.cos(), angle.sin());
    let velocity = direction * base_speed;

    world
        .spawn((
            (
                Bolt,
                ExtraBolt,
                Position2D(spawn_pos),
                PreviousPosition(spawn_pos),
                Scale2D {
                    x: radius,
                    y: radius,
                },
                PreviousScale {
                    x: radius,
                    y: radius,
                },
                Aabb2D::new(Vec2::ZERO, Vec2::new(radius, radius)),
            ),
            (
                CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
                Velocity2D(velocity),
                BoltBaseSpeed(base_speed),
                BoltMinSpeed(min_speed),
                BoltMaxSpeed(max_speed),
                BoltRadius(radius),
                CleanupOnNodeExit,
                GameDrawLayer::Bolt,
            ),
        ))
        .id()
}

/// Steer toward nearest entity of a type.
pub mod attraction;
/// Flat bump force increase.
pub mod bump_force;
/// Spawn two bolts chained together.
pub mod chain_bolt;
/// Arc damage jumping between cells.
pub mod chain_lightning;
/// Multiplicative damage bonus.
pub mod damage_boost;
/// Escalating chaos — fires multiple random effects per cell destroyed.
pub mod entropy_engine;
/// Instant area damage burst.
pub mod explode;
/// Gravity well that attracts bolts within radius.
pub mod gravity_well;
/// Decrement lives.
pub mod life_lost;
/// Pass through cells instead of bouncing.
pub mod piercing;
/// Beam through cells in velocity direction.
pub mod piercing_beam;
/// Shockwave at every active bolt position.
pub mod pulse;
/// Breaker deceleration multiplier.
pub mod quick_stop;
/// Stacking damage bonus on consecutive cell hits.
pub mod ramping_damage;
/// Weighted random selection from a pool.
pub mod random_effect;
/// Invisible bottom wall that bounces bolt once.
pub mod second_wind;
/// Temporary breaker protection.
pub mod shield;
/// Expanding ring of area damage.
pub mod shockwave;
/// Size increase (bolt radius or breaker width).
pub mod size_boost;
/// Spawn additional bolts.
pub mod spawn_bolts;
/// Temporary phantom bolt with infinite piercing.
pub mod spawn_phantom;
/// Multiplicative speed scaling.
pub mod speed_boost;
/// Two bolts connected by a damaging beam.
pub mod tether_beam;
/// Subtract time from node timer.
pub mod time_penalty;

/// Register all effect runtime systems.
pub(crate) fn register(app: &mut bevy::prelude::App) {
    speed_boost::register(app);
    damage_boost::register(app);
    piercing::register(app);
    size_boost::register(app);
    bump_force::register(app);
    shockwave::register(app);
    chain_lightning::register(app);
    piercing_beam::register(app);
    pulse::register(app);
    shield::register(app);
    gravity_well::register(app);
    spawn_phantom::register(app);
    entropy_engine::register(app);
    ramping_damage::register(app);
    explode::register(app);
    spawn_bolts::register(app);
    chain_bolt::register(app);
    attraction::register(app);
    quick_stop::register(app);
    tether_beam::register(app);
    life_lost::register(app);
    time_penalty::register(app);
    second_wind::register(app);
    random_effect::register(app);
}
