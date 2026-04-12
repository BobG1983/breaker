use bevy::prelude::*;

use super::helpers::*;
use crate::bolt::{
    components::PiercingRemaining,
    test_utils::{damage_stack, piercing_stack},
};

#[test]
fn cell_collision_emits_damage_cell_with_base_damage() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = super::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    let cell_entity = spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);

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
        "DamageCell.damage should be base_damage (10.0), got {}",
        msgs.0[0].damage
    );
}

/// Spec behavior 1: Base damage with no `ActiveDamageBoosts` component.
/// `ActiveDamageBoosts` absent => identity (1.0), damage = 10.0 * 1.0 = 10.0.
#[test]
fn cell_collision_emits_damage_cell_with_no_effective_damage_multiplier() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = super::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    // No ActiveDamageBoosts component
    spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "bolt with no ActiveDamageBoosts should emit one DamageCell"
    );
    assert!(
        (msgs.0[0].damage - 10.0).abs() < f32::EPSILON,
        "no ActiveDamageBoosts should produce damage == 10.0 (identity), got {}",
        msgs.0[0].damage
    );
}

/// Spec behavior 2: Boosted damage with `ActiveDamageBoosts(1.5)`.
/// Formula: 10.0 * 1.5 = 15.0.
#[test]
fn cell_collision_emits_damage_cell_with_boosted_damage() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = super::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(damage_stack(&[1.5]));

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert_eq!(msgs.0.len(), 1, "boosted bolt should emit one DamageCell");
    assert!(
        (msgs.0[0].damage - 15.0).abs() < f32::EPSILON,
        "DamageCell.damage with ActiveDamageBoosts(1.5) should be 15.0, got {}",
        msgs.0[0].damage
    );
}

/// Spec behavior 2 edge case: `ActiveDamageBoosts(1.0)` is identity.
#[test]
fn cell_collision_emits_damage_cell_with_identity_effective_damage_multiplier() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = super::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(damage_stack(&[1.0]));

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "ActiveDamageBoosts(1.0) bolt should emit one DamageCell"
    );
    assert!(
        (msgs.0[0].damage - 10.0).abs() < f32::EPSILON,
        "ActiveDamageBoosts(1.0) should produce damage == 10.0, got {}",
        msgs.0[0].damage
    );
}

#[test]
fn two_bolts_emit_damage_cell_with_correct_source_bolt() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = super::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_a = spawn_cell(&mut app, -100.0, 100.0);
    let cell_b = spawn_cell(&mut app, 100.0, 100.0);

    let start_y = 100.0 - cc.height / 2.0 - bc.radius - 2.0;

    spawn_bolt(&mut app, -100.0, start_y, 0.0, 400.0);
    spawn_bolt(&mut app, 100.0, start_y, 0.0, 400.0);

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
    let bc = super::helpers::test_bolt_definition();

    spawn_right_wall(&mut app);

    // Right wall at (490,0) half_extents (90,300); inner face at x=400.
    let start_x = 490.0 - 90.0 - bc.radius - 5.0;
    spawn_bolt(&mut app, start_x, 0.0, 400.0, 0.1);

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
    let bc = super::helpers::test_bolt_definition();

    let near_cell_y = 60.0;
    let far_cell_y = 90.0;
    let cell_a = spawn_cell_with_health(&mut app, 0.0, near_cell_y, 10.0);
    let cell_b = spawn_cell_with_health(&mut app, 0.0, far_cell_y, 10.0);

    let start_y = near_cell_y - bc.radius - 25.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 10000.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert((piercing_stack(&[2]), PiercingRemaining(2)));

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
    let bc = super::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    let cell_entity = spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);

    tick(&mut app);

    let hit_msgs = app.world().resource::<FullHitMessages>();
    assert_eq!(
        hit_msgs.0.len(),
        1,
        "should emit exactly one BoltImpactCell"
    );
    assert_eq!(hit_msgs.0[0].cell, cell_entity);
    assert_eq!(hit_msgs.0[0].bolt, bolt_entity);

    let dmg_msgs = app.world().resource::<DamageCellMessages>();
    assert_eq!(
        dmg_msgs.0.len(),
        1,
        "should emit exactly one DamageCell alongside BoltImpactCell"
    );
    assert_eq!(dmg_msgs.0[0].cell, cell_entity);
}

/// Behavior 1: `bolt_cell_collision` uses `ActiveDamageBoosts.multiplier()` for damage.
///
/// Given: Bolt with `damage_stack(&[3.0])`, cell entity.
/// When: bolt collides with cell.
/// Then: `DamageCell` message has damage = 10.0 * 3.0 = 30.0.
#[test]
fn cell_collision_uses_active_damage_boosts_multiplier() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = super::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(damage_stack(&[3.0]));

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "bolt with ActiveDamageBoosts should emit one DamageCell"
    );
    assert!(
        (msgs.0[0].damage - 30.0).abs() < f32::EPSILON,
        "DamageCell.damage should be 10.0 * 3.0 = 30.0 from ActiveDamageBoosts, got {}",
        msgs.0[0].damage
    );
}

/// Behavior 2: `bolt_cell_collision` uses default multiplier when no `ActiveDamageBoosts`.
///
/// Given: Bolt with NO `ActiveDamageBoosts`.
/// When: bolt collides with cell.
/// Then: `DamageCell` message has damage = 10.0 (default 1.0 multiplier).
#[test]
fn cell_collision_ignores_stale_effective_damage_multiplier() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = super::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    // No ActiveDamageBoosts — verifies default multiplier of 1.0
    spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "bolt with no ActiveDamageBoosts should emit one DamageCell"
    );
    assert!(
        (msgs.0[0].damage - 10.0).abs() < f32::EPSILON,
        "DamageCell.damage should be 10.0 (no ActiveDamageBoosts = default multiplier), got {}",
        msgs.0[0].damage
    );
}
