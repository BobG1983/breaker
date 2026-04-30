use super::{super::helpers::*, helpers::*};
use crate::{
    cells::{components::*, resources::CellConfig},
    prelude::*,
    state::run::node::definition::NodePool,
};

// NOTE: locked_cell_definition_spawns_with_locked_component,
// non_locked_cell_does_not_have_locked_component, and
// locked_cell_definition_spawns_with_lock_adjacents_component
// have been REMOVED — locking is no longer driven by CellBehavior.

#[test]
fn regen_cell_definition_spawns_with_cell_regen_component() {
    let layout = NodeLayout {
        name:            "regen_test".to_owned(),
        timer_secs:      60.0,
        cols:            2,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("R"), s("N")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           None,
        sequences:       None,
    };
    let mut app = behavior_test_app(layout, behavior_registry());
    app.update();

    let regen_cells: Vec<&RegenRate> = app
        .world_mut()
        .query::<(&Cell, &RegenRate)>()
        .iter(app.world())
        .map(|(_, regen)| regen)
        .collect();
    assert_eq!(
        regen_cells.len(),
        1,
        "cell with behaviors: [Regen {{ rate: 2.0 }}] should have RegenRate component"
    );
    assert!(
        (regen_cells[0].0 - 2.0).abs() < f32::EPSILON,
        "RegenRate rate should be 2.0, got {}",
        regen_cells[0].0
    );
}

#[test]
fn non_regen_cell_does_not_have_cell_regen_component() {
    let layout = NodeLayout {
        name:            "no_regen_test".to_owned(),
        timer_secs:      60.0,
        cols:            2,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("N"), s("N")]],
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           None,
        sequences:       None,
    };
    let mut app = behavior_test_app(layout, behavior_registry());
    app.update();

    let regen_count = app
        .world_mut()
        .query::<(&Cell, &RegenRate)>()
        .iter(app.world())
        .count();
    assert_eq!(
        regen_count, 0,
        "cells with behaviors: None should NOT have RegenRate component"
    );
}

#[test]
fn cell_spacing_matches_config() {
    let layout = full_layout();
    let config = CellConfig::default();
    let step_x = config.width + config.padding_x;
    let step_y = config.height + config.padding_y;
    let mut app = test_app(layout);
    app.update();

    let positions = collect_sorted_cell_positions(&mut app);

    // Check horizontal spacing within row 0 (first 3 cells)
    let dx_01 = positions[1].0 - positions[0].0;
    assert!(
        (dx_01 - step_x).abs() < 0.01,
        "horizontal spacing should be {step_x}, got {dx_01}"
    );
    let dx_12 = positions[2].0 - positions[1].0;
    assert!(
        (dx_12 - step_x).abs() < 0.01,
        "horizontal spacing should be {step_x}, got {dx_12}"
    );

    // Check vertical spacing between row 0 and row 1 (same column)
    let dy = positions[0].1 - positions[3].1;
    assert!(
        (dy - step_y).abs() < 0.01,
        "vertical spacing should be {step_y}, got {dy}"
    );
}
