use std::{collections::HashSet, time::Duration};

use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::{
    cells::components::Cell,
    hazard::{definition::HazardKind, resources::ActiveHazards},
    prelude::*,
};

// Behavior 19 — no config → no debris; reader cleared.
#[test]
fn no_debris_when_config_absent() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Fracture);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&FractureDebris>();
    assert_eq!(query.iter(app.world()).count(), 0);
}

#[test]
fn no_config_reader_drains_and_second_tick_with_config_does_not_retroactively_spawn() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    add_fracture_stacks(&mut app, 1);

    // First tick: no config → reader.clear() drains the message.
    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    // Install config and tick again WITHOUT writing a new message.
    install_fracture_config(&mut app, canonical_config());
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&FractureDebris>();
    assert_eq!(query.iter(app.world()).count(), 0);
}

// Behavior 20 — zero stacks → no debris; reader cleared.
#[test]
fn no_debris_at_zero_stacks() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    app.world_mut().insert_resource(FractureConfig {
        base_splits:      2,
        per_level_splits: 1,
    });

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&FractureDebris>();
    assert_eq!(query.iter(app.world()).count(), 0);
}

#[test]
fn zero_stacks_reader_drains_and_stack_added_mid_way_does_not_retroactively_spawn() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    install_fracture_config(&mut app, canonical_config());

    // First tick: zero stacks → reader.clear() drains the message.
    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    // Add a stack; tick again without a new message.
    add_fracture_stacks(&mut app, 1);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&FractureDebris>();
    assert_eq!(query.iter(app.world()).count(), 0);
}

// Behavior 21 — multiple deaths each spawn independently.
#[test]
fn multiple_deaths_each_spawn_debris_independently() {
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
    write_cell_destroyed(&mut app, Vec2::new(500.0, 300.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&FractureDebris>();
    assert_eq!(query.iter(app.world()).count(), 4);
}

#[test]
fn three_deaths_at_stack_two_spawn_nine_debris_with_per_victim_offsets() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 2);

    let victims = [
        Vec2::new(10.0, 20.0),
        Vec2::new(100.0, 200.0),
        Vec2::new(-50.0, 0.0),
    ];
    for v in &victims {
        write_cell_destroyed(&mut app, *v);
    }
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<(&FractureDebris, &Position2D)>();
    let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
    assert_eq!(positions.len(), 9);

    let expected_offsets = [
        Vec2::new(70.0, 0.0),
        Vec2::new(-70.0, 0.0),
        Vec2::new(0.0, 24.0),
    ];
    for victim in &victims {
        for offset in &expected_offsets {
            let expected_pos = *victim + *offset;
            assert!(
                positions
                    .iter()
                    .any(|p| approx_eq_vec2(*p, expected_pos, 1e-4)),
                "missing debris at {expected_pos:?} (victim {victim:?} + offset {offset:?})"
            );
        }
    }
}

// Behavior 22 — recursive fracture pin: destroying a FractureDebris cell also spawns.
#[test]
fn destroying_a_fracture_debris_cell_also_spawns_new_debris() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 1);

    // Seed an existing debris-marked entity as if it were already in the world.
    let victim = Vec2::new(200.0, 200.0);
    app.world_mut()
        .spawn((Cell, FractureDebris, Position2D(victim)));

    write_cell_destroyed(&mut app, victim);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<(&FractureDebris, &Position2D)>();
    let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
    assert_eq!(positions.len(), 3);

    // Two NEW debris at the death position ± (70, 0).
    let expected_right = victim + Vec2::new(70.0, 0.0);
    let expected_left = victim + Vec2::new(-70.0, 0.0);
    assert!(
        positions
            .iter()
            .any(|p| approx_eq_vec2(*p, expected_right, 1e-4)),
        "missing right-offset debris"
    );
    assert!(
        positions
            .iter()
            .any(|p| approx_eq_vec2(*p, expected_left, 1e-4)),
        "missing left-offset debris"
    );
    // Seeded debris still present at victim position.
    assert!(
        positions.iter().any(|p| approx_eq_vec2(*p, victim, 1e-4)),
        "original debris should still be present"
    );
}

#[test]
fn recursive_fracture_scales_with_stack_count() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 3);

    let victim = Vec2::new(200.0, 200.0);
    app.world_mut()
        .spawn((Cell, FractureDebris, Position2D(victim)));

    write_cell_destroyed(&mut app, victim);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&FractureDebris>();
    assert_eq!(query.iter(app.world()).count(), 5);
}

// Behavior 23 — duplicate victim positions produce independent debris entities.
#[test]
fn duplicate_victim_positions_produce_independent_debris_entities() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 1);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    write_cell_destroyed(&mut app, Vec2::ZERO);
    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&FractureDebris>();
    assert_eq!(query.iter(app.world()).count(), 6);
}

#[test]
fn duplicate_victim_positions_produce_six_distinct_entities() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 1);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    write_cell_destroyed(&mut app, Vec2::ZERO);
    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app
        .world_mut()
        .query_filtered::<Entity, With<FractureDebris>>();
    let entities: HashSet<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 6);
}

// ── register integration ────────────────────────────────────────────

// Behavior 24 — full register path spawns debris under active gate.
#[test]
fn register_with_config_and_stack_spawns_debris_on_death() {
    let mut app = test_app_playing();
    register(&mut app);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 1);

    write_cell_destroyed(&mut app, Vec2::new(50.0, 50.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<(&FractureDebris, &Position2D)>();
    let positions: Vec<Vec2> = query.iter(app.world()).map(|(_, p)| p.0).collect();
    assert_eq!(positions.len(), 2);

    let expected = [Vec2::new(120.0, 50.0), Vec2::new(-20.0, 50.0)];
    for exp in &expected {
        assert!(
            positions.iter().any(|p| approx_eq_vec2(*p, *exp, 1e-4)),
            "missing expected position {exp:?}"
        );
    }
}

#[test]
fn register_second_tick_without_message_spawns_no_new_debris() {
    let mut app = test_app_playing();
    register(&mut app);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 1);

    write_cell_destroyed(&mut app, Vec2::new(50.0, 50.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
    // Second tick, no new message.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<&FractureDebris>();
    assert_eq!(query.iter(app.world()).count(), 2);
}
