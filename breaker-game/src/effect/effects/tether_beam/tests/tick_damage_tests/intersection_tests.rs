//! Tests for beam-cell intersection geometry: segment vs AABB, multiple cells, dedup.

use super::super::helpers::*;

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
