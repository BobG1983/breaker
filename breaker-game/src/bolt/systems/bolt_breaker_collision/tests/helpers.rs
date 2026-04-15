use bevy::prelude::*;
use rantzsoft_spatial2d::components::{GlobalPosition2D, Spatial2D};

pub(super) use crate::bolt::test_utils::default_bolt_definition;
use crate::{
    bolt::{
        components::BoltRadius, systems::bolt_breaker_collision::system::bolt_breaker_collision,
    },
    breaker::{
        components::{BaseHeight, BaseWidth, BreakerReflectionSpread, BreakerTilt},
        definition::BreakerDefinition,
    },
    prelude::*,
    shared::{GameDrawLayer, size::BaseRadius},
};

pub(super) fn test_app() -> App {
    TestAppBuilder::new()
        .with_physics()
        .with_message::<BoltImpactBreaker>()
        .with_system(
            FixedUpdate,
            bolt_breaker_collision
                .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
        )
        .build()
}

pub(super) fn default_breaker_width() -> BaseWidth {
    BaseWidth(120.0)
}

pub(super) fn default_breaker_height() -> BaseHeight {
    BaseHeight(20.0)
}

pub(super) fn default_bolt_radius() -> BoltRadius {
    BaseRadius(default_bolt_definition().radius)
}

pub(super) fn default_reflection_spread() -> BreakerReflectionSpread {
    BreakerReflectionSpread(BreakerDefinition::default().reflection_spread.to_radians())
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

pub(super) use crate::{bolt::test_utils::spawn_bolt, shared::test_utils::tick};

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
        NodeScalingFactor(entity_scale),
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
    let entity = spawn_bolt(app, x, y, vx, vy);
    app.world_mut()
        .entity_mut(entity)
        .insert(NodeScalingFactor(entity_scale));
    entity
}
