use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, plugin::RantzPhysics2dPlugin,
};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D};

use super::effect::*;
use crate::{
    bolt::BASE_BOLT_DAMAGE,
    cells::{components::Cell, messages::DamageCell},
    shared::{CELL_LAYER, WALL_LAYER},
};

// -- fire tests ──────────────────────────────────────────────────

#[test]
fn fire_spawns_shockwave_entity_at_source_position() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(100.0, 200.0, 0.0)).id();

    fire(entity, 24.0, 8.0, 1, 50.0, "", &mut world);

    let mut query = world.query::<(
        &ShockwaveSource,
        &ShockwaveRadius,
        &ShockwaveMaxRadius,
        &ShockwaveSpeed,
        &Transform,
    )>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(results.len(), 1, "expected exactly one shockwave entity");

    let (source, radius, max_radius, speed, transform) = results[0];
    assert_eq!(source.0, entity);
    assert!(
        (radius.0 - 0.0).abs() < f32::EPSILON,
        "expected radius 0.0, got {}",
        radius.0
    );
    // stacks=1 -> effective = 24.0 + (1-1)*8.0 = 24.0
    assert!(
        (max_radius.0 - 24.0).abs() < f32::EPSILON,
        "expected max_radius 24.0, got {}",
        max_radius.0
    );
    assert!(
        (speed.0 - 50.0).abs() < f32::EPSILON,
        "expected speed 50.0, got {}",
        speed.0
    );
    assert!(
        (transform.translation.x - 100.0).abs() < f32::EPSILON,
        "expected x 100.0, got {}",
        transform.translation.x
    );
    assert!(
        (transform.translation.y - 200.0).abs() < f32::EPSILON,
        "expected y 200.0, got {}",
        transform.translation.y
    );
}

#[test]
fn fire_effective_range_scales_with_stacks() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

    // stacks=3, base=24, per_level=8 -> effective = 24 + (3-1)*8 = 40
    fire(entity, 24.0, 8.0, 3, 50.0, "", &mut world);

    let mut query = world.query::<&ShockwaveMaxRadius>();
    let max_radius = query.iter(&world).next().unwrap();
    assert!(
        (max_radius.0 - 40.0).abs() < f32::EPSILON,
        "expected max_radius 40.0, got {}",
        max_radius.0
    );
}

#[test]
fn reverse_is_noop_shockwave_entity_remains() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

    fire(entity, 24.0, 8.0, 1, 50.0, "", &mut world);

    // Verify shockwave exists before reverse
    let mut query = world.query::<&ShockwaveSource>();
    assert_eq!(query.iter(&world).count(), 1);

    reverse(entity, "", &mut world);

    // Shockwave entity should still exist after reverse (no-op)
    assert_eq!(query.iter(&world).count(), 1);
}

// -- Behavior 7: fire() spawns ShockwaveDamaged on shockwave entity ──

#[test]
fn fire_spawns_shockwave_damaged_component_on_entity() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(50.0, 75.0, 0.0)).id();

    fire(entity, 24.0, 8.0, 1, 50.0, "", &mut world);

    let mut query = world.query::<&ShockwaveDamaged>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        results.len(),
        1,
        "expected exactly one entity with ShockwaveDamaged"
    );
    assert!(
        results[0].0.is_empty(),
        "ShockwaveDamaged set should be empty on spawn"
    );
}

#[test]
fn fire_twice_spawns_two_independent_shockwave_damaged() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(50.0, 75.0, 0.0)).id();

    fire(entity, 24.0, 8.0, 1, 50.0, "", &mut world);
    fire(entity, 24.0, 8.0, 1, 50.0, "", &mut world);

    let mut query = world.query::<&ShockwaveDamaged>();
    let count = query.iter(&world).count();
    assert_eq!(
        count, 2,
        "two fire() calls should produce two ShockwaveDamaged components"
    );

    for damaged in query.iter(&world) {
        assert!(
            damaged.0.is_empty(),
            "each ShockwaveDamaged should start with an empty HashSet"
        );
    }
}

// -- system tests ────────────────────────────────────────────────

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<crate::shared::game_state::GameState>();
    app.add_sub_state::<crate::shared::playing_state::PlayingState>();
    app.add_systems(Update, tick_shockwave);
    app.add_systems(Update, despawn_finished_shockwave);
    app
}

fn enter_playing(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<crate::shared::game_state::GameState>>()
        .set(crate::shared::game_state::GameState::Playing);
    app.update();
}

#[test]
fn tick_shockwave_expands_radius_by_speed_times_dt() {
    let mut app = test_app();
    enter_playing(&mut app);

    let shockwave = app
        .world_mut()
        .spawn((
            ShockwaveRadius(0.0),
            ShockwaveMaxRadius(100.0),
            ShockwaveSpeed(50.0),
        ))
        .id();

    app.update();

    let radius = app.world().get::<ShockwaveRadius>(shockwave).unwrap();
    // After one update tick, radius should have increased by speed * dt.
    // dt is not zero since MinimalPlugins provides Time.
    assert!(
        radius.0 > 0.0,
        "shockwave radius should expand after tick, got {}",
        radius.0
    );
}

#[test]
fn despawn_finished_shockwave_removes_entity_when_radius_ge_max() {
    let mut app = test_app();
    enter_playing(&mut app);

    let shockwave = app
        .world_mut()
        .spawn((
            ShockwaveRadius(100.0),
            ShockwaveMaxRadius(100.0),
            ShockwaveSpeed(50.0),
        ))
        .id();

    app.update();

    // Entity should be despawned because radius >= max_radius
    assert!(
        app.world().get_entity(shockwave).is_err(),
        "shockwave entity should be despawned when radius >= max_radius"
    );
}

// -- Shockwave damage system tests ───────────────────────────────

/// Collects [`DamageCell`] messages into a resource for test assertions.
#[derive(Resource, Default)]
struct DamageCellCollector(Vec<DamageCell>);

fn collect_damage_cells(
    mut reader: MessageReader<DamageCell>,
    mut collector: ResMut<DamageCellCollector>,
) {
    for msg in reader.read() {
        collector.0.push(msg.clone());
    }
}

fn damage_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(RantzPhysics2dPlugin);
    app.add_message::<DamageCell>();
    app.insert_resource(DamageCellCollector::default());
    app.add_systems(Update, apply_shockwave_damage);
    app.add_systems(Update, collect_damage_cells.after(apply_shockwave_damage));
    app
}

/// Accumulates one fixed timestep then runs one update (ensures quadtree maintenance runs).
fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

fn spawn_test_cell(app: &mut App, x: f32, y: f32) -> Entity {
    let pos = Vec2::new(x, y);
    app.world_mut()
        .spawn((
            Cell,
            Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
            CollisionLayers::new(CELL_LAYER, 0),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
        ))
        .id()
}

fn spawn_shockwave(app: &mut App, x: f32, y: f32, radius: f32, damaged: HashSet<Entity>) -> Entity {
    app.world_mut()
        .spawn((
            ShockwaveSource(Entity::PLACEHOLDER),
            ShockwaveRadius(radius),
            ShockwaveMaxRadius(100.0),
            ShockwaveSpeed(50.0),
            ShockwaveDamaged(damaged),
            Transform::from_xyz(x, y, 0.0),
        ))
        .id()
}

// -- Behavior 1: Shockwave damages a cell within its current radius ──

#[test]
fn shockwave_damages_cell_within_radius() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 30.0, 0.0);
    let shockwave = spawn_shockwave(&mut app, 0.0, 0.0, 35.0, HashSet::new());

    // Tick to populate quadtree, then update to run damage system
    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected exactly one DamageCell message, got {}",
        collector.0.len()
    );
    assert_eq!(collector.0[0].cell, cell);
    assert!(
        (collector.0[0].damage - BASE_BOLT_DAMAGE).abs() < f32::EPSILON,
        "expected damage {}, got {}",
        BASE_BOLT_DAMAGE,
        collector.0[0].damage
    );
    assert!(
        collector.0[0].source_chip.is_none(),
        "source_chip should be None for shockwave damage"
    );

    // Cell should be in the shockwave's damaged set
    let damaged = app.world().get::<ShockwaveDamaged>(shockwave).unwrap();
    assert!(
        damaged.0.contains(&cell),
        "cell entity should be in ShockwaveDamaged set"
    );
}

// -- Behavior 2: Shockwave does not damage a cell outside its radius ──

#[test]
fn shockwave_does_not_damage_cell_outside_radius() {
    let mut app = damage_test_app();

    spawn_test_cell(&mut app, 200.0, 0.0);
    let shockwave = spawn_shockwave(&mut app, 0.0, 0.0, 35.0, HashSet::new());

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "no DamageCell messages should be sent for cell outside radius"
    );

    let damaged = app.world().get::<ShockwaveDamaged>(shockwave).unwrap();
    assert!(damaged.0.is_empty(), "ShockwaveDamaged should remain empty");
}

// -- Behavior 3: Shockwave does not damage the same cell twice ──

#[test]
fn shockwave_does_not_damage_already_damaged_cell() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 20.0, 0.0);
    let mut already_damaged = HashSet::new();
    already_damaged.insert(cell);
    let _shockwave = spawn_shockwave(&mut app, 0.0, 0.0, 35.0, already_damaged);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "already-damaged cell should not receive DamageCell again"
    );
}

#[test]
fn shockwave_damages_new_cell_but_not_already_damaged_cell() {
    let mut app = damage_test_app();

    let cell_already = spawn_test_cell(&mut app, 20.0, 0.0);
    let cell_new = spawn_test_cell(&mut app, 25.0, 0.0);

    let mut already_damaged = HashSet::new();
    already_damaged.insert(cell_already);
    let _shockwave = spawn_shockwave(&mut app, 0.0, 0.0, 35.0, already_damaged);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "only the new cell should be damaged, got {} messages",
        collector.0.len()
    );
    assert_eq!(
        collector.0[0].cell, cell_new,
        "only the new cell should receive damage"
    );
}

// -- Behavior 4: Shockwave damages multiple cells in range ──

#[test]
fn shockwave_damages_multiple_cells_in_range() {
    let mut app = damage_test_app();

    let cell1 = spawn_test_cell(&mut app, 10.0, 0.0);
    let cell2 = spawn_test_cell(&mut app, 0.0, 15.0);
    let cell3 = spawn_test_cell(&mut app, -20.0, 0.0);
    let shockwave = spawn_shockwave(&mut app, 0.0, 0.0, 25.0, HashSet::new());

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        3,
        "expected 3 DamageCell messages, got {}",
        collector.0.len()
    );

    let damaged_cells: HashSet<Entity> = collector.0.iter().map(|m| m.cell).collect();
    assert!(damaged_cells.contains(&cell1), "cell1 should be damaged");
    assert!(damaged_cells.contains(&cell2), "cell2 should be damaged");
    assert!(damaged_cells.contains(&cell3), "cell3 should be damaged");

    for msg in &collector.0 {
        assert!(
            (msg.damage - BASE_BOLT_DAMAGE).abs() < f32::EPSILON,
            "each cell damage should be BASE_BOLT_DAMAGE (10.0)"
        );
    }

    let sw_damaged = app.world().get::<ShockwaveDamaged>(shockwave).unwrap();
    assert_eq!(
        sw_damaged.0.len(),
        3,
        "all three cells should be in ShockwaveDamaged set"
    );
}

#[test]
fn shockwave_only_damages_cell_layer_not_wall_layer() {
    let mut app = damage_test_app();

    // Spawn two cells on CELL_LAYER
    spawn_test_cell(&mut app, 10.0, 0.0);
    spawn_test_cell(&mut app, 0.0, 15.0);

    // Spawn one entity on WALL_LAYER (not CELL_LAYER)
    let wall_pos = Vec2::new(-20.0, 0.0);
    app.world_mut().spawn((
        Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
        CollisionLayers::new(WALL_LAYER, 0),
        Position2D(wall_pos),
        GlobalPosition2D(wall_pos),
        Spatial2D,
    ));

    spawn_shockwave(&mut app, 0.0, 0.0, 25.0, HashSet::new());

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        2,
        "only CELL_LAYER entities should be damaged, got {}",
        collector.0.len()
    );
}

// -- Behavior 5: Multiple shockwaves damage independently ──

#[test]
fn multiple_shockwaves_damage_independently() {
    let mut app = damage_test_app();

    let cell_near_sw1 = spawn_test_cell(&mut app, 15.0, 0.0);
    let cell_near_sw2 = spawn_test_cell(&mut app, 90.0, 0.0);

    let sw1 = spawn_shockwave(&mut app, 0.0, 0.0, 25.0, HashSet::new());
    let sw2 = spawn_shockwave(&mut app, 100.0, 0.0, 25.0, HashSet::new());

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        2,
        "expected 2 DamageCell messages (one per shockwave), got {}",
        collector.0.len()
    );

    let damaged_cells: HashSet<Entity> = collector.0.iter().map(|m| m.cell).collect();
    assert!(
        damaged_cells.contains(&cell_near_sw1),
        "cell near sw1 should be damaged"
    );
    assert!(
        damaged_cells.contains(&cell_near_sw2),
        "cell near sw2 should be damaged"
    );

    let sw1_damaged = app.world().get::<ShockwaveDamaged>(sw1).unwrap();
    assert!(
        sw1_damaged.0.contains(&cell_near_sw1),
        "sw1 should track cell_near_sw1"
    );
    assert!(
        !sw1_damaged.0.contains(&cell_near_sw2),
        "sw1 should NOT track cell_near_sw2"
    );

    let sw2_damaged = app.world().get::<ShockwaveDamaged>(sw2).unwrap();
    assert!(
        sw2_damaged.0.contains(&cell_near_sw2),
        "sw2 should track cell_near_sw2"
    );
    assert!(
        !sw2_damaged.0.contains(&cell_near_sw1),
        "sw2 should NOT track cell_near_sw1"
    );
}

#[test]
fn cell_midway_between_two_shockwaves_not_reached_by_either() {
    let mut app = damage_test_app();

    // Cell at midpoint (50, 0) -- neither shockwave with radius 25 reaches it
    spawn_test_cell(&mut app, 50.0, 0.0);
    spawn_shockwave(&mut app, 0.0, 0.0, 25.0, HashSet::new());
    spawn_shockwave(&mut app, 100.0, 0.0, 25.0, HashSet::new());

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "cell at midpoint should not be reached by either shockwave"
    );
}

// -- Behavior 6: Shockwave with zero radius damages no cells ──

#[test]
fn shockwave_with_zero_radius_damages_no_cells() {
    let mut app = damage_test_app();

    spawn_test_cell(&mut app, 1.0, 0.0);
    spawn_shockwave(&mut app, 0.0, 0.0, 0.0, HashSet::new());

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "shockwave with radius 0.0 should damage no cells"
    );
}

#[test]
fn shockwave_with_zero_radius_at_same_position_as_cell_no_damage() {
    let mut app = damage_test_app();

    // Cell at same position as shockwave -- radius 0 means no matches
    spawn_test_cell(&mut app, 0.0, 0.0);
    spawn_shockwave(&mut app, 0.0, 0.0, 0.0, HashSet::new());

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "shockwave with radius 0.0 at cell position should still damage no cells"
    );
}

// -- Damage scaling: Shockwave damage scales by ShockwaveDamageMultiplier ──

#[test]
fn shockwave_damage_scales_by_effective_damage_multiplier() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 20.0, 0.0);

    // Spawn shockwave with ShockwaveDamageMultiplier(2.0)
    app.world_mut().spawn((
        ShockwaveSource(Entity::PLACEHOLDER),
        ShockwaveRadius(35.0),
        ShockwaveMaxRadius(100.0),
        ShockwaveSpeed(50.0),
        ShockwaveDamaged(HashSet::new()),
        ShockwaveDamageMultiplier(2.0),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected exactly one DamageCell message, got {}",
        collector.0.len()
    );
    assert_eq!(collector.0[0].cell, cell);

    let expected_damage = BASE_BOLT_DAMAGE * 2.0;
    assert!(
        (collector.0[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {} (BASE_BOLT_DAMAGE * 2.0), got {}",
        expected_damage,
        collector.0[0].damage
    );
}

// -- Damage scaling: High multiplier across multiple cells ──

#[test]
fn shockwave_damage_scales_with_high_multiplier_across_multiple_cells() {
    let mut app = damage_test_app();

    let cell1 = spawn_test_cell(&mut app, 10.0, 0.0);
    let cell2 = spawn_test_cell(&mut app, 0.0, 15.0);
    let cell3 = spawn_test_cell(&mut app, -20.0, 0.0);

    app.world_mut().spawn((
        ShockwaveSource(Entity::PLACEHOLDER),
        ShockwaveRadius(25.0),
        ShockwaveMaxRadius(100.0),
        ShockwaveSpeed(50.0),
        ShockwaveDamaged(HashSet::new()),
        ShockwaveDamageMultiplier(3.5),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        3,
        "expected 3 DamageCell messages, got {}",
        collector.0.len()
    );

    let damaged_cells: HashSet<Entity> = collector.0.iter().map(|m| m.cell).collect();
    assert!(damaged_cells.contains(&cell1), "cell1 should be damaged");
    assert!(damaged_cells.contains(&cell2), "cell2 should be damaged");
    assert!(damaged_cells.contains(&cell3), "cell3 should be damaged");

    let expected_damage = BASE_BOLT_DAMAGE * 3.5;
    for msg in &collector.0 {
        assert!(
            (msg.damage - expected_damage).abs() < f32::EPSILON,
            "each cell damage should be BASE_BOLT_DAMAGE * 3.5 = {}, got {}",
            expected_damage,
            msg.damage
        );
    }
}

#[test]
fn shockwave_damage_zero_multiplier_produces_zero_damage() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 10.0, 0.0);

    app.world_mut().spawn((
        ShockwaveSource(Entity::PLACEHOLDER),
        ShockwaveRadius(25.0),
        ShockwaveMaxRadius(100.0),
        ShockwaveSpeed(50.0),
        ShockwaveDamaged(HashSet::new()),
        ShockwaveDamageMultiplier(0.0),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected one DamageCell even with zero multiplier"
    );
    assert_eq!(collector.0[0].cell, cell);
    assert!(
        (collector.0[0].damage - 0.0).abs() < f32::EPSILON,
        "zero multiplier should produce zero damage, got {}",
        collector.0[0].damage
    );
}

// -- Section C: EffectSourceChip attribution tests ───────────────────

use crate::effect::core::EffectSourceChip;

#[test]
fn fire_stores_effect_source_chip_with_non_empty_chip_name() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(100.0, 200.0, 0.0)).id();

    fire(entity, 24.0, 8.0, 1, 50.0, "seismic", &mut world);

    let mut query = world.query::<&EffectSourceChip>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        results.len(),
        1,
        "expected one entity with EffectSourceChip"
    );
    assert_eq!(
        results[0].0,
        Some("seismic".to_string()),
        "spawned shockwave should have EffectSourceChip(Some(\"seismic\"))"
    );
}

#[test]
fn fire_stores_effect_source_chip_none_with_empty_chip_name() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

    fire(entity, 24.0, 8.0, 1, 50.0, "", &mut world);

    let mut query = world.query::<&EffectSourceChip>();
    let results: Vec<_> = query.iter(&world).collect();
    assert_eq!(
        results.len(),
        1,
        "expected one entity with EffectSourceChip"
    );
    assert_eq!(
        results[0].0, None,
        "empty source_chip should produce EffectSourceChip(None)"
    );
}

#[test]
fn apply_shockwave_damage_populates_source_chip_from_effect_source_chip() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 20.0, 0.0);

    app.world_mut().spawn((
        ShockwaveSource(Entity::PLACEHOLDER),
        ShockwaveRadius(35.0),
        ShockwaveMaxRadius(100.0),
        ShockwaveSpeed(50.0),
        ShockwaveDamaged(HashSet::new()),
        EffectSourceChip(Some("seismic".to_string())),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1, "expected one DamageCell message");
    assert_eq!(collector.0[0].cell, cell);
    assert_eq!(
        collector.0[0].source_chip,
        Some("seismic".to_string()),
        "DamageCell should have source_chip from EffectSourceChip"
    );
}

#[test]
fn apply_shockwave_damage_source_chip_none_when_effect_source_chip_none() {
    let mut app = damage_test_app();

    spawn_test_cell(&mut app, 20.0, 0.0);

    app.world_mut().spawn((
        ShockwaveSource(Entity::PLACEHOLDER),
        ShockwaveRadius(35.0),
        ShockwaveMaxRadius(100.0),
        ShockwaveSpeed(50.0),
        ShockwaveDamaged(HashSet::new()),
        EffectSourceChip(None),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(
        collector.0[0].source_chip, None,
        "EffectSourceChip(None) should produce source_chip None"
    );
}

#[test]
fn apply_shockwave_damage_defaults_to_none_when_no_effect_source_chip_component() {
    let mut app = damage_test_app();

    spawn_test_cell(&mut app, 20.0, 0.0);

    // No EffectSourceChip component on shockwave
    app.world_mut().spawn((
        ShockwaveSource(Entity::PLACEHOLDER),
        ShockwaveRadius(35.0),
        ShockwaveMaxRadius(100.0),
        ShockwaveSpeed(50.0),
        ShockwaveDamaged(HashSet::new()),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(
        collector.0[0].source_chip, None,
        "missing EffectSourceChip should default to source_chip None"
    );
}

#[test]
fn multiple_shockwaves_with_different_source_chips_produce_correctly_attributed_damage() {
    let mut app = damage_test_app();

    let cell_a = spawn_test_cell(&mut app, 15.0, 0.0);
    let cell_b = spawn_test_cell(&mut app, 90.0, 0.0);

    app.world_mut().spawn((
        ShockwaveSource(Entity::PLACEHOLDER),
        ShockwaveRadius(25.0),
        ShockwaveMaxRadius(100.0),
        ShockwaveSpeed(50.0),
        ShockwaveDamaged(HashSet::new()),
        EffectSourceChip(Some("alpha".to_string())),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    app.world_mut().spawn((
        ShockwaveSource(Entity::PLACEHOLDER),
        ShockwaveRadius(25.0),
        ShockwaveMaxRadius(100.0),
        ShockwaveSpeed(50.0),
        ShockwaveDamaged(HashSet::new()),
        EffectSourceChip(Some("beta".to_string())),
        Transform::from_xyz(100.0, 0.0, 0.0),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        2,
        "expected 2 DamageCell messages, got {}",
        collector.0.len()
    );

    let msg_a = collector.0.iter().find(|m| m.cell == cell_a).unwrap();
    assert_eq!(
        msg_a.source_chip,
        Some("alpha".to_string()),
        "cell near shockwave A should have source_chip alpha"
    );

    let msg_b = collector.0.iter().find(|m| m.cell == cell_b).unwrap();
    assert_eq!(
        msg_b.source_chip,
        Some("beta".to_string()),
        "cell near shockwave B should have source_chip beta"
    );
}
