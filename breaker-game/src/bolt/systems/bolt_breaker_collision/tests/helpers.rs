use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Spatial2D, Velocity2D};

use super::super::system::bolt_breaker_collision;
use crate::{
    bolt::{
        BoltConfig,
        components::{Bolt, BoltBaseSpeed, BoltRadius},
        messages::BoltHitBreaker,
    },
    breaker::{
        components::{
            Breaker, BreakerHeight, BreakerTilt, BreakerWidth, MaxReflectionAngle,
            MinAngleFromHorizontal,
        },
        resources::BreakerConfig,
    },
    shared::{EntityScale, GameDrawLayer},
};

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<BoltHitBreaker>()
        .add_systems(FixedUpdate, bolt_breaker_collision);
    app
}

pub(super) fn default_breaker_width() -> BreakerWidth {
    BreakerWidth(120.0)
}

pub(super) fn default_breaker_height() -> BreakerHeight {
    BreakerHeight(20.0)
}

pub(super) fn default_bolt_radius() -> BoltRadius {
    BoltRadius(BoltConfig::default().radius)
}

pub(super) fn default_max_reflection_angle() -> MaxReflectionAngle {
    MaxReflectionAngle(BreakerConfig::default().max_reflection_angle.to_radians())
}

pub(super) fn default_min_angle() -> MinAngleFromHorizontal {
    MinAngleFromHorizontal(
        BreakerConfig::default()
            .min_angle_from_horizontal
            .to_radians(),
    )
}

pub(super) fn bolt_param_bundle() -> (BoltBaseSpeed, BoltRadius) {
    let bolt_config = BoltConfig::default();
    (
        BoltBaseSpeed(bolt_config.base_speed),
        BoltRadius(bolt_config.radius),
    )
}

/// Breaker entities use `Position2D` as canonical position.
pub(super) fn spawn_breaker_at(app: &mut App, x: f32, y: f32) {
    app.world_mut().spawn((
        Breaker,
        BreakerTilt::default(),
        default_breaker_width(),
        default_breaker_height(),
        default_max_reflection_angle(),
        default_min_angle(),
        Position2D(Vec2::new(x, y)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));
}

/// Accumulates one fixed timestep of overstep, then runs one update.
pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

/// Bolt entities now use `Position2D` as canonical position.
pub(super) fn spawn_bolt(app: &mut App, x: f32, y: f32, vx: f32, vy: f32) -> Entity {
    app.world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(vx, vy)),
            bolt_param_bundle(),
            Position2D(Vec2::new(x, y)),
        ))
        .id()
}

#[derive(Resource, Default)]
pub(super) struct HitBreakers(pub(super) u32);

pub(super) fn collect_breaker_hits(
    mut reader: MessageReader<BoltHitBreaker>,
    mut hits: ResMut<HitBreakers>,
) {
    for _msg in reader.read() {
        hits.0 += 1;
    }
}

#[derive(Resource, Default)]
pub(super) struct CapturedHitBolts(pub(super) Vec<Entity>);

pub(super) fn collect_breaker_hit_bolts(
    mut reader: MessageReader<BoltHitBreaker>,
    mut captured: ResMut<CapturedHitBolts>,
) {
    for msg in reader.read() {
        captured.0.push(msg.bolt);
    }
}

pub(super) fn spawn_scaled_breaker_at(app: &mut App, x: f32, y: f32, entity_scale: f32) {
    app.world_mut().spawn((
        Breaker,
        BreakerTilt::default(),
        default_breaker_width(),
        default_breaker_height(),
        default_max_reflection_angle(),
        default_min_angle(),
        EntityScale(entity_scale),
        Position2D(Vec2::new(x, y)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));
}

pub(super) fn spawn_scaled_bolt(
    app: &mut App,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    entity_scale: f32,
) -> Entity {
    app.world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(vx, vy)),
            bolt_param_bundle(),
            EntityScale(entity_scale),
            Position2D(Vec2::new(x, y)),
        ))
        .id()
}
