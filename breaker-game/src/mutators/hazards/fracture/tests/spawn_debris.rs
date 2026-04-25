use std::time::Duration;

use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::{
    mutators::hazards::{definition::HazardKind, resources::ActiveHazards},
    prelude::{Hp, *},
};

// ── fracture_on_death ────────────────────────────────────────────────

// Behavior 10 — stack 1 spawns 2 debris at ±(70,0).
#[test]
fn stack_one_spawns_two_debris() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    app.world_mut().insert_resource(FractureConfig {
        base_splits:      2,
        per_level_splits: 1,
    });
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Fracture);

    write_cell_destroyed(&mut app, Vec2::new(100.0, 200.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app
        .world_mut()
        .query::<(&FractureDebris, &Position2D, &Hp)>();
    let debris: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(debris.len(), 2);
    for (_, pos, hp) in &debris {
        // Each debris sits at the victim position plus one of the
        // first two offsets (right, left).
        let offset = pos.0 - Vec2::new(100.0, 200.0);
        assert!(
            (offset - Vec2::new(70.0, 0.0)).length() < 1e-4
                || (offset - Vec2::new(-70.0, 0.0)).length() < 1e-4,
            "debris at unexpected offset: {offset:?}"
        );
        assert!((hp.current - 1.0).abs() < f32::EPSILON);
    }
}

#[test]
fn stack_one_spawns_two_debris_at_negative_fractional_victim() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 1);

    let victim = Vec2::new(-42.5, 17.25);
    write_cell_destroyed(&mut app, victim);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<(&FractureDebris, &Position2D)>();
    let offsets: Vec<Vec2> = query
        .iter(app.world())
        .map(|(_, pos)| pos.0 - victim)
        .collect();
    assert_eq!(offsets.len(), 2);
    for offset in &offsets {
        assert!(
            approx_eq_vec2(*offset, Vec2::new(70.0, 0.0), 1e-4)
                || approx_eq_vec2(*offset, Vec2::new(-70.0, 0.0), 1e-4),
            "unexpected offset {offset:?}"
        );
    }
}

// Behavior 11 — stack 2 spawns 3 debris (right, left, up).
#[test]
fn stack_two_spawns_three_debris_right_left_up() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 2);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<(&FractureDebris, &Position2D)>();
    let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
    assert_eq!(positions.len(), 3);

    let expected = [
        Vec2::new(70.0, 0.0),
        Vec2::new(-70.0, 0.0),
        Vec2::new(0.0, 24.0),
    ];
    for exp in &expected {
        assert!(
            positions.iter().any(|p| approx_eq_vec2(*p, *exp, 1e-4)),
            "expected offset {exp:?} missing"
        );
    }
    // DOWN offset must NOT be present at stack 2.
    let down = Vec2::new(0.0, -24.0);
    assert!(
        !positions.iter().any(|p| approx_eq_vec2(*p, down, 1e-4)),
        "down offset should not be present at stack 2"
    );
}

#[test]
fn three_base_splits_at_stack_one_also_fills_first_three_offsets() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    install_fracture_config(
        &mut app,
        FractureConfig {
            base_splits:      3,
            per_level_splits: 0,
        },
    );
    add_fracture_stacks(&mut app, 1);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<(&FractureDebris, &Position2D)>();
    let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
    assert_eq!(positions.len(), 3);

    let expected = [
        Vec2::new(70.0, 0.0),
        Vec2::new(-70.0, 0.0),
        Vec2::new(0.0, 24.0),
    ];
    for exp in &expected {
        assert!(
            positions.iter().any(|p| approx_eq_vec2(*p, *exp, 1e-4)),
            "expected offset {exp:?} missing"
        );
    }
}

// Behavior 12 — stack 3 spawns 4 debris at all orthogonal offsets.
#[test]
fn stack_three_spawns_four_debris() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    app.world_mut().insert_resource(FractureConfig {
        base_splits:      2,
        per_level_splits: 1,
    });
    for _ in 0..3 {
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Fracture);
    }

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&FractureDebris>();
    assert_eq!(query.iter(app.world()).count(), 4);
}

#[test]
fn stack_three_fills_all_four_orthogonal_offsets() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 3);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<(&FractureDebris, &Position2D)>();
    let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
    assert_eq!(positions.len(), 4);

    let expected = [
        Vec2::new(70.0, 0.0),
        Vec2::new(-70.0, 0.0),
        Vec2::new(0.0, 24.0),
        Vec2::new(0.0, -24.0),
    ];
    for exp in &expected {
        assert!(
            positions.iter().any(|p| approx_eq_vec2(*p, *exp, 1e-4)),
            "expected offset {exp:?} missing"
        );
    }
}

// Behavior 13 — stack 10 also spawns 4 (cap enforcement at spawn time).
#[test]
fn stack_ten_spawns_four_debris_all_offsets() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 10);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<(&FractureDebris, &Position2D)>();
    let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
    assert_eq!(positions.len(), 4);

    let expected = [
        Vec2::new(70.0, 0.0),
        Vec2::new(-70.0, 0.0),
        Vec2::new(0.0, 24.0),
        Vec2::new(0.0, -24.0),
    ];
    for exp in &expected {
        assert!(
            positions.iter().any(|p| approx_eq_vec2(*p, *exp, 1e-4)),
            "expected offset {exp:?} missing"
        );
    }
}

#[test]
fn stack_one_hundred_still_caps_at_four_debris() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 100);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&FractureDebris>();
    assert_eq!(query.iter(app.world()).count(), 4);
}
