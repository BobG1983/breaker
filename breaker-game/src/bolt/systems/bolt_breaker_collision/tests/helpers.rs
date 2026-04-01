use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, plugin::RantzPhysics2dPlugin,
};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D, Velocity2D};

use super::super::system::bolt_breaker_collision;
use crate::{
    bolt::{
        BoltConfig,
        components::{Bolt, BoltRadius},
        messages::BoltImpactBreaker,
    },
    breaker::{
        components::{Breaker, BreakerHeight, BreakerReflectionSpread, BreakerTilt, BreakerWidth},
        resources::BreakerConfig,
    },
    shared::{BOLT_LAYER, BREAKER_LAYER, EntityScale, GameDrawLayer},
};

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(RantzPhysics2dPlugin)
        .add_message::<BoltImpactBreaker>()
        .add_systems(
            FixedUpdate,
            bolt_breaker_collision
                .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
        );
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

pub(super) fn default_reflection_spread() -> BreakerReflectionSpread {
    BreakerReflectionSpread(BreakerConfig::default().reflection_spread.to_radians())
}

/// Breaker entities use `Position2D` as canonical position.
pub(super) fn spawn_breaker_at(app: &mut App, x: f32, y: f32) -> Entity {
    let w = default_breaker_width();
    let h = default_breaker_height();
    let half_extents = Vec2::new(w.half_width(), h.half_height());
    let pos = Vec2::new(x, y);
    app.world_mut()
        .spawn((
            Breaker,
            BreakerTilt::default(),
            w,
            h,
            default_reflection_spread(),
            Aabb2D::new(Vec2::ZERO, half_extents),
            CollisionLayers::new(BREAKER_LAYER, BOLT_LAYER),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            GameDrawLayer::Breaker,
        ))
        .id()
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
    Bolt::builder()
        .at_position(Vec2::new(x, y))
        .config(&BoltConfig::default())
        .with_velocity(Velocity2D(Vec2::new(vx, vy)))
        .primary()
        .spawn(app.world_mut())
}

#[derive(Resource, Default)]
pub(super) struct HitBreakers(pub(super) u32);

pub(super) fn collect_breaker_hits(
    mut reader: MessageReader<BoltImpactBreaker>,
    mut hits: ResMut<HitBreakers>,
) {
    for _msg in reader.read() {
        hits.0 += 1;
    }
}

#[derive(Resource, Default)]
pub(super) struct CapturedHitBolts(pub(super) Vec<Entity>);

pub(super) fn collect_breaker_hit_bolts(
    mut reader: MessageReader<BoltImpactBreaker>,
    mut captured: ResMut<CapturedHitBolts>,
) {
    for msg in reader.read() {
        captured.0.push(msg.bolt);
    }
}

/// Captured bolt-and-breaker entity pairs from `BoltImpactBreaker` messages.
#[derive(Resource, Default)]
pub(super) struct CapturedHitPairs(pub(super) Vec<(Entity, Entity)>);

pub(super) fn collect_breaker_hit_pairs(
    mut reader: MessageReader<BoltImpactBreaker>,
    mut captured: ResMut<CapturedHitPairs>,
) {
    for msg in reader.read() {
        captured.0.push((msg.bolt, msg.breaker));
    }
}

pub(super) fn spawn_scaled_breaker_at(app: &mut App, x: f32, y: f32, entity_scale: f32) {
    let w = default_breaker_width();
    let h = default_breaker_height();
    let half_extents = Vec2::new(w.half_width(), h.half_height());
    let pos = Vec2::new(x, y);
    app.world_mut().spawn((
        Breaker,
        BreakerTilt::default(),
        w,
        h,
        default_reflection_spread(),
        EntityScale(entity_scale),
        Aabb2D::new(Vec2::ZERO, half_extents),
        CollisionLayers::new(BREAKER_LAYER, BOLT_LAYER),
        Position2D(pos),
        GlobalPosition2D(pos),
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
    let entity = Bolt::builder()
        .at_position(Vec2::new(x, y))
        .config(&BoltConfig::default())
        .with_velocity(Velocity2D(Vec2::new(vx, vy)))
        .primary()
        .spawn(app.world_mut());
    app.world_mut()
        .entity_mut(entity)
        .insert(EntityScale(entity_scale));
    entity
}
