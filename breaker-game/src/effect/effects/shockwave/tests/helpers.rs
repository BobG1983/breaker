use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, plugin::RantzPhysics2dPlugin,
};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D};

use crate::{
    cells::{
        components::{Cell, CellHealth, Locked},
        messages::DamageCell,
    },
    chips::components::DamageBoost,
    effect::effects::shockwave::system::*,
    shared::{BOLT_LAYER, CELL_LAYER, CleanupOnNodeExit, GameDrawLayer},
};

// --- Test infrastructure ---

/// Captured `DamageCell` messages written by the shockwave collision system.
#[derive(Resource, Default)]
pub(super) struct CapturedDamage(pub Vec<DamageCell>);

pub(super) fn capture_damage(
    mut reader: MessageReader<DamageCell>,
    mut captured: ResMut<CapturedDamage>,
) {
    for msg in reader.read() {
        captured.0.push(msg.clone());
    }
}

pub(super) fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(RantzPhysics2dPlugin)
        .add_message::<DamageCell>()
        .init_resource::<CapturedDamage>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .add_systems(FixedPostUpdate, capture_damage)
        .add_observer(handle_shockwave);
    app
}

pub(super) fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

pub(super) fn spawn_bolt(app: &mut App, x: f32, y: f32) -> Entity {
    app.world_mut().spawn(Position2D(Vec2::new(x, y))).id()
}

pub(super) fn spawn_bolt_with_damage_boost(app: &mut App, x: f32, y: f32, boost: f32) -> Entity {
    app.world_mut()
        .spawn((Position2D(Vec2::new(x, y)), DamageBoost(boost)))
        .id()
}

pub(super) fn spawn_cell(app: &mut App, x: f32, y: f32, hp: f32) -> Entity {
    let pos = Vec2::new(x, y);
    app.world_mut()
        .spawn((
            Cell,
            CellHealth::new(hp),
            Aabb2D::new(Vec2::ZERO, Vec2::new(35.0, 12.0)),
            CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            GameDrawLayer::Cell,
        ))
        .id()
}

pub(super) fn spawn_locked_cell(app: &mut App, x: f32, y: f32, hp: f32) -> Entity {
    let pos = Vec2::new(x, y);
    app.world_mut()
        .spawn((
            Cell,
            CellHealth::new(hp),
            Locked,
            Aabb2D::new(Vec2::ZERO, Vec2::new(35.0, 12.0)),
            CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            GameDrawLayer::Cell,
        ))
        .id()
}

pub(super) fn trigger_shockwave(app: &mut App, bolt: Entity, range: f32, speed: f32) {
    use crate::effect::typed_events::ShockwaveFired;

    app.world_mut().commands().trigger(ShockwaveFired {
        base_range: range,
        range_per_level: 0.0,
        stacks: 1,
        speed,
        targets: vec![crate::effect::definition::EffectTarget::Entity(bolt)],
        source_chip: None,
    });
    app.world_mut().flush();
    tick(app);
}

/// Count entities with [`ShockwaveRadius`] component.
pub(super) fn shockwave_entity_count(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<Entity, With<ShockwaveRadius>>()
        .iter(app.world())
        .count()
}

/// Get the first shockwave entity (panics if none).
pub(super) fn get_shockwave_entity(app: &mut App) -> Entity {
    app.world_mut()
        .query_filtered::<Entity, With<ShockwaveRadius>>()
        .iter(app.world())
        .next()
        .expect("should have at least one ShockwaveRadius entity")
}

/// Asserts the standard components present on every spawned shockwave entity:
/// `ShockwaveAlreadyHit` (empty), `GameDrawLayer::Fx`, `CleanupOnNodeExit`,
/// `Spatial2D`, and default `Scale2D`.
pub(super) fn assert_standard_shockwave_components(world: &World, sw_entity: Entity) {
    let already_hit = world
        .get::<ShockwaveAlreadyHit>(sw_entity)
        .expect("shockwave entity should have ShockwaveAlreadyHit");
    assert!(
        already_hit.0.is_empty(),
        "ShockwaveAlreadyHit should start empty"
    );

    let draw_layer = world
        .get::<GameDrawLayer>(sw_entity)
        .expect("shockwave entity should have GameDrawLayer");
    assert!(
        matches!(draw_layer, GameDrawLayer::Fx),
        "draw layer should be Fx"
    );

    assert!(
        world.get::<CleanupOnNodeExit>(sw_entity).is_some(),
        "shockwave entity should have CleanupOnNodeExit"
    );

    assert!(
        world.get::<Spatial2D>(sw_entity).is_some(),
        "shockwave entity should have Spatial2D"
    );

    let scale = world
        .get::<rantzsoft_spatial2d::components::Scale2D>(sw_entity)
        .expect("shockwave entity should have Scale2D");
    assert!(
        (scale.x - 1.0).abs() < f32::EPSILON && (scale.y - 1.0).abs() < f32::EPSILON,
        "initial Scale2D should be (1.0, 1.0), got ({}, {})",
        scale.x,
        scale.y
    );
}
