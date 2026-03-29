use super::*;

#[test]
fn tick_tether_beam_damages_cell_intersecting_beam_segment() {
    let mut app = damage_test_app();

    let (_bolt_a, _bolt_b, _beam) =
        spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 2.0);
    let cell = spawn_test_cell(&mut app, 50.0, 0.0);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected 1 DamageCell message, got {}",
        collector.0.len()
    );
    assert_eq!(collector.0[0].cell, cell);
    let expected_damage = BASE_BOLT_DAMAGE * 2.0;
    assert!(
        (collector.0[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {expected_damage}, got {}",
        collector.0[0].damage
    );
}

#[test]
fn tick_tether_beam_does_not_damage_cell_not_intersecting() {
    let mut app = damage_test_app();

    // Beam along y=0 from (0,0) to (100,0)
    spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);
    // Cell at (50, 50) with small AABB — does NOT intersect the beam segment at y=0
    spawn_test_cell_with_extents(&mut app, 50.0, 50.0, Vec2::new(5.0, 5.0));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "cell at (50, 50) should not be hit by horizontal beam at y=0"
    );
}

#[test]
fn tick_tether_beam_uses_line_segment_vs_aabb_not_circle() {
    let mut app = damage_test_app();

    // Beam from (0,0) to (100,0) along y=0 with damage_mult=1.0
    spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);

    // Cell at (50, 30) with half_extents (5,5) — AABB spans y=[25,35], does NOT intersect y=0
    spawn_test_cell_with_extents(&mut app, 50.0, 30.0, Vec2::new(5.0, 5.0));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "cell near beam but AABB not intersecting should receive no damage"
    );
}

#[test]
fn tick_tether_beam_cell_aabb_barely_intersects_beam() {
    let mut app = damage_test_app();

    // Beam from (0,0) to (100,0) along y=0
    spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);

    // Cell at (50, 6) with half_extents (5,5) — AABB spans y=[1,11], DOES intersect y=0
    let cell = spawn_test_cell_with_extents(&mut app, 50.0, 6.0, Vec2::new(5.0, 5.0));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "cell at (50, 6) with half_extents (5,5) should intersect beam at y=0"
    );
    assert_eq!(collector.0[0].cell, cell);
}

#[test]
fn tick_tether_beam_damages_multiple_cells_along_beam() {
    let mut app = damage_test_app();

    spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);

    let cell_a = spawn_test_cell(&mut app, 20.0, 0.0);
    let cell_b = spawn_test_cell(&mut app, 60.0, 0.0);
    let cell_c = spawn_test_cell(&mut app, 90.0, 0.0);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        3,
        "expected 3 DamageCell messages, got {}",
        collector.0.len()
    );

    let damaged_cells: HashSet<Entity> = collector.0.iter().map(|m| m.cell).collect();
    assert!(damaged_cells.contains(&cell_a), "cell A should be damaged");
    assert!(damaged_cells.contains(&cell_b), "cell B should be damaged");
    assert!(damaged_cells.contains(&cell_c), "cell C should be damaged");

    for msg in &collector.0 {
        assert!(
            (msg.damage - BASE_BOLT_DAMAGE).abs() < f32::EPSILON,
            "each cell damage should be BASE_BOLT_DAMAGE * 1.0 = 10.0, got {}",
            msg.damage
        );
    }
}

#[test]
fn tick_tether_beam_dedup_damages_cell_at_most_once_per_tick() {
    let mut app = damage_test_app();

    spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);
    let cell = spawn_test_cell(&mut app, 50.0, 0.0);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "cell should be damaged exactly once per tick (dedup), got {}",
        collector.0.len()
    );
    assert_eq!(collector.0[0].cell, cell);
}

#[test]
fn tick_tether_beam_skips_entities_outside_cell_layer() {
    let mut app = damage_test_app();

    spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);

    // Spawn a bolt-layer entity at (50, 0) — should NOT be damaged
    let pos = Vec2::new(50.0, 0.0);
    app.world_mut().spawn((
        Aabb2D::new(Vec2::ZERO, Vec2::new(10.0, 10.0)),
        CollisionLayers::new(BOLT_LAYER, 0),
        Position2D(pos),
        GlobalPosition2D(pos),
        Spatial2D,
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "non-CELL_LAYER entities should not receive DamageCell"
    );
}

#[test]
fn tick_tether_beam_damages_entity_with_cell_layer_in_combined_layers() {
    let mut app = damage_test_app();

    spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);

    // Entity with CELL_LAYER | BOLT_LAYER — IS on CELL_LAYER, so should be damaged
    let pos = Vec2::new(50.0, 0.0);
    let cell = app
        .world_mut()
        .spawn((
            Cell,
            Aabb2D::new(Vec2::ZERO, Vec2::new(10.0, 10.0)),
            CollisionLayers::new(CELL_LAYER | BOLT_LAYER, 0),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
        ))
        .id();

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "entity with CELL_LAYER in combined mask should be damaged"
    );
    assert_eq!(collector.0[0].cell, cell);
}

#[test]
fn tick_tether_beam_zero_length_segment_damages_cell_containing_point() {
    let mut app = damage_test_app();

    // Both bolts at same position — zero-length beam
    spawn_tether_beam(&mut app, Vec2::new(50.0, 50.0), Vec2::new(50.0, 50.0), 1.0);

    // Cell at (50, 50) with AABB half_extents (10, 10) — contains the point
    let cell = spawn_test_cell(&mut app, 50.0, 50.0);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "zero-length beam at cell position should damage the cell"
    );
    assert_eq!(collector.0[0].cell, cell);
}

#[test]
fn tick_tether_beam_zero_length_segment_does_not_damage_distant_cell() {
    let mut app = damage_test_app();

    // Both bolts at same position (50, 50) — zero-length beam
    spawn_tether_beam(&mut app, Vec2::new(50.0, 50.0), Vec2::new(50.0, 50.0), 1.0);

    // Cell at (100, 100) with small AABB (5, 5) — does not contain point (50, 50)
    spawn_test_cell_with_extents(&mut app, 100.0, 100.0, Vec2::new(5.0, 5.0));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert!(
        collector.0.is_empty(),
        "zero-length beam at (50,50) should not damage cell at (100,100)"
    );
}

// ── Damage scaling: Tether beam damage scales by effective_damage_multiplier ──

#[test]
fn tick_tether_beam_damage_scales_by_effective_damage_multiplier() {
    let mut app = damage_test_app();

    // Tether beam with damage_mult=2.0, effective_damage_multiplier=1.5
    let (_bolt_a, _bolt_b, _beam) = spawn_tether_beam_with_edm(
        &mut app,
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 0.0),
        2.0,
        1.5,
    );
    let cell = spawn_test_cell(&mut app, 50.0, 0.0);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected 1 DamageCell message, got {}",
        collector.0.len()
    );
    assert_eq!(collector.0[0].cell, cell);

    // damage = BASE_BOLT_DAMAGE * damage_mult * effective_damage_multiplier
    //        = 10.0 * 2.0 * 1.5 = 30.0
    let expected_damage = BASE_BOLT_DAMAGE * 2.0 * 1.5;
    assert!(
        (collector.0[0].damage - expected_damage).abs() < f32::EPSILON,
        "expected damage {} (10.0 * 2.0 * 1.5), got {}",
        expected_damage,
        collector.0[0].damage
    );
}

#[test]
fn tick_tether_beam_damage_zero_edm_produces_zero() {
    let mut app = damage_test_app();

    // EDM = 0.0 should produce zero damage
    let (_bolt_a, _bolt_b, _beam) = spawn_tether_beam_with_edm(
        &mut app,
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 0.0),
        2.0,
        0.0,
    );
    let cell = spawn_test_cell(&mut app, 50.0, 0.0);

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(
        collector.0.len(),
        1,
        "expected one DamageCell even with zero EDM"
    );
    assert_eq!(collector.0[0].cell, cell);

    // damage = 10.0 * 2.0 * 0.0 = 0.0
    assert!(
        (collector.0[0].damage - 0.0).abs() < f32::EPSILON,
        "zero EDM should produce zero damage, got {}",
        collector.0[0].damage
    );
}
