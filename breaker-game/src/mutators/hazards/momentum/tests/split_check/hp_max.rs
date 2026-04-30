//! Group F addendum — hp.max preservation on split (Behavior 71).

use bevy::prelude::*;

use super::{
    super::{
        super::system::MomentumConfig,
        helpers::{all_cells, canonical_momentum_config, run_fixed_update, spawn_cell_at_with_max},
    },
    helpers::split_test_app,
};
use crate::{
    prelude::*,
    shared::collision_layers::{BOLT_LAYER, CELL_LAYER},
};

#[test]
fn split_does_not_reset_hp_max_on_parent() {
    let mut app = split_test_app();
    let parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 20.0, 10.0, Some(20.0));

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(parent).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "current resets to starting; got {}",
        hp.current
    );
    assert!(
        (hp.starting - 10.0).abs() < f32::EPSILON,
        "starting is unchanged"
    );
    assert_eq!(
        hp.max,
        Some(20.0),
        "hp.max must be PRESERVED at Some(20.0) after split; got {:?}",
        hp.max
    );
}

#[test]
fn split_preserves_larger_existing_hp_max() {
    let mut app = split_test_app();
    let parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 30.0, 10.0, Some(50.0));

    run_fixed_update(&mut app);

    let hp = app.world().get::<Hp>(parent).unwrap();
    assert!((hp.current - 10.0).abs() < f32::EPSILON);
    assert_eq!(
        hp.max,
        Some(50.0),
        "larger pre-existing hp.max must be preserved at Some(50.0); got {:?}",
        hp.max
    );
}

#[test]
fn spawned_cells_have_max_none_after_split() {
    let mut app = split_test_app();
    let parent = spawn_cell_at_with_max(&mut app, Vec2::ZERO, 20.0, 10.0, Some(20.0));

    run_fixed_update(&mut app);

    let cells = all_cells(&mut app);
    for (e, _, hp) in &cells {
        if *e != parent {
            assert_eq!(
                hp.max, None,
                "spawned cells must have hp.max = None (ceiling lift is attach_momentum_ceiling's job next tick); got {:?}",
                hp.max
            );
        }
    }
    // Touch imports.
    let _ = CELL_LAYER;
    let _ = BOLT_LAYER;
    let _: MomentumConfig = canonical_momentum_config();
}
