use bevy::prelude::*;

use super::super::helpers::*;
use crate::{effect_v3::effects::tether_beam::components::*, shared::test_utils::tick};

// ── Group A — tick_tether_beam geometry ────────────────────────────────

#[test]
fn cell_on_beam_line_is_damaged() {
    let mut app = tether_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
    let cell_entity = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.0));

    let beam_entity = app
        .world_mut()
        .spawn((
            TetherBeamSource { bolt_a, bolt_b },
            TetherBeamDamage(12.5),
            TetherBeamWidth(10.0),
        ))
        .id();

    tick(&mut app);

    let msgs = damage_msgs(&app);
    assert_eq!(
        msgs.len(),
        1,
        "expected exactly 1 DamageDealt<Cell> message"
    );
    assert_eq!(msgs[0].target, cell_entity);
    assert!((msgs[0].amount - 12.5).abs() < 1e-6);
    assert_eq!(msgs[0].dealer, Some(beam_entity));
}

#[test]
fn cell_beyond_bolt_b_is_not_damaged() {
    let mut app = tether_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
    let _cell = spawn_alive_cell(&mut app, Vec2::new(150.0, 0.0));

    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(12.5),
        TetherBeamWidth(10.0),
    ));

    tick(&mut app);

    assert_eq!(damage_msgs(&app).len(), 0);
}

#[test]
fn cell_behind_bolt_a_is_not_damaged() {
    let mut app = tether_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
    let _cell = spawn_alive_cell(&mut app, Vec2::new(-10.0, 0.0));

    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(12.5),
        TetherBeamWidth(10.0),
    ));

    tick(&mut app);

    assert_eq!(damage_msgs(&app).len(), 0);
}

#[test]
fn cell_at_bolt_a_position_is_damaged() {
    let mut app = tether_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
    let cell_entity = spawn_alive_cell(&mut app, Vec2::new(0.0, 0.0));

    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(12.5),
        TetherBeamWidth(10.0),
    ));

    tick(&mut app);

    let msgs = damage_msgs(&app);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].target, cell_entity);
}

#[test]
fn cell_at_beam_len_boundary_is_damaged() {
    let mut app = tether_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
    let cell_entity = spawn_alive_cell(&mut app, Vec2::new(100.0, 0.0));

    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(12.5),
        TetherBeamWidth(10.0),
    ));

    tick(&mut app);

    let msgs = damage_msgs(&app);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].target, cell_entity);
}

#[test]
fn cell_within_half_width_is_damaged_both_sides() {
    let mut app = tether_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
    let positive_cell = spawn_alive_cell(&mut app, Vec2::new(50.0, 9.0));
    let negative_cell = spawn_alive_cell(&mut app, Vec2::new(50.0, -9.0));

    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(12.5),
        TetherBeamWidth(10.0),
    ));

    tick(&mut app);

    let msgs = damage_msgs(&app);
    assert_eq!(msgs.len(), 2, "both symmetric cells must be damaged");
    let targets: Vec<Entity> = msgs.iter().map(|m| m.target).collect();
    assert!(targets.contains(&positive_cell));
    assert!(targets.contains(&negative_cell));
}

#[test]
fn cell_at_half_width_boundary_is_damaged_both_sides() {
    let mut app = tether_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
    let positive_cell = spawn_alive_cell(&mut app, Vec2::new(50.0, 10.0));
    let negative_cell = spawn_alive_cell(&mut app, Vec2::new(50.0, -10.0));

    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(12.5),
        TetherBeamWidth(10.0),
    ));

    tick(&mut app);

    let msgs = damage_msgs(&app);
    assert_eq!(msgs.len(), 2);
    let targets: Vec<Entity> = msgs.iter().map(|m| m.target).collect();
    assert!(targets.contains(&positive_cell));
    assert!(targets.contains(&negative_cell));
}

#[test]
fn cell_outside_half_width_is_not_damaged_both_sides() {
    let mut app = tether_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
    let _positive_cell = spawn_alive_cell(&mut app, Vec2::new(50.0, 11.0));
    let _negative_cell = spawn_alive_cell(&mut app, Vec2::new(50.0, -11.0));

    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(12.5),
        TetherBeamWidth(10.0),
    ));

    tick(&mut app);

    assert_eq!(damage_msgs(&app).len(), 0);
}

#[test]
fn dead_cells_are_never_damaged() {
    let mut app = tether_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
    let _dead_cell = spawn_dead_cell(&mut app, Vec2::new(50.0, 0.0));
    let alive_cell = spawn_alive_cell(&mut app, Vec2::new(50.0, 5.0));

    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(12.5),
        TetherBeamWidth(10.0),
    ));

    tick(&mut app);

    let msgs = damage_msgs(&app);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].target, alive_cell);
}

#[test]
fn despawned_bolt_a_produces_no_damage_and_no_panic() {
    let mut app = tether_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
    let _cell = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.0));

    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(12.5),
        TetherBeamWidth(10.0),
    ));

    app.world_mut().despawn(bolt_a);

    tick(&mut app);

    assert_eq!(damage_msgs(&app).len(), 0);
    let beam_count = app
        .world_mut()
        .query::<&TetherBeamSource>()
        .iter(app.world())
        .count();
    assert_eq!(
        beam_count, 1,
        "tick must not despawn the beam when an endpoint is missing"
    );
}

#[test]
fn despawned_bolt_b_produces_no_damage_and_no_panic() {
    let mut app = tether_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(100.0, 0.0));
    let _cell = spawn_alive_cell(&mut app, Vec2::new(50.0, 0.0));

    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(12.5),
        TetherBeamWidth(10.0),
    ));

    app.world_mut().despawn(bolt_b);

    tick(&mut app);

    assert_eq!(damage_msgs(&app).len(), 0);
    let beam_count = app
        .world_mut()
        .query::<&TetherBeamSource>()
        .iter(app.world())
        .count();
    assert_eq!(beam_count, 1);
}

#[test]
fn zero_length_beam_produces_no_damage() {
    let mut app = tether_test_app();
    let bolt_a = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let bolt_b = spawn_endpoint(&mut app, Vec2::new(0.0, 0.0));
    let _cell_at_origin = spawn_alive_cell(&mut app, Vec2::new(0.0, 0.0));
    let _cell_nearby = spawn_alive_cell(&mut app, Vec2::new(5.0, 0.0));

    app.world_mut().spawn((
        TetherBeamSource { bolt_a, bolt_b },
        TetherBeamDamage(12.5),
        TetherBeamWidth(10.0),
    ));

    tick(&mut app);

    assert_eq!(damage_msgs(&app).len(), 0);
}
