use super::*;

// ── Behavior 16: apply_pulse_damage damages cells within radius ──

#[test]
fn pulse_ring_damages_cell_within_radius() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 20.0, 0.0);

    app.world_mut().spawn((
        PulseRing,
        PulseRadius(25.0),
        PulseMaxRadius(50.0),
        PulseSpeed(0.0),
        PulseDamaged::default(),
        Position2D(Vec2::ZERO),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected one DamageCell message, got {}",
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
        "source_chip should be None for pulse damage"
    );
}

#[test]
fn pulse_ring_does_not_damage_already_damaged_cell() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 20.0, 0.0);

    let mut already_damaged = HashSet::new();
    already_damaged.insert(cell);

    app.world_mut().spawn((
        PulseRing,
        PulseRadius(25.0),
        PulseMaxRadius(50.0),
        PulseSpeed(0.0),
        PulseDamaged(already_damaged),
        Position2D(Vec2::ZERO),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "already-damaged cell should not receive DamageCell again"
    );
}

// ── Behavior 17: Each PulseRing damages cells independently ──

#[test]
fn each_pulse_ring_damages_cells_independently() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 15.0, 0.0);

    // Two rings at the same position, each with empty PulseDamaged
    app.world_mut().spawn((
        PulseRing,
        PulseRadius(25.0),
        PulseMaxRadius(50.0),
        PulseSpeed(0.0),
        PulseDamaged::default(),
        Position2D(Vec2::ZERO),
    ));
    app.world_mut().spawn((
        PulseRing,
        PulseRadius(25.0),
        PulseMaxRadius(50.0),
        PulseSpeed(0.0),
        PulseDamaged::default(),
        Position2D(Vec2::ZERO),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        2,
        "each ring should send its own DamageCell, expected 2, got {}",
        collector.0.len()
    );

    // Both messages should reference the same cell
    assert_eq!(collector.0[0].cell, cell);
    assert_eq!(collector.0[1].cell, cell);
}

// ── Behavior 18: Pulse ring does not damage non-CELL_LAYER entities ──

#[test]
fn pulse_ring_does_not_damage_non_cell_layer_entities() {
    let mut app = damage_test_app();

    // Spawn a bolt-layer entity (not a cell)
    let bolt_pos = Vec2::new(10.0, 0.0);
    app.world_mut().spawn((
        Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
        CollisionLayers::new(BOLT_LAYER, 0),
        Position2D(bolt_pos),
        GlobalPosition2D(bolt_pos),
        Spatial2D,
    ));

    // Spawn a wall-layer entity (not a cell)
    let wall_pos = Vec2::new(5.0, 0.0);
    app.world_mut().spawn((
        Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
        CollisionLayers::new(WALL_LAYER, 0),
        Position2D(wall_pos),
        GlobalPosition2D(wall_pos),
        Spatial2D,
    ));

    app.world_mut().spawn((
        PulseRing,
        PulseRadius(25.0),
        PulseMaxRadius(50.0),
        PulseSpeed(0.0),
        PulseDamaged::default(),
        Position2D(Vec2::ZERO),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "non-CELL_LAYER entities should not receive damage"
    );
}

#[test]
fn pulse_ring_damages_entity_with_cell_layer_in_combined_mask() {
    let mut app = damage_test_app();

    // Entity with CELL_LAYER | WALL_LAYER -- should be damaged since it IS on CELL_LAYER
    let pos = Vec2::new(10.0, 0.0);
    let cell = app
        .world_mut()
        .spawn((
            Cell,
            Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
            CollisionLayers::new(CELL_LAYER | WALL_LAYER, 0),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
        ))
        .id();

    app.world_mut().spawn((
        PulseRing,
        PulseRadius(25.0),
        PulseMaxRadius(50.0),
        PulseSpeed(0.0),
        PulseDamaged::default(),
        Position2D(Vec2::ZERO),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "entity with CELL_LAYER in combined mask should be damaged"
    );
    assert_eq!(collector.0[0].cell, cell);
}

// ── Damage scaling: Pulse ring damage scales by PulseRingDamageMultiplier ──

#[test]
fn pulse_ring_damage_scales_by_effective_damage_multiplier() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 20.0, 0.0);

    app.world_mut().spawn((
        PulseRing,
        PulseRadius(25.0),
        PulseMaxRadius(50.0),
        PulseSpeed(0.0),
        PulseDamaged::default(),
        PulseRingDamageMultiplier(1.5),
        Position2D(Vec2::ZERO),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected one DamageCell message, got {}",
        collector.0.len()
    );
    assert_eq!(collector.0[0].cell, cell);

    let expected_damage = BASE_BOLT_DAMAGE * 1.5;
    assert!(
        (collector.0[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {} (BASE_BOLT_DAMAGE * 1.5), got {}",
        expected_damage,
        collector.0[0].damage
    );
}

#[test]
fn pulse_ring_damage_zero_multiplier_produces_zero_damage() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 20.0, 0.0);

    app.world_mut().spawn((
        PulseRing,
        PulseRadius(25.0),
        PulseMaxRadius(50.0),
        PulseSpeed(0.0),
        PulseDamaged::default(),
        PulseRingDamageMultiplier(0.0),
        Position2D(Vec2::ZERO),
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

// -- Section D: EffectSourceChip attribution tests ───────────────────

use crate::effect::core::EffectSourceChip;

#[test]
fn pulse_fire_stores_effect_source_chip_with_non_empty_chip_name() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(100.0, 200.0, 0.0)).id();

    fire(
        entity,
        PulseEmitter {
            base_range: 32.0,
            range_per_level: 8.0,
            stacks: 1,
            speed: 50.0,
            interval: 0.5,
            timer: 0.0,
        },
        "resonance",
        &mut world,
    );

    let source_chip = world
        .get::<EffectSourceChip>(entity)
        .expect("entity should have EffectSourceChip after fire()");
    assert_eq!(
        source_chip.0,
        Some("resonance".to_string()),
        "fire() with non-empty source_chip should store EffectSourceChip(Some(...))"
    );
}

#[test]
fn pulse_fire_stores_effect_source_chip_none_with_empty_chip_name() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

    fire(
        entity,
        PulseEmitter {
            base_range: 32.0,
            range_per_level: 8.0,
            stacks: 1,
            speed: 50.0,
            interval: 0.5,
            timer: 0.0,
        },
        "",
        &mut world,
    );

    let source_chip = world
        .get::<EffectSourceChip>(entity)
        .expect("entity should have EffectSourceChip after fire()");
    assert_eq!(
        source_chip.0, None,
        "fire() with empty source_chip should store EffectSourceChip(None)"
    );
}

#[test]
fn apply_pulse_damage_populates_source_chip_from_ring_effect_source_chip() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 20.0, 0.0);

    app.world_mut().spawn((
        PulseRing,
        PulseRadius(25.0),
        PulseMaxRadius(50.0),
        PulseSpeed(0.0),
        PulseDamaged::default(),
        EffectSourceChip(Some("resonance".to_string())),
        Position2D(Vec2::ZERO),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(collector.0[0].cell, cell);
    assert_eq!(
        collector.0[0].source_chip,
        Some("resonance".to_string()),
        "DamageCell should have source_chip from ring's EffectSourceChip"
    );
}

#[test]
fn apply_pulse_damage_source_chip_none_when_no_effect_source_chip_on_ring() {
    let mut app = damage_test_app();

    spawn_test_cell(&mut app, 20.0, 0.0);

    // No EffectSourceChip on the ring
    app.world_mut().spawn((
        PulseRing,
        PulseRadius(25.0),
        PulseMaxRadius(50.0),
        PulseSpeed(0.0),
        PulseDamaged::default(),
        Position2D(Vec2::ZERO),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(
        collector.0[0].source_chip, None,
        "missing EffectSourceChip on ring should default to source_chip None"
    );
}

// ── Behavior: apply_pulse_damage uses Position2D not Transform when both present ──

#[test]
fn apply_pulse_damage_uses_position2d_not_transform_when_both_present() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 20.0, 0.0);

    // PulseRing at Position2D origin (0,0), but Transform at (500,500) — divergent.
    // If the system reads Position2D, cell at (20,0) is within radius 25.
    // If the system incorrectly reads Transform, cell would be ~500+ units away — outside radius.
    app.world_mut().spawn((
        PulseRing,
        PulseRadius(25.0),
        PulseMaxRadius(50.0),
        PulseSpeed(0.0),
        PulseDamaged::default(),
        Position2D(Vec2::ZERO),
        Transform::from_xyz(500.0, 500.0, 0.0),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "cell at (20,0) should be within radius 25 of Position2D (0,0), got {} messages",
        collector.0.len()
    );
    assert_eq!(
        collector.0[0].cell, cell,
        "DamageCell should target the cell within Position2D-based radius"
    );
}
