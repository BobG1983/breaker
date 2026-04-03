//! Migration tests for `bolt_cell_collision` — verifying the system reads
//! `BoltBaseDamage` from the entity component.

use bevy::{ecs::world::CommandQueue, prelude::*};
use rantzsoft_spatial2d::components::Velocity2D;

use super::helpers::*;
use crate::{
    bolt::{
        components::{Bolt, BoltBaseDamage, PiercingRemaining},
        definition::BoltDefinition,
        resources::DEFAULT_BOLT_BASE_DAMAGE,
    },
    effect::effects::{damage_boost::ActiveDamageBoosts, piercing::ActivePiercings},
};

fn make_default_bolt_definition() -> BoltDefinition {
    BoltDefinition {
        name: "Bolt".to_string(),
        base_speed: 720.0,
        min_speed: 360.0,
        max_speed: 1440.0,
        radius: 14.0,
        base_damage: 10.0,
        effects: vec![],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    }
}

/// Spawns a bolt via `.definition()` at the given position with velocity.
fn spawn_bolt_from_definition(app: &mut App, x: f32, y: f32, vx: f32, vy: f32) -> Entity {
    let def = make_default_bolt_definition();
    let world = app.world_mut();
    let mut queue = CommandQueue::default();
    let entity = {
        let mut commands = Commands::new(&mut queue, world);
        Bolt::builder()
            .at_position(Vec2::new(x, y))
            .definition(&def)
            .with_velocity(Velocity2D(Vec2::new(vx, vy)))
            .primary()
            .headless()
            .spawn(&mut commands)
    };
    queue.apply(world);
    entity
}

/// Spawns a bolt via `.definition()` with a custom `base_damage`.
fn spawn_bolt_with_damage(app: &mut App, x: f32, y: f32, vx: f32, vy: f32, damage: f32) -> Entity {
    let def = BoltDefinition {
        base_damage: damage,
        ..make_default_bolt_definition()
    };
    let world = app.world_mut();
    let mut queue = CommandQueue::default();
    let entity = {
        let mut commands = Commands::new(&mut queue, world);
        Bolt::builder()
            .at_position(Vec2::new(x, y))
            .definition(&def)
            .with_velocity(Velocity2D(Vec2::new(vx, vy)))
            .primary()
            .headless()
            .spawn(&mut commands)
    };
    queue.apply(world);
    entity
}

// ── Behavior 20: bolt_cell_collision queries BoltBaseDamage from entity ──

#[test]
fn collision_uses_bolt_base_damage_from_entity() {
    let mut app = test_app_with_damage_and_wall_messages();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - 14.0 - 2.0; // radius 14.0 from definition
    spawn_bolt_from_definition(&mut app, 0.0, start_y, 0.0, 400.0);

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert_eq!(msgs.0.len(), 1, "should emit one DamageCell");
    assert!(
        (msgs.0[0].damage - 10.0).abs() < f32::EPSILON,
        "DamageCell.damage should be 10.0 (from BoltBaseDamage), got {}",
        msgs.0[0].damage
    );
}

#[test]
fn collision_with_zero_base_damage() {
    // Edge case: BoltBaseDamage(0.0) -- damage is 0.0
    let mut app = test_app_with_damage_and_wall_messages();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - 14.0 - 2.0;
    let bolt_entity = spawn_bolt_from_definition(&mut app, 0.0, start_y, 0.0, 400.0);
    // Override base damage to 0.0
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(BoltBaseDamage(0.0));

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert_eq!(msgs.0.len(), 1, "should emit one DamageCell");
    assert!(
        (msgs.0[0].damage).abs() < f32::EPSILON,
        "DamageCell.damage should be 0.0, got {}",
        msgs.0[0].damage
    );
}

// ── Behavior 21: bolt_cell_collision uses BoltBaseDamage with ActiveDamageBoosts ──

#[test]
fn collision_uses_bolt_base_damage_with_damage_boost() {
    let mut app = test_app_with_damage_and_wall_messages();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - 14.0 - 2.0;
    let bolt_entity = spawn_bolt_from_definition(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(ActiveDamageBoosts(vec![1.5]));

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert_eq!(msgs.0.len(), 1, "should emit one DamageCell");
    assert!(
        (msgs.0[0].damage - 15.0).abs() < f32::EPSILON,
        "DamageCell.damage should be 10.0 * 1.5 = 15.0, got {}",
        msgs.0[0].damage
    );
}

#[test]
fn collision_high_base_damage_with_boost() {
    // Edge case: BoltBaseDamage(25.0) with ActiveDamageBoosts(vec![2.0]) = 50.0
    let mut app = test_app_with_damage_and_wall_messages();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - 14.0 - 2.0;
    let bolt_entity = spawn_bolt_with_damage(&mut app, 0.0, start_y, 0.0, 400.0, 25.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(ActiveDamageBoosts(vec![2.0]));

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert_eq!(msgs.0.len(), 1, "should emit one DamageCell");
    assert!(
        (msgs.0[0].damage - 50.0).abs() < f32::EPSILON,
        "DamageCell.damage should be 25.0 * 2.0 = 50.0, got {}",
        msgs.0[0].damage
    );
}

// ── Behavior 22: bolt_cell_collision uses BoltBaseDamage for piercing lookahead ──

#[test]
fn collision_piercing_uses_bolt_base_damage_for_lookahead() {
    let mut app = test_app_with_damage_and_wall_messages();

    let cell_y = 100.0;
    // Cell with HP 10.0 -- BoltBaseDamage 10.0 should be enough to pierce
    spawn_cell_with_health(&mut app, 0.0, cell_y, 10.0);

    let start_y = cell_y - 14.0 - 25.0;
    let bolt_entity = spawn_bolt_from_definition(&mut app, 0.0, start_y, 0.0, 10000.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert((ActivePiercings(vec![1]), PiercingRemaining(1)));

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert!(!msgs.0.is_empty(), "should emit at least one DamageCell");
    assert!(
        (msgs.0[0].damage - 10.0).abs() < f32::EPSILON,
        "DamageCell.damage should be 10.0, got {}",
        msgs.0[0].damage
    );

    // Verify pierce happened (PiercingRemaining decremented)
    let pr = app
        .world()
        .get::<PiercingRemaining>(bolt_entity)
        .expect("bolt should have PiercingRemaining");
    assert_eq!(
        pr.0, 0,
        "PiercingRemaining should be decremented to 0 after pierce, got {}",
        pr.0
    );
}

#[test]
fn collision_piercing_low_damage_does_not_pierce() {
    // Edge case: BoltBaseDamage(5.0) with cell HP 10.0 -- 5.0 < 10.0, no pierce
    let mut app = test_app_with_damage_and_wall_messages();

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 10.0);

    let start_y = cell_y - 14.0 - 25.0;
    let bolt_entity = spawn_bolt_with_damage(&mut app, 0.0, start_y, 0.0, 10000.0, 5.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert((ActivePiercings(vec![1]), PiercingRemaining(1)));

    tick(&mut app);

    // Bolt should reflect (not pierce) because 5.0 < 10.0 cell HP
    let pr = app
        .world()
        .get::<PiercingRemaining>(bolt_entity)
        .expect("bolt should have PiercingRemaining");
    assert_eq!(
        pr.0, 1,
        "PiercingRemaining should NOT be decremented (damage 5.0 < HP 10.0), got {}",
        pr.0
    );
}

// ── Behavior 23: bolt_cell_collision reads BoltBaseDamage from entity ──

#[test]
fn collision_uses_entity_damage_not_constant() {
    // Given: BoltBaseDamage(25.0) on entity (different from default 10.0)
    // Then: damage is 25.0, NOT 10.0
    let mut app = test_app_with_damage_and_wall_messages();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - 14.0 - 2.0;
    let _bolt_entity = spawn_bolt_with_damage(&mut app, 0.0, start_y, 0.0, 400.0, 25.0);

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert_eq!(msgs.0.len(), 1, "should emit one DamageCell");
    assert!(
        (msgs.0[0].damage - 25.0).abs() < f32::EPSILON,
        "DamageCell.damage should be 25.0 (from BoltBaseDamage), NOT {} (default). Got {}",
        DEFAULT_BOLT_BASE_DAMAGE,
        msgs.0[0].damage
    );
}

// ── Behavior 24: different BoltBaseDamage on two bolts ──

#[test]
fn collision_two_bolts_different_base_damage() {
    let mut app = test_app_with_damage_and_wall_messages();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_a = spawn_cell(&mut app, -100.0, 100.0);
    let cell_b = spawn_cell(&mut app, 100.0, 100.0);

    let start_y = 100.0 - cc.height / 2.0 - 14.0 - 2.0;

    // Bolt A: damage 10.0
    spawn_bolt_from_definition(&mut app, -100.0, start_y, 0.0, 400.0);
    // Bolt B: damage 25.0
    spawn_bolt_with_damage(&mut app, 100.0, start_y, 0.0, 400.0, 25.0);

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert_eq!(
        msgs.0.len(),
        2,
        "should emit two DamageCell messages, got {}",
        msgs.0.len()
    );

    let msg_a = msgs.0.iter().find(|m| m.cell == cell_a);
    let msg_b = msgs.0.iter().find(|m| m.cell == cell_b);
    assert!(msg_a.is_some(), "DamageCell for cell A should exist");
    assert!(msg_b.is_some(), "DamageCell for cell B should exist");

    assert!(
        (msg_a.unwrap().damage - 10.0).abs() < f32::EPSILON,
        "cell A damage should be 10.0 (from bolt A), got {}",
        msg_a.unwrap().damage
    );
    assert!(
        (msg_b.unwrap().damage - 25.0).abs() < f32::EPSILON,
        "cell B damage should be 25.0 (from bolt B), got {}",
        msg_b.unwrap().damage
    );
}

#[test]
fn collision_two_bolts_different_damage_with_boosts() {
    // Edge case: Both bolts with ActiveDamageBoosts(vec![2.0])
    let mut app = test_app_with_damage_and_wall_messages();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_a = spawn_cell(&mut app, -100.0, 100.0);
    let cell_b = spawn_cell(&mut app, 100.0, 100.0);

    let start_y = 100.0 - cc.height / 2.0 - 14.0 - 2.0;

    let bolt_a = spawn_bolt_from_definition(&mut app, -100.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_a)
        .insert(ActiveDamageBoosts(vec![2.0]));

    let bolt_b = spawn_bolt_with_damage(&mut app, 100.0, start_y, 0.0, 400.0, 25.0);
    app.world_mut()
        .entity_mut(bolt_b)
        .insert(ActiveDamageBoosts(vec![2.0]));

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert_eq!(msgs.0.len(), 2, "should emit two DamageCell messages");

    let msg_a = msgs.0.iter().find(|m| m.cell == cell_a).unwrap();
    let msg_b = msgs.0.iter().find(|m| m.cell == cell_b).unwrap();

    assert!(
        (msg_a.damage - 20.0).abs() < f32::EPSILON,
        "cell A damage should be 10.0 * 2.0 = 20.0, got {}",
        msg_a.damage
    );
    assert!(
        (msg_b.damage - 50.0).abs() < f32::EPSILON,
        "cell B damage should be 25.0 * 2.0 = 50.0, got {}",
        msg_b.damage
    );
}

// ── Behavior 25: BoltBaseDamage absent falls back to DEFAULT_BOLT_BASE_DAMAGE ──

#[test]
fn collision_without_bolt_base_damage_falls_back_to_default() {
    // Given: Bolt with BoltBaseDamage(10.0) (from definition)
    // Then: damage is 10.0 (base_damage from definition)
    let mut app = test_app_with_damage_and_wall_messages();
    let cc = crate::cells::resources::CellConfig::default();
    let bc = super::helpers::test_bolt_definition();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    // The definition helper inserts BoltBaseDamage(10.0)
    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert_eq!(msgs.0.len(), 1, "should emit one DamageCell");
    assert!(
        (msgs.0[0].damage - DEFAULT_BOLT_BASE_DAMAGE).abs() < f32::EPSILON,
        "DamageCell.damage should fall back to DEFAULT_BOLT_BASE_DAMAGE ({DEFAULT_BOLT_BASE_DAMAGE}), got {}",
        msgs.0[0].damage
    );
}

#[test]
fn collision_without_bolt_base_damage_with_boost_uses_fallback() {
    // Edge case: BoltBaseDamage(10.0), ActiveDamageBoosts(2.0) -> 10.0 * 2.0 = 20.0
    let mut app = test_app_with_damage_and_wall_messages();
    let cc = crate::cells::resources::CellConfig::default();
    let bc = super::helpers::test_bolt_definition();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(ActiveDamageBoosts(vec![2.0]));

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert_eq!(msgs.0.len(), 1, "should emit one DamageCell");
    assert!(
        (msgs.0[0].damage - 20.0).abs() < f32::EPSILON,
        "DamageCell.damage should be DEFAULT_BOLT_BASE_DAMAGE * 2.0 = 20.0, got {}",
        msgs.0[0].damage
    );
}

// ── Behavior 26: pierce lookahead uses DEFAULT_BOLT_BASE_DAMAGE fallback ──

#[test]
fn collision_pierce_lookahead_uses_fallback_when_no_bolt_base_damage() {
    // Given: Bolt with BoltBaseDamage(10.0) from definition, cell HP 10.0
    // Then: effective damage = 10.0. 10.0 <= 10.0 -> pierce.
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = super::helpers::test_bolt_definition();

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 10.0);

    let start_y = cell_y - bc.radius - 25.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 10000.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert((ActivePiercings(vec![1]), PiercingRemaining(1)));

    tick(&mut app);

    let msgs = app.world().resource::<DamageCellMessages>();
    assert!(!msgs.0.is_empty(), "should emit at least one DamageCell");

    let pr = app
        .world()
        .get::<PiercingRemaining>(bolt_entity)
        .expect("bolt should have PiercingRemaining");
    assert_eq!(
        pr.0, 0,
        "PiercingRemaining should be decremented (fallback damage 10.0 >= HP 10.0), got {}",
        pr.0
    );
}

#[test]
fn collision_pierce_lookahead_fallback_insufficient_damage() {
    // Edge case: BoltBaseDamage(10.0), cell HP 15.0, effective 10.0 < 15.0 -> no pierce
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = super::helpers::test_bolt_definition();

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 15.0);

    let start_y = cell_y - bc.radius - 25.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 10000.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert((ActivePiercings(vec![1]), PiercingRemaining(1)));

    tick(&mut app);

    let pr = app
        .world()
        .get::<PiercingRemaining>(bolt_entity)
        .expect("bolt should have PiercingRemaining");
    assert_eq!(
        pr.0, 1,
        "PiercingRemaining should NOT be decremented (fallback 10.0 < HP 15.0), got {}",
        pr.0
    );
}
