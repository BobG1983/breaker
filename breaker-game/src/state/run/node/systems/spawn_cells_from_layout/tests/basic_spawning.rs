use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;
use rantzsoft_stateflow::CleanupOnExit;

use super::{super::system::grid_extent, helpers::*};
use crate::{
    cells::{components::*, resources::CellConfig},
    shared::PlayfieldConfig,
    state::{
        run::node::{NodeLayout, definition::NodePool, messages::CellsSpawned},
        types::NodeState,
    },
};

/// Helper to reduce verbosity of String grid construction.
fn s(val: &str) -> String {
    val.to_owned()
}

#[test]
fn correct_cell_count_full_layout() {
    let layout = full_layout();
    let expected = layout.cell_count();
    let mut app = test_app(layout);
    app.update();

    let count = app.world_mut().query::<&Cell>().iter(app.world()).count();
    assert_eq!(count, expected);
    assert_eq!(count, 6);
}

#[test]
fn dot_slots_produce_no_entities() {
    let layout = sparse_layout();
    let total_slots = (layout.cols * layout.rows) as usize;
    let mut app = test_app(layout);
    app.update();

    let count = app.world_mut().query::<&Cell>().iter(app.world()).count();
    assert_eq!(count, 3);
    assert!(count < total_slots, "dots should not spawn cells");
}

#[test]
fn cells_get_hp_from_type_definition() {
    let layout = full_layout();
    let mut app = test_app(layout);
    app.update();

    let mut found_standard = false;
    let mut found_tough = false;
    for health in app.world_mut().query::<&CellHealth>().iter(app.world()) {
        if (health.max - 20.0).abs() < f32::EPSILON {
            found_standard = true;
        }
        if (health.max - 30.0).abs() < f32::EPSILON {
            found_tough = true;
        }
    }
    assert!(
        found_standard,
        "should have standard cells (Standard toughness base=20.0)"
    );
    assert!(
        found_tough,
        "should have tough cells (Tough toughness base=30.0)"
    );
}

#[test]
fn required_to_clear_present_when_true() {
    let layout = full_layout();
    let mut app = test_app(layout);
    app.update();

    let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
    let required_count = app
        .world_mut()
        .query::<(&Cell, &RequiredToClear)>()
        .iter(app.world())
        .count();
    assert_eq!(cell_count, required_count);
}

#[test]
fn spawn_cells_sends_cells_spawned_message() {
    let mut app = test_app(full_layout());
    app.update();

    let messages = app.world().resource::<Messages<CellsSpawned>>();
    assert!(
        messages.iter_current_update_messages().count() > 0,
        "spawn_cells_from_layout must send CellsSpawned message"
    );
}

#[test]
fn all_cells_have_cleanup_marker() {
    let layout = full_layout();
    let mut app = test_app(layout);
    app.update();

    let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
    let marked_count = app
        .world_mut()
        .query::<(&Cell, &CleanupOnExit<NodeState>)>()
        .iter(app.world())
        .count();
    assert_eq!(cell_count, marked_count);
}

#[test]
fn all_cells_within_playfield() {
    let layout = full_layout();
    let config = CellConfig::default();
    let playfield = PlayfieldConfig::default();
    let mut app = test_app(layout);
    app.update();

    for position in app
        .world_mut()
        .query_filtered::<&Position2D, With<Cell>>()
        .iter(app.world())
    {
        let x = position.0.x;
        let y = position.0.y;
        assert!(
            x.abs() < playfield.right() + config.width / 2.0,
            "cell x={x} out of bounds"
        );
        assert!(
            y < playfield.top() + config.height / 2.0,
            "cell y={y} above playfield"
        );
    }
}

#[test]
fn all_cells_have_dimensions_and_damage_visuals() {
    let layout = full_layout();
    let mut app = test_app(layout);
    app.update();

    let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
    let with_dims = app
        .world_mut()
        .query::<(&Cell, &CellWidth, &CellHeight, &CellDamageVisuals)>()
        .iter(app.world())
        .count();
    assert_eq!(cell_count, with_dims);
}

#[test]
fn unrecognized_alias_produces_no_entity() {
    let layout = NodeLayout {
        name:            "unknown".to_owned(),
        timer_secs:      60.0,
        cols:            3,
        rows:            1,
        grid_top_offset: 50.0,
        grid:            vec![vec![s("S"), s("X"), s("S")]], // "X" not in registry
        pool:            NodePool::default(),
        entity_scale:    1.0,
        locks:           None,
    };
    let mut app = test_app(layout);
    app.update();

    let count = app.world_mut().query::<&Cell>().iter(app.world()).count();
    assert_eq!(
        count, 2,
        "unrecognized alias 'X' should be silently skipped, only 2 cells spawned"
    );
}

// --- Cell position tests ---

#[test]
fn grid_is_horizontally_centered() {
    let layout = full_layout();
    let config = CellConfig::default();
    let step_x = config.width + config.padding_x;
    let mut app = test_app(layout.clone());
    app.update();

    // Grid should be centered: sum of all x positions per row should be ~0
    // With 3 columns the positions should be symmetric around 0
    let cols_f = f32::from(u16::try_from(layout.cols).unwrap_or(u16::MAX));
    let grid_width = grid_extent(step_x, cols_f, config.padding_x);
    let expected_start = -grid_width / 2.0 + config.width / 2.0;
    let expected_end = step_x.mul_add(cols_f - 1.0, expected_start);
    let center = f32::midpoint(expected_start, expected_end);

    assert!(
        center.abs() < 1.0,
        "grid center should be near 0, got {center:.2}"
    );
}

#[test]
fn cell_positions_match_grid_coordinates() {
    let layout = full_layout();
    let config = CellConfig::default();
    let playfield = PlayfieldConfig::default();
    let step_x = config.width + config.padding_x;
    let step_y = config.height + config.padding_y;
    let mut app = test_app(layout.clone());
    app.update();

    let grid_width = grid_extent(
        step_x,
        f32::from(u16::try_from(layout.cols).unwrap_or(u16::MAX)),
        config.padding_x,
    );
    let start_x = -grid_width / 2.0 + config.width / 2.0;
    let start_y = playfield.top() - layout.grid_top_offset - config.height / 2.0;

    let positions = collect_sorted_cell_positions(&mut app);

    // full_layout: row 0 = [T, S, S], row 1 = [S, S, S]
    let expected: Vec<(f32, f32)> = vec![
        // Row 0
        (start_x, start_y),
        (start_x + step_x, start_y),
        (step_x.mul_add(2.0, start_x), start_y),
        // Row 1
        (start_x, start_y - step_y),
        (start_x + step_x, start_y - step_y),
        (step_x.mul_add(2.0, start_x), start_y - step_y),
    ];

    assert_positions_match(&positions, &expected);
}

#[test]
fn sparse_layout_positions_skip_dots() {
    let layout = sparse_layout();
    let config = CellConfig::default();
    let playfield = PlayfieldConfig::default();
    let step_x = config.width + config.padding_x;
    let step_y = config.height + config.padding_y;
    let mut app = test_app(layout.clone());
    app.update();

    let grid_width = grid_extent(
        step_x,
        f32::from(u16::try_from(layout.cols).unwrap_or(u16::MAX)),
        config.padding_x,
    );
    let start_x = -grid_width / 2.0 + config.width / 2.0;
    let start_y = playfield.top() - layout.grid_top_offset - config.height / 2.0;

    let positions = collect_sorted_cell_positions(&mut app);

    // sparse_layout: row 0 = [., S, .], row 1 = [T, ., S]
    let expected: Vec<(f32, f32)> = vec![
        (start_x + step_x, start_y),                      // row 0, col 1
        (start_x, start_y - step_y),                      // row 1, col 0
        (step_x.mul_add(2.0, start_x), start_y - step_y), // row 1, col 2
    ];

    assert_positions_match(&positions, &expected);
}

#[test]
fn all_cells_have_cell_type_alias() {
    let layout = full_layout();
    let mut app = test_app(layout);
    app.update();

    let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
    let alias_count = app
        .world_mut()
        .query::<(&Cell, &CellTypeAlias)>()
        .iter(app.world())
        .count();
    assert_eq!(
        cell_count, alias_count,
        "every cell should have a CellTypeAlias"
    );
}

#[test]
fn cell_type_alias_matches_grid_string() {
    // full_layout: row 0 = ["T", "S", "S"], row 1 = ["S", "S", "S"] -> 1 T, 5 S
    let layout = full_layout();
    let mut app = test_app(layout);
    app.update();

    let mut t_count = 0;
    let mut s_count = 0;
    for alias in app.world_mut().query::<&CellTypeAlias>().iter(app.world()) {
        match alias.0.as_str() {
            "T" => t_count += 1,
            "S" => s_count += 1,
            other => panic!("unexpected alias '{other}'"),
        }
    }
    assert_eq!(t_count, 1, "should have 1 tough cell");
    assert_eq!(s_count, 5, "should have 5 standard cells");
}
