use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::{
        components::PiercingRemaining,
        test_utils::{damage_stack, piercing_stack},
    },
    cells::{behaviors::locked::components::Locked, components::Cell},
    shared::death_pipeline::{hp::Hp, invulnerable::Invulnerable, killed_by::KilledBy},
};

#[test]
fn cell_collision_emits_damage_cell_with_base_damage() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = super::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    let cell_entity = spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);

    tick(&mut app);

    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "should emit exactly one DamageDealt<Cell> message on cell hit"
    );
    assert_eq!(
        msgs.0[0].target, cell_entity,
        "DamageDealt<Cell>.target should match the hit cell entity"
    );
    assert!(
        (msgs.0[0].amount - 10.0).abs() < f32::EPSILON,
        "DamageDealt<Cell>.amount should be base_damage (10.0), got {}",
        msgs.0[0].amount
    );
    assert_eq!(
        msgs.0[0].dealer,
        Some(bolt_entity),
        "DamageDealt<Cell>.dealer should be Some(bolt_entity)"
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

    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "bolt with no ActiveDamageBoosts should emit one DamageDealt<Cell>"
    );
    assert!(
        (msgs.0[0].amount - 10.0).abs() < f32::EPSILON,
        "no ActiveDamageBoosts should produce amount == 10.0 (identity), got {}",
        msgs.0[0].amount
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

    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "boosted bolt should emit one DamageDealt<Cell>"
    );
    assert!(
        (msgs.0[0].amount - 15.0).abs() < f32::EPSILON,
        "DamageDealt<Cell>.amount with ActiveDamageBoosts(1.5) should be 15.0, got {}",
        msgs.0[0].amount
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

    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "ActiveDamageBoosts(1.0) bolt should emit one DamageDealt<Cell>"
    );
    assert!(
        (msgs.0[0].amount - 10.0).abs() < f32::EPSILON,
        "ActiveDamageBoosts(1.0) should produce amount == 10.0, got {}",
        msgs.0[0].amount
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

    let bolt_a = spawn_bolt(&mut app, -100.0, start_y, 0.0, 400.0);
    let bolt_b = spawn_bolt(&mut app, 100.0, start_y, 0.0, 400.0);

    tick(&mut app);

    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert_eq!(
        msgs.0.len(),
        2,
        "two bolts hitting two cells should produce two DamageDealt<Cell> messages"
    );

    let msg_a = msgs.0.iter().find(|m| m.target == cell_a);
    let msg_b = msgs.0.iter().find(|m| m.target == cell_b);
    assert!(msg_a.is_some(), "DamageDealt<Cell> for cell A should exist");
    assert!(msg_b.is_some(), "DamageDealt<Cell> for cell B should exist");

    // Behavior 6: dealer field is attributed to the correct bolt (no cross-attribution).
    assert_eq!(
        msg_a.unwrap().dealer,
        Some(bolt_a),
        "DamageDealt<Cell> for cell A should have dealer = Some(bolt_a)"
    );
    assert_eq!(
        msg_b.unwrap().dealer,
        Some(bolt_b),
        "DamageDealt<Cell> for cell B should have dealer = Some(bolt_b)"
    );
}

#[test]
fn wall_hit_does_not_emit_damage_cell() {
    // A bolt hitting only a wall should produce zero DamageDealt<Cell> messages.
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = super::helpers::test_bolt_definition();

    spawn_right_wall(&mut app);

    // Right wall at (490,0) half_extents (90,300); inner face at x=400.
    let start_x = 490.0 - 90.0 - bc.radius - 5.0;
    spawn_bolt(&mut app, start_x, 0.0, 400.0, 0.1);

    tick(&mut app);

    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert!(
        msgs.0.is_empty(),
        "wall hit should NOT emit DamageDealt<Cell>, got {} messages",
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

    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert_eq!(
        msgs.0.len(),
        2,
        "piercing bolt should emit DamageDealt<Cell> for each pierced cell, got {}",
        msgs.0.len()
    );

    for msg in &msgs.0 {
        assert!(
            (msg.amount - 10.0).abs() < f32::EPSILON,
            "each DamageDealt<Cell>.amount should be 10.0, got {}",
            msg.amount
        );
    }

    let cells_hit: Vec<Entity> = msgs.0.iter().map(|m| m.target).collect();
    assert!(
        cells_hit.contains(&cell_a),
        "DamageDealt<Cell> for near cell should exist"
    );
    assert!(
        cells_hit.contains(&cell_b),
        "DamageDealt<Cell> for far cell should exist"
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

    let dmg_msgs = app.world().resource::<DamageDealtCellMessages>();
    assert_eq!(
        dmg_msgs.0.len(),
        1,
        "should emit exactly one DamageDealt<Cell> alongside BoltImpactCell"
    );
    assert_eq!(dmg_msgs.0[0].target, cell_entity);
}

/// Behavior 1: `bolt_cell_collision` uses `ActiveDamageBoosts.multiplier()` for damage.
///
/// Given: Bolt with `damage_stack(&[3.0])`, cell entity.
/// When: bolt collides with cell.
/// Then: `DamageDealt<Cell>` message has amount = 10.0 * 3.0 = 30.0.
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

    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "bolt with ActiveDamageBoosts should emit one DamageDealt<Cell>"
    );
    assert!(
        (msgs.0[0].amount - 30.0).abs() < f32::EPSILON,
        "DamageDealt<Cell>.amount should be 10.0 * 3.0 = 30.0 from ActiveDamageBoosts, got {}",
        msgs.0[0].amount
    );
}

/// Behavior 2: `bolt_cell_collision` uses default multiplier when no `ActiveDamageBoosts`.
///
/// Given: Bolt with NO `ActiveDamageBoosts`.
/// When: bolt collides with cell.
/// Then: `DamageDealt<Cell>` message has amount = 10.0 (default 1.0 multiplier).
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

    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "bolt with no ActiveDamageBoosts should emit one DamageDealt<Cell>"
    );
    assert!(
        (msgs.0[0].amount - 10.0).abs() < f32::EPSILON,
        "DamageDealt<Cell>.amount should be 10.0 (no ActiveDamageBoosts = default multiplier), got {}",
        msgs.0[0].amount
    );
}

// ── Behavior 6: dealer attribution ──────────────────────────────────────────

/// Behavior 6: `DamageDealt<Cell>.dealer` is set to the source bolt entity.
#[test]
fn cell_collision_sets_dealer_to_source_bolt_entity() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = super::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);

    tick(&mut app);

    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "exactly one DamageDealt<Cell> should be emitted for the hit cell"
    );
    assert_eq!(
        msgs.0[0].dealer,
        Some(bolt_entity),
        "DamageDealt<Cell>.dealer should be Some(bolt_entity)"
    );
}

// ── Behavior 8.5: producer is type-agnostic about Locked/Invulnerable ──────

/// Behavior 8.5: `bolt_cell_collision` emits `DamageDealt<Cell>` even for a
/// `Locked` + `Invulnerable` cell. The producer does NOT filter lock state —
/// immunity is enforced downstream in `apply_damage::<Cell>`.
#[test]
fn cell_collision_emits_damage_cell_for_locked_invulnerable_cell() {
    use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
    use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D};

    let mut app = test_app_with_damage_and_wall_messages();
    let bc = super::helpers::test_bolt_definition();
    let cc = crate::cells::resources::CellConfig::default();

    let (cw, ch) = default_cell_dims();
    let half_extents = Vec2::new(cw.half_width(), ch.half_height());
    let cell_y = 100.0;
    let pos = Vec2::new(0.0, cell_y);
    let cell_entity = app
        .world_mut()
        .spawn((
            Cell,
            cw,
            ch,
            Hp::new(10.0),
            KilledBy::default(),
            Locked,
            Invulnerable,
            Aabb2D::new(Vec2::ZERO, half_extents),
            CollisionLayers::new(crate::shared::CELL_LAYER, crate::shared::BOLT_LAYER),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            crate::shared::GameDrawLayer::Cell,
        ))
        .id();

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);

    tick(&mut app);

    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "bolt hitting Locked/Invulnerable cell should STILL emit DamageDealt<Cell> (producer is filter-free)"
    );
    assert_eq!(msgs.0[0].target, cell_entity);
    assert!(
        (msgs.0[0].amount - 10.0).abs() < f32::EPSILON,
        "amount should be 10.0 even against a locked cell"
    );
}
