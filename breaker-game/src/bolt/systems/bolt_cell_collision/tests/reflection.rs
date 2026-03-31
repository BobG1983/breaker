use bevy::prelude::*;

use super::helpers::*;

#[test]
fn vertical_adjacent_cells_no_cascade() {
    let mut app = test_app();
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();
    app.insert_resource(HitCells::default()).add_systems(
        FixedUpdate,
        collect_cell_hits.after(super::super::system::bolt_cell_collision),
    );

    let upper_y = 100.0;
    let lower_y = upper_y - GRID_STEP_Y;
    spawn_cell(&mut app, 0.0, upper_y);
    spawn_cell(&mut app, 0.0, lower_y);

    // Bolt below the upper cell, moving up
    let start_y = upper_y - cc.height / 2.0 - bc.radius - 2.0;
    spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);

    // Two frames -- CCD should prevent cascade
    tick(&mut app);
    tick(&mut app);

    let hits = app.world().resource::<HitCells>();
    assert_eq!(
        hits.0.len(),
        1,
        "bolt should hit only one cell across two frames, not cascade (got {} hits)",
        hits.0.len()
    );
}

#[test]
fn horizontal_adjacent_cells_no_cascade() {
    let mut app = test_app();
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();
    app.insert_resource(HitCells::default()).add_systems(
        FixedUpdate,
        collect_cell_hits.after(super::super::system::bolt_cell_collision),
    );

    let left_x = 0.0;
    let right_x = left_x + GRID_STEP_X;
    let cell_y = 100.0;
    spawn_cell(&mut app, left_x, cell_y);
    spawn_cell(&mut app, right_x, cell_y);

    // Bolt left of right cell, moving right
    let start_x = right_x - cc.width / 2.0 - bc.radius - 2.0;
    spawn_bolt(&mut app, start_x, cell_y, 400.0, 10.0);

    tick(&mut app);
    tick(&mut app);

    let hits = app.world().resource::<HitCells>();
    assert_eq!(
        hits.0.len(),
        1,
        "bolt should hit only one cell across two frames, not cascade (got {} hits)",
        hits.0.len()
    );
}

#[test]
fn grid_entry_from_below_hits_one_cell() {
    let mut app = test_app();
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();
    app.insert_resource(HitCells::default()).add_systems(
        FixedUpdate,
        collect_cell_hits.after(super::super::system::bolt_cell_collision),
    );

    // 3x2 mini-grid at real spacing
    let base_y = 100.0;
    for row in 0..2 {
        for col in 0..3 {
            let x = (f32::from(i16::try_from(col).unwrap_or(0)) - 1.0) * GRID_STEP_X;
            let y = f32::from(i16::try_from(row).unwrap_or(0)).mul_add(GRID_STEP_Y, base_y);
            spawn_cell(&mut app, x, y);
        }
    }

    let start_y = base_y - cc.height / 2.0 - bc.radius - 2.0;
    spawn_bolt(&mut app, 0.0, start_y, 30.0, 400.0);

    tick(&mut app);
    tick(&mut app);
    tick(&mut app);

    let hits = app.world().resource::<HitCells>();
    assert_eq!(
        hits.0.len(),
        1,
        "bolt entering grid should hit exactly 1 cell across 3 frames, not cascade (got {})",
        hits.0.len()
    );
}
