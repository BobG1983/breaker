//! Tests for zero-length beam edge cases in `tick_tether_beam`.

use super::super::helpers::*;

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
