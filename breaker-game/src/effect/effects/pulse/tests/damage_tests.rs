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
        Transform::from_xyz(0.0, 0.0, 0.0),
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
        Transform::from_xyz(0.0, 0.0, 0.0),
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
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    app.world_mut().spawn((
        PulseRing,
        PulseRadius(25.0),
        PulseMaxRadius(50.0),
        PulseSpeed(0.0),
        PulseDamaged::default(),
        Transform::from_xyz(0.0, 0.0, 0.0),
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
        Transform::from_xyz(0.0, 0.0, 0.0),
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
        Transform::from_xyz(0.0, 0.0, 0.0),
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
