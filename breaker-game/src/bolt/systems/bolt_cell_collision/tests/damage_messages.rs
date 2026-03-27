use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::helpers::*;
use crate::{
    bolt::components::Bolt,
    chips::components::{DamageBoost, Piercing, PiercingRemaining},
};

#[test]
fn cell_collision_emits_damage_cell_with_base_damage() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    let cell_entity = spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let _bolt_entity = app
        .world_mut()
        .spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, start_y)),
        ))
        .id();

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "should emit exactly one DamageCell message on cell hit"
    );
    assert_eq!(
        msgs.0[0].cell, cell_entity,
        "DamageCell.cell should match the hit cell entity"
    );
    assert!(
        (msgs.0[0].damage - 10.0).abs() < f32::EPSILON,
        "DamageCell.damage should be BASE_BOLT_DAMAGE (10.0), got {}",
        msgs.0[0].damage
    );
}

#[test]
fn cell_collision_emits_damage_cell_with_zero_damage_boost() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    app.world_mut().spawn((
        Bolt,
        bolt_param_bundle(),
        Velocity2D(Vec2::new(0.0, 400.0)),
        DamageBoost(0.0),
        Position2D(Vec2::new(0.0, start_y)),
    ));

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "DamageBoost(0.0) bolt should emit one DamageCell"
    );
    assert!(
        (msgs.0[0].damage - 10.0).abs() < f32::EPSILON,
        "DamageBoost(0.0) should produce damage == 10.0, got {}",
        msgs.0[0].damage
    );
}

#[test]
fn cell_collision_emits_damage_cell_with_boosted_damage() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let _bolt_entity = app
        .world_mut()
        .spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            DamageBoost(0.5),
            Position2D(Vec2::new(0.0, start_y)),
        ))
        .id();

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert_eq!(msgs.0.len(), 1, "boosted bolt should emit one DamageCell");
    assert!(
        (msgs.0[0].damage - 15.0).abs() < f32::EPSILON,
        "DamageCell.damage with DamageBoost(0.5) should be 15.0, got {}",
        msgs.0[0].damage
    );
}

#[test]
fn two_bolts_emit_damage_cell_with_correct_source_bolt() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_a = spawn_cell(&mut app, -100.0, 100.0);
    let cell_b = spawn_cell(&mut app, 100.0, 100.0);

    let start_y = 100.0 - cc.height / 2.0 - bc.radius - 2.0;

    let _bolt_a = app
        .world_mut()
        .spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(-100.0, start_y)),
        ))
        .id();
    let _bolt_b = app
        .world_mut()
        .spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(100.0, start_y)),
        ))
        .id();

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert_eq!(
        msgs.0.len(),
        2,
        "two bolts hitting two cells should produce two DamageCell messages"
    );

    let msg_a = msgs.0.iter().find(|m| m.cell == cell_a);
    let msg_b = msgs.0.iter().find(|m| m.cell == cell_b);
    assert!(msg_a.is_some(), "DamageCell for cell A should exist");
    assert!(msg_b.is_some(), "DamageCell for cell B should exist");
}

#[test]
fn wall_hit_does_not_emit_damage_cell() {
    // A bolt hitting only a wall should produce zero DamageCell messages.
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = crate::bolt::resources::BoltConfig::default();

    spawn_wall(&mut app, 200.0, 0.0, 50.0, 300.0);

    let start_x = 200.0 - 50.0 - bc.radius - 5.0;
    app.world_mut().spawn((
        Bolt,
        bolt_param_bundle(),
        Velocity2D(Vec2::new(400.0, 0.1)),
        Position2D(Vec2::new(start_x, 0.0)),
    ));

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert!(
        msgs.0.is_empty(),
        "wall hit should NOT emit DamageCell, got {} messages",
        msgs.0.len()
    );
}

#[test]
fn piercing_bolt_emits_damage_cell_for_each_pierced_cell() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = crate::bolt::resources::BoltConfig::default();

    let near_cell_y = 60.0;
    let far_cell_y = 90.0;
    let cell_a = spawn_cell_with_health(&mut app, 0.0, near_cell_y, 10.0);
    let cell_b = spawn_cell_with_health(&mut app, 0.0, far_cell_y, 10.0);

    let start_y = near_cell_y - bc.radius - 25.0;
    let _bolt_entity = app
        .world_mut()
        .spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 10000.0)),
            Piercing(2),
            PiercingRemaining(2),
            Position2D(Vec2::new(0.0, start_y)),
        ))
        .id();

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert_eq!(
        msgs.0.len(),
        2,
        "piercing bolt should emit DamageCell for each pierced cell, got {}",
        msgs.0.len()
    );

    for msg in &msgs.0 {
        assert!(
            (msg.damage - 10.0).abs() < f32::EPSILON,
            "each DamageCell.damage should be 10.0, got {}",
            msg.damage
        );
    }

    let cells_hit: Vec<Entity> = msgs.0.iter().map(|m| m.cell).collect();
    assert!(
        cells_hit.contains(&cell_a),
        "DamageCell for near cell should exist"
    );
    assert!(
        cells_hit.contains(&cell_b),
        "DamageCell for far cell should exist"
    );
}

#[test]
fn cell_hit_emits_both_bolt_hit_cell_and_damage_cell() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    let cell_entity = spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = app
        .world_mut()
        .spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(0.0, start_y)),
        ))
        .id();

    tick(&mut app);

    let hit_msgs = app.world().resource::<FullHitMessages>();
    assert_eq!(hit_msgs.0.len(), 1, "should emit exactly one BoltHitCell");
    assert_eq!(hit_msgs.0[0].cell, cell_entity);
    assert_eq!(hit_msgs.0[0].bolt, bolt_entity);

    let dmg_msgs = app.world().resource::<DamageCellMessages>();
    assert_eq!(
        dmg_msgs.0.len(),
        1,
        "should emit exactly one DamageCell alongside BoltHitCell"
    );
    assert_eq!(dmg_msgs.0[0].cell, cell_entity);
}
