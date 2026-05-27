//! Group H — `bolt_cell_collision` pierce integration for entities
//! carrying `PhantomBolt` (Behaviors H1–H6).
//!
//! A bolt entity with the `PhantomBolt` marker deals damage but does NOT
//! reflect off a cell, does NOT consume `PiercingRemaining` charges, and
//! does NOT stamp `LastImpact`. Real bolts (no `PhantomBolt`) continue to
//! behave exactly as before.
//!
//! These tests pin the cross-domain integration: writer-code adds a
//! `Query<(), With<PhantomBolt>>` check to `bolt_cell_collision` before
//! the `can_pierce && would_destroy` branch, promoting the entity to
//! "pierce always" when the marker is present.
//!
//! Group T11–T13 — cross-frame dedup via `PhantomDamagedCells` + revert.
//!
//! Post-Wave-3B contract:
//! - Phantom hits a cell → damage emitted once, cell added to `PhantomDamagedCells`.
//! - Phantom hits the SAME cell on a later frame → suppressed entirely (no damage,
//!   no `BoltImpactCell`).
//! - Phantom reverts via `PhantomBolt::become_normal` → next cell contact uses the
//!   normal Reflect path.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{BaseSpeed, GlobalPosition2D, Spatial2D};

use super::helpers::*;
use crate::{
    bolt::{
        components::{
            BoltBaseDamage, LastImpact, LifetimeEndBehavior, PhantomBolt, PhantomDamagedCells,
            PhantomDedupKey, PiercingRemaining,
        },
        test_utils::piercing_stack,
    },
    cells::resources::CellConfig,
    prelude::*,
    shared::size::BaseRadius,
};

// ── Phantom spawner (Wave-1 vocabulary) ────────────────────────────────────

/// Spawns an entity that is itself a full bolt, then stamps it as a phantom
/// by directly inserting the phantom triple via `world_mut()`. Direct insertion
/// is synchronous — the components are present before any `app.update()` runs,
/// so `bolt_cell_collision` (`FixedUpdate`) sees the marker on the first tick.
///
/// `owner` parameter removed — `PhantomOwner`/`PhantomLifetime` are legacy
/// types being deleted in Wave 5; they are not part of the Wave 1 component
/// vocabulary and must not be inserted here.
fn spawn_phantom_bolt_for_cell_collision(
    app: &mut App,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    radius: f32,
) -> Entity {
    let pos = Vec2::new(x, y);
    let velocity = Vec2::new(vx, vy);
    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            Velocity2D(velocity),
            BaseSpeed(velocity.length()),
            BoltBaseDamage(10.0),
            BaseRadius(radius),
            CollisionLayers::new(
                BOLT_LAYER,
                BOLT_LAYER | WALL_LAYER | BREAKER_LAYER | CELL_LAYER,
            ),
        ))
        .id();
    app.world_mut().entity_mut(entity).insert((
        PhantomBolt,
        PhantomDedupKey::Bolt(entity),
        PhantomDamagedCells::default(),
    ));
    entity
}

// ── H1 — phantom deals damage but does NOT reflect ────────────────────────

#[test]
fn phantom_bolt_entity_deals_damage_but_does_not_reflect_off_cell() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 30.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let phantom =
        spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, start_y, 0.0, 400.0, bc.radius);

    tick(&mut app);

    let velocity = app.world().get::<Velocity2D>(phantom).unwrap();
    assert!(
        velocity.0.y > 0.0,
        "phantom MUST NOT be reflected — velocity.y should remain upward, got {}",
        velocity.0.y
    );

    let damages: Vec<_> = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(phantom))
        .collect();
    assert_eq!(
        damages.len(),
        1,
        "exactly one DamageDealt<Cell> from the phantom, got {}",
        damages.len()
    );
    assert!(
        (damages[0].amount - 10.0).abs() < 1e-4,
        "damage amount expected 10.0 (base damage), got {}",
        damages[0].amount
    );
}

// ── H1 (edge case) — phantom does NOT get LastImpact stamped ──────────────

#[test]
fn phantom_bolt_entity_does_not_get_last_impact_stamped() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 30.0);
    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let phantom =
        spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, start_y, 0.0, 400.0, bc.radius);

    tick(&mut app);

    assert!(
        app.world().get::<LastImpact>(phantom).is_none(),
        "phantom MUST NOT have LastImpact stamped (pierce-through semantics)"
    );
}

// ── H2 — phantom pierces a cell it would NOT destroy ──────────────────────

#[test]
fn phantom_bolt_pierces_cell_it_would_not_destroy() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    // 100 Hp — damage 10 would NOT kill → normal pierce gate reflects.
    // PhantomBolt marker must override.
    spawn_cell_with_health(&mut app, 0.0, cell_y, 100.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let phantom =
        spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, start_y, 0.0, 400.0, bc.radius);

    tick(&mut app);

    let velocity = app.world().get::<Velocity2D>(phantom).unwrap();
    assert!(
        velocity.0.y > 0.0,
        "PhantomBolt marker must override would-destroy requirement — \
         phantom must pierce even high-Hp cells. Got velocity.y={}",
        velocity.0.y
    );

    let damages: Vec<_> = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(phantom))
        .collect();
    assert_eq!(damages.len(), 1, "exactly one damage message emitted");
    assert!(
        (damages[0].amount - 10.0).abs() < 1e-4,
        "damage amount 10.0 (unchanged), got {}",
        damages[0].amount
    );
}

// ── H3 — phantom pierces multiple cells in one frame ─────────────────────-

#[test]
fn phantom_bolt_pierces_multiple_cells_in_one_frame() {
    let mut app = test_app_with_damage_and_wall_messages();

    let cell_a = spawn_cell_with_health(&mut app, 0.0, 100.0, 30.0);
    let cell_b = spawn_cell_with_health(&mut app, 0.0, 130.0, 30.0);

    let phantom = spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, 50.0, 0.0, 6000.0, 6.0);

    tick(&mut app);

    let damages: Vec<_> = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(phantom))
        .collect();
    let targets: Vec<Entity> = damages.iter().map(|d| d.target).collect();
    assert_eq!(
        damages.len(),
        2,
        "expected two DamageDealt<Cell> messages, got {}",
        damages.len()
    );
    assert!(
        targets.contains(&cell_a),
        "must target cell_a, got {targets:?}"
    );
    assert!(
        targets.contains(&cell_b),
        "must target cell_b, got {targets:?}"
    );

    let velocity = app.world().get::<Velocity2D>(phantom).unwrap();
    assert!(
        velocity.0.y > 0.0,
        "phantom velocity direction preserved through pierces, got {}",
        velocity.0.y
    );
    let position = app.world().get::<Position2D>(phantom).unwrap();
    assert!(
        position.0.y > 130.0 - 6.0,
        "phantom must travel past cell_b (y > 130.0 - radius), got y={}",
        position.0.y
    );
}

// ── H4 — phantom does NOT consume PiercingRemaining charges ───────────────

#[test]
fn phantom_bolt_does_not_consume_piercing_remaining_charges() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 10.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let phantom =
        spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, start_y, 0.0, 400.0, bc.radius);
    // Attach PiercingRemaining + piercing_stack.
    app.world_mut()
        .entity_mut(phantom)
        .insert((piercing_stack(&[3]), PiercingRemaining(3)));

    tick(&mut app);

    let pr = app
        .world()
        .get::<PiercingRemaining>(phantom)
        .expect("PiercingRemaining must still be present");
    assert_eq!(
        pr.0, 3,
        "PhantomBolt pierce must NOT consume PiercingRemaining charges, got {}",
        pr.0
    );
    let velocity = app.world().get::<Velocity2D>(phantom).unwrap();
    assert!(
        velocity.0.y > 0.0,
        "phantom must not reflect even with PiercingRemaining, got velocity.y={}",
        velocity.0.y
    );
}

// ── H4 (edge case) — phantom with PiercingRemaining(0) still pierces ──────

#[test]
fn phantom_bolt_with_zero_piercing_remaining_still_pierces() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 30.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let phantom =
        spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, start_y, 0.0, 400.0, bc.radius);
    app.world_mut()
        .entity_mut(phantom)
        .insert((piercing_stack(&[]), PiercingRemaining(0)));

    tick(&mut app);

    let velocity = app.world().get::<Velocity2D>(phantom).unwrap();
    assert!(
        velocity.0.y > 0.0,
        "phantom pierce is unconditional on marker — charges irrelevant. \
         Got velocity.y={}",
        velocity.0.y
    );
}

// ── H5 — real bolt (no PhantomBolt marker) reflects normally ──────────────

#[test]
fn real_bolt_without_phantom_marker_reflects_off_cell() {
    let mut app = test_app();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();
    app.insert_resource(HitCells::default()).add_systems(
        FixedUpdate,
        collect_cell_hits
            .after(crate::bolt::systems::bolt_cell_collision::system::bolt_cell_collision),
    );

    let cell_y = 100.0;
    spawn_cell_with_health(&mut app, 0.0, cell_y, 30.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);

    tick(&mut app);

    let vel = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(
        vel.0.y < 0.0,
        "real bolt (no PhantomBolt marker) must reflect — regression guard. \
         Got velocity.y={}",
        vel.0.y
    );
    let hits = app.world().resource::<HitCells>();
    assert_eq!(hits.0.len(), 1, "BoltImpactCell must still be emitted");
}

// ── H6 — real + phantom co-existing: real reflects, phantom pierces ───────

#[test]
fn real_and_phantom_bolt_co_exist_real_reflects_phantom_pierces() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();

    let cell = spawn_cell_with_health(&mut app, 0.0, 100.0, 30.0);

    let start_y = 100.0 - cc.height / 2.0 - bc.radius - 2.0;
    let real = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    let phantom =
        spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, start_y, 0.0, 400.0, bc.radius);

    tick(&mut app);

    let real_vel = app.world().get::<Velocity2D>(real).unwrap();
    assert!(
        real_vel.0.y < 0.0,
        "real bolt must reflect, got velocity.y={}",
        real_vel.0.y
    );
    let phantom_vel = app.world().get::<Velocity2D>(phantom).unwrap();
    assert!(
        phantom_vel.0.y > 0.0,
        "phantom bolt must pierce, got velocity.y={}",
        phantom_vel.0.y
    );

    let damages: Vec<_> = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|d| d.target == cell)
        .collect();
    assert_eq!(
        damages.len(),
        2,
        "both bolts must hit the cell — two DamageDealt<Cell>, got {}",
        damages.len()
    );

    assert!(
        app.world().get_entity(real).is_ok(),
        "real bolt still alive"
    );
    assert!(
        app.world().get_entity(phantom).is_ok(),
        "phantom bolt still alive"
    );
}

// ── H6 (edge case) — real gets LastImpact stamped, phantom does NOT ───────

#[test]
fn real_bolt_gets_last_impact_but_phantom_does_not() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();

    spawn_cell_with_health(&mut app, 0.0, 100.0, 30.0);

    let start_y = 100.0 - cc.height / 2.0 - bc.radius - 2.0;
    let real = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    let phantom =
        spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, start_y, 0.0, 400.0, bc.radius);

    tick(&mut app);

    assert!(
        app.world().get::<LastImpact>(real).is_some(),
        "real bolt must get LastImpact stamped on reflect"
    );
    assert!(
        app.world().get::<LastImpact>(phantom).is_none(),
        "phantom bolt must NOT get LastImpact stamped"
    );
}

// ── T11 — Phantom-bolt cell hit: damage emitted exactly once, velocity unchanged ──

#[test]
fn phantom_bolt_cell_hit_emits_damage_once_and_does_not_flip_velocity() {
    let mut app = test_app_with_damage_and_wall_messages();

    // High-velocity bolt guarantees CCD impact within a single 60 Hz tick
    // (100 px travel budget covers the ~10 px gap to the cell bottom face).
    let cell_entity = spawn_cell_with_health(&mut app, 0.0, 100.0, 30.0);
    // start_y = 100.0 - 12.0 (half cell height) - 8.0 (bolt radius) - 2.0 = 78.0
    let phantom = spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, 78.0, 0.0, 6000.0, 8.0);
    // Re-insert BaseSpeed to match the high-velocity setup
    app.world_mut()
        .entity_mut(phantom)
        .insert(BaseSpeed(6000.0));

    tick(&mut app);

    // Velocity unchanged (no reflect, forward direction preserved)
    let vel = app.world().get::<Velocity2D>(phantom).unwrap();
    assert!(
        (vel.0.x - 0.0).abs() < 1e-3,
        "velocity.x must be 0.0, got {}",
        vel.0.x
    );
    assert!(
        vel.0.y > 0.0,
        "velocity.y must remain positive (no reflect), got {}",
        vel.0.y
    );
    assert!(
        (vel.0.y - 6000.0).abs() < 1e-1,
        "velocity.y magnitude must be ~6000.0 after apply_velocity_formula, got {}",
        vel.0.y
    );

    // Exactly one DamageDealt<Cell> for this phantom on this cell
    let damages: Vec<_> = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(phantom) && m.target == cell_entity)
        .collect();
    assert_eq!(
        damages.len(),
        1,
        "exactly one DamageDealt<Cell> expected, got {}",
        damages.len()
    );
    assert!(
        (damages[0].amount - 10.0).abs() < 1e-4,
        "damage amount must be 10.0 (raw base_damage), got {}",
        damages[0].amount
    );

    // Exactly one BoltImpactCell message
    let hit_count = app
        .world()
        .resource::<FullHitMessages>()
        .0
        .iter()
        .filter(|m| m.bolt == phantom && m.cell == cell_entity)
        .count();
    assert_eq!(
        hit_count, 1,
        "exactly one BoltImpactCell expected, got {hit_count}"
    );

    // LastImpact not stamped
    assert!(
        app.world().get::<LastImpact>(phantom).is_none(),
        "phantom must not receive LastImpact"
    );

    // PhantomDamagedCells now contains the cell
    let deduped_cells = app
        .world()
        .get::<PhantomDamagedCells>(phantom)
        .expect("PhantomDamagedCells must be present on phantom");
    assert!(
        deduped_cells.0.contains(&cell_entity),
        "cell_entity must be in PhantomDamagedCells after first hit"
    );
    assert_eq!(
        deduped_cells.0.len(),
        1,
        "PhantomDamagedCells must contain exactly one entry, got {}",
        deduped_cells.0.len()
    );

    // Position advanced past the cell (top of cell + bolt radius = 120.0)
    let pos = app.world().get::<Position2D>(phantom).unwrap();
    assert!(
        pos.0.y > 120.0,
        "phantom must have crossed the cell fully (y > 120.0), got {}",
        pos.0.y
    );
}

// ── T11 edge case 1a — angled velocity preserved through phantom pierce ────

#[test]
fn phantom_bolt_cell_hit_angled_velocity_preserved() {
    let mut app = test_app_with_damage_and_wall_messages();

    spawn_cell_with_health(&mut app, 0.0, 100.0, 30.0);
    let phantom = spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, 78.0, 3000.0, 6000.0, 8.0);
    let speed = 3000.0_f32.hypot(6000.0_f32);
    app.world_mut().entity_mut(phantom).insert(BaseSpeed(speed));

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(phantom).unwrap();
    assert!(
        (vel.0.x - 3000.0).abs() < 1.0,
        "velocity.x must be ~3000.0, got {}",
        vel.0.x
    );
    assert!(
        (vel.0.y - 6000.0).abs() < 1.0,
        "velocity.y must be ~6000.0, got {}",
        vel.0.y
    );
    let pos = app.world().get::<Position2D>(phantom).unwrap();
    // Position must have advanced along the velocity direction — y must be past the cell
    assert!(
        pos.0.y > 120.0,
        "phantom must travel past the cell, got y={}",
        pos.0.y
    );
}

// ── T11 edge case 1b — phantom pierces high-Hp cell under new dedup contract ──

#[test]
fn phantom_bolt_cell_hit_high_hp_cell_still_pierces_with_dedup_contract() {
    let mut app = test_app_with_damage_and_wall_messages();

    let cell_entity = spawn_cell_with_health(&mut app, 0.0, 100.0, 1000.0);
    let phantom = spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, 78.0, 0.0, 6000.0, 8.0);
    app.world_mut()
        .entity_mut(phantom)
        .insert(BaseSpeed(6000.0));

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(phantom).unwrap();
    assert!(
        vel.0.y > 0.0,
        "phantom must pierce high-Hp cell, got velocity.y={}",
        vel.0.y
    );

    let damages: Vec<_> = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(phantom) && m.target == cell_entity)
        .collect();
    assert_eq!(
        damages.len(),
        1,
        "exactly one damage message, got {}",
        damages.len()
    );

    let deduped_cells = app.world().get::<PhantomDamagedCells>(phantom).unwrap();
    assert!(
        deduped_cells.0.contains(&cell_entity),
        "high-Hp cell must be added to PhantomDamagedCells"
    );
}

// ── T11 edge case 1c — PiercingRemaining(3) not consumed on phantom pierce ──

#[test]
fn phantom_bolt_cell_hit_does_not_consume_piercing_remaining() {
    let mut app = test_app_with_damage_and_wall_messages();

    spawn_cell_with_health(&mut app, 0.0, 100.0, 30.0);
    let phantom = spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, 78.0, 0.0, 6000.0, 8.0);
    app.world_mut().entity_mut(phantom).insert((
        BaseSpeed(6000.0),
        piercing_stack(&[3]),
        PiercingRemaining(3),
    ));

    tick(&mut app);

    let pr = app.world().get::<PiercingRemaining>(phantom).unwrap();
    assert_eq!(
        pr.0, 3,
        "PiercingRemaining must not be consumed on phantom pierce, got {}",
        pr.0
    );
}

// ── T11 edge case 1d — PiercingRemaining(0) still pierces on phantom ──────

#[test]
fn phantom_bolt_cell_hit_with_zero_piercing_remaining_still_pierces() {
    let mut app = test_app_with_damage_and_wall_messages();

    let cell_entity = spawn_cell_with_health(&mut app, 0.0, 100.0, 30.0);
    let phantom = spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, 78.0, 0.0, 6000.0, 8.0);
    app.world_mut().entity_mut(phantom).insert((
        BaseSpeed(6000.0),
        piercing_stack(&[]),
        PiercingRemaining(0),
    ));

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(phantom).unwrap();
    assert!(
        vel.0.y > 0.0,
        "phantom must pierce even with PiercingRemaining=0"
    );

    let damaged = app.world().get::<PhantomDamagedCells>(phantom).unwrap();
    assert!(
        damaged.0.contains(&cell_entity),
        "cell must be added to PhantomDamagedCells even with PiercingRemaining=0"
    );
}

// ── T12 — Phantom impacts same cell on consecutive frames: dedup via PhantomDamagedCells ──

#[test]
fn phantom_bolt_same_cell_consecutive_frames_dedups_via_phantom_damaged_cells() {
    let mut app = test_app_with_damage_and_wall_messages();

    let cell_entity = spawn_cell_with_health(&mut app, 0.0, 100.0, 1000.0);
    let phantom = spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, 78.0, 0.0, 6000.0, 8.0);
    app.world_mut()
        .entity_mut(phantom)
        .insert(BaseSpeed(6000.0));

    // Tick 1: first contact — damage emitted once
    tick(&mut app);

    let count_after_tick1 = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(phantom) && m.target == cell_entity)
        .count();
    assert_eq!(
        count_after_tick1, 1,
        "after tick 1: exactly one DamageDealt<Cell>, got {count_after_tick1}"
    );
    let damaged_after_tick1 = app.world().get::<PhantomDamagedCells>(phantom).unwrap();
    assert!(
        damaged_after_tick1.0.contains(&cell_entity),
        "cell must be in PhantomDamagedCells after tick 1"
    );
    assert_eq!(
        damaged_after_tick1.0.len(),
        1,
        "PhantomDamagedCells must have 1 entry after tick 1"
    );

    // Reposition bolt back to before the cell for tick 2
    app.world_mut()
        .entity_mut(phantom)
        .insert(Position2D(Vec2::new(0.0, 78.0)));

    // Tick 2: CCD re-encounters cell_entity — cross-frame dedup must suppress
    tick(&mut app);

    let count_after_tick2 = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(phantom) && m.target == cell_entity)
        .count();
    assert_eq!(
        count_after_tick2, 1,
        "after tick 2: DamageDealt<Cell> count must still be 1 (dedup suppressed), got {count_after_tick2}"
    );

    // BoltImpactCell count also stays at 1
    let impact_count = app
        .world()
        .resource::<FullHitMessages>()
        .0
        .iter()
        .filter(|m| m.bolt == phantom && m.cell == cell_entity)
        .count();
    assert_eq!(
        impact_count, 1,
        "BoltImpactCell must also be suppressed on tick 2 — total must be 1, got {impact_count}"
    );

    // Velocity still forward (no reflect on tick 2 either)
    let vel = app.world().get::<Velocity2D>(phantom).unwrap();
    assert!(
        vel.0.y > 0.0,
        "phantom must not reflect on deduped tick 2, got velocity.y={}",
        vel.0.y
    );

    // Position advanced past the cell on tick 2 as well
    let pos = app.world().get::<Position2D>(phantom).unwrap();
    assert!(
        pos.0.y > 120.0,
        "phantom must still travel past the cell on tick 2, got y={}",
        pos.0.y
    );

    // PhantomDamagedCells unchanged in size
    let damaged_after_tick2 = app.world().get::<PhantomDamagedCells>(phantom).unwrap();
    assert_eq!(
        damaged_after_tick2.0.len(),
        1,
        "PhantomDamagedCells must still contain only 1 entry after tick 2"
    );
}

// ── T12 edge case 2a — two cells, one already in PhantomDamagedCells ──────

#[test]
fn phantom_bolt_skips_already_damaged_cell_but_hits_new_cell() {
    use std::collections::HashSet;

    let mut app = test_app_with_damage_and_wall_messages();

    // Place cell_a (pre-damaged) and cell_b (fresh) along the bolt path
    let cell_a = spawn_cell_with_health(&mut app, 0.0, 100.0, 1000.0);
    let cell_b = spawn_cell_with_health(&mut app, 0.0, 130.0, 1000.0);

    let phantom = spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, 50.0, 0.0, 6000.0, 8.0);
    app.world_mut()
        .entity_mut(phantom)
        .insert(BaseSpeed(6000.0));

    // Pre-populate PhantomDamagedCells with cell_a
    let mut pre_set = HashSet::new();
    pre_set.insert(cell_a);
    app.world_mut()
        .entity_mut(phantom)
        .insert(PhantomDamagedCells(pre_set));

    tick(&mut app);

    // Only cell_b should have received damage
    let damages_a = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(phantom) && m.target == cell_a)
        .count();
    assert_eq!(
        damages_a, 0,
        "cell_a (already in PhantomDamagedCells) must receive NO damage, got {damages_a}"
    );

    let damages_b = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(phantom) && m.target == cell_b)
        .count();
    assert_eq!(
        damages_b, 1,
        "cell_b (not in set) must receive exactly one damage message, got {damages_b}"
    );

    // BoltImpactCell for cell_a is suppressed
    let impact_a = app
        .world()
        .resource::<FullHitMessages>()
        .0
        .iter()
        .filter(|m| m.bolt == phantom && m.cell == cell_a)
        .count();
    assert_eq!(
        impact_a, 0,
        "BoltImpactCell for cell_a must be suppressed, got {impact_a}"
    );

    // PhantomDamagedCells now contains both
    let damaged = app.world().get::<PhantomDamagedCells>(phantom).unwrap();
    assert_eq!(
        damaged.0.len(),
        2,
        "PhantomDamagedCells must contain both cell_a and cell_b, got {} entries",
        damaged.0.len()
    );
    assert!(damaged.0.contains(&cell_a), "cell_a must still be in set");
    assert!(damaged.0.contains(&cell_b), "cell_b must now be in set");
}

// ── T12 edge case 2b — three consecutive ticks, damage emitted only on tick 1 ──

#[test]
fn phantom_bolt_same_cell_three_consecutive_frames_damage_emitted_only_once() {
    let mut app = test_app_with_damage_and_wall_messages();

    let cell_entity = spawn_cell_with_health(&mut app, 0.0, 100.0, 1000.0);
    let phantom = spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, 78.0, 0.0, 6000.0, 8.0);
    app.world_mut()
        .entity_mut(phantom)
        .insert(BaseSpeed(6000.0));

    // Tick 1
    tick(&mut app);

    // Reposition and tick 2
    app.world_mut()
        .entity_mut(phantom)
        .insert(Position2D(Vec2::new(0.0, 78.0)));
    tick(&mut app);

    // Reposition and tick 3
    app.world_mut()
        .entity_mut(phantom)
        .insert(Position2D(Vec2::new(0.0, 78.0)));
    tick(&mut app);

    let count = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(phantom) && m.target == cell_entity)
        .count();
    assert_eq!(
        count, 1,
        "after three ticks on same cell, damage count must still be 1, got {count}"
    );
}

// ── T12 edge case 2c — empty PhantomDamagedCells + first hit → emits damage ──

#[test]
fn phantom_bolt_empty_damaged_cells_first_hit_emits_damage_and_inserts() {
    let mut app = test_app_with_damage_and_wall_messages();

    let cell_entity = spawn_cell_with_health(&mut app, 0.0, 100.0, 30.0);
    let phantom = spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, 78.0, 0.0, 6000.0, 8.0);
    app.world_mut()
        .entity_mut(phantom)
        .insert(BaseSpeed(6000.0));

    // Confirm PhantomDamagedCells starts empty
    let before = app.world().get::<PhantomDamagedCells>(phantom).unwrap();
    assert!(
        before.0.is_empty(),
        "PhantomDamagedCells must be empty before first tick"
    );

    tick(&mut app);

    let damages = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(phantom) && m.target == cell_entity)
        .count();
    assert_eq!(damages, 1, "first hit must emit exactly one damage message");

    let after = app.world().get::<PhantomDamagedCells>(phantom).unwrap();
    assert!(
        after.0.contains(&cell_entity),
        "cell must be in PhantomDamagedCells after first hit"
    );
}

// ── T13 — Reverted phantom resumes normal cell rebound on next contact ─────

#[test]
fn reverted_phantom_bolt_resumes_normal_cell_rebound_on_next_contact() {
    let mut app = test_app_with_damage_and_wall_messages();

    let cell_entity = spawn_cell_with_health(&mut app, 0.0, 100.0, 1000.0);

    // Spawn phantom, then revert BEFORE any collision
    let bolt = spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, 78.0, 0.0, 6000.0, 8.0);
    app.world_mut().entity_mut(bolt).insert(BaseSpeed(6000.0));

    // Sentinel: become_phantom stub must have inserted PhantomBolt — if absent,
    // the no-op stub false-passes all subsequent T13 assertions.
    assert!(
        app.world().get::<PhantomBolt>(bolt).is_some(),
        "pre-condition: PhantomBolt must be present — become_phantom stub must have inserted it"
    );

    // Revert directly via world_mut — avoids depending on Wave 3A's lifespan tick
    app.world_mut().entity_mut(bolt).remove::<(
        PhantomBolt,
        PhantomDedupKey,
        PhantomDamagedCells,
        LifetimeEndBehavior,
    )>();

    // Pre-condition: no PhantomBolt marker
    assert!(
        app.world().get::<PhantomBolt>(bolt).is_none(),
        "pre-condition: bolt must not have PhantomBolt marker before tick"
    );

    tick(&mut app);

    // Normal bolt must reflect (vy < 0 after bounce off cell top face)
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        vel.0.y < 0.0,
        "reverted bolt must reflect off cell (vy < 0), got vy={}",
        vel.0.y
    );

    // LastImpact stamped (normal reflect path)
    assert!(
        app.world().get::<LastImpact>(bolt).is_some(),
        "reverted bolt must get LastImpact stamped"
    );

    // Damage emitted once
    let damages = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(bolt) && m.target == cell_entity)
        .count();
    assert_eq!(
        damages, 1,
        "reverted bolt must emit exactly one damage message"
    );

    // PhantomDamagedCells absent (removed by revert)
    assert!(
        app.world().get::<PhantomDamagedCells>(bolt).is_none(),
        "PhantomDamagedCells must be absent after revert"
    );

    // PhantomBolt absent
    assert!(
        app.world().get::<PhantomBolt>(bolt).is_none(),
        "PhantomBolt must remain absent after revert"
    );
}

// ── T13 edge case 3a — phantom hits cell_x, reverts, then hits different cell_y normally ──

#[test]
fn reverted_phantom_hits_different_cell_with_normal_reflect() {
    let mut app = test_app_with_damage_and_wall_messages();

    // cell_x at x=200 — off the bolt's vertical (x=0) path during phase 2 rebound,
    // so no double-bounce back into it after reflecting off cell_y.
    let cell_x = spawn_cell_with_health(&mut app, 200.0, 100.0, 1000.0);
    let cell_y_entity = spawn_cell_with_health(&mut app, 0.0, 200.0, 1000.0);

    // Phase 1: phantom hits cell_x — bolt travels at angle to reach cell_x at x=200
    let bolt = spawn_phantom_bolt_for_cell_collision(&mut app, 200.0, 78.0, 0.0, 6000.0, 8.0);
    app.world_mut().entity_mut(bolt).insert(BaseSpeed(6000.0));

    // Sentinel: become_phantom stub must have inserted PhantomBolt — bolt must be
    // phantom DURING phase 1's pierce tick; damage count alone doesn't catch a no-op stub.
    assert!(
        app.world().get::<PhantomBolt>(bolt).is_some(),
        "pre-condition: PhantomBolt must be present — become_phantom stub must have inserted it"
    );

    tick(&mut app);

    let phantom_damage_count = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(bolt) && m.target == cell_x)
        .count();
    assert_eq!(phantom_damage_count, 1, "phantom phase: cell_x hit once");

    // Revert the bolt
    app.world_mut().entity_mut(bolt).remove::<(
        PhantomBolt,
        PhantomDedupKey,
        PhantomDamagedCells,
        LifetimeEndBehavior,
    )>();
    assert!(
        app.world().get::<PhantomBolt>(bolt).is_none(),
        "bolt must be reverted before phase 2"
    );

    // Reposition to approach cell_y
    app.world_mut()
        .entity_mut(bolt)
        .insert(Position2D(Vec2::new(0.0, 178.0)));

    tick(&mut app);

    // Normal Reflect at cell_y
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        vel.0.y < 0.0,
        "reverted bolt must reflect off cell_y, got vy={}",
        vel.0.y
    );

    // Two total damage messages (one phantom on cell_x, one normal on cell_y)
    let total_damage = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(bolt))
        .count();
    assert_eq!(
        total_damage, 2,
        "total damage messages must be 2, got {total_damage}"
    );

    let cell_y_damage = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(bolt) && m.target == cell_y_entity)
        .count();
    assert_eq!(
        cell_y_damage, 1,
        "cell_y must receive exactly one normal-reflect damage"
    );
}

// ── T13 edge case 3b — phantom hits cell_x, reverts, re-enters cell_x normally ──

#[test]
fn reverted_phantom_re_enters_previously_damaged_cell_with_normal_reflect() {
    let mut app = test_app_with_damage_and_wall_messages();

    let cell_x = spawn_cell_with_health(&mut app, 0.0, 100.0, 1000.0);

    // Phase 1: phantom hits cell_x (adds to PhantomDamagedCells)
    let bolt = spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, 78.0, 0.0, 6000.0, 8.0);
    app.world_mut().entity_mut(bolt).insert(BaseSpeed(6000.0));

    // Sentinel: become_phantom stub must have inserted PhantomBolt before phase 1 tick.
    assert!(
        app.world().get::<PhantomBolt>(bolt).is_some(),
        "pre-condition: PhantomBolt must be present — become_phantom stub must have inserted it"
    );

    tick(&mut app);

    let phantom_count = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(bolt) && m.target == cell_x)
        .count();
    assert_eq!(phantom_count, 1, "phantom phase: one damage on cell_x");

    // Revert
    app.world_mut().entity_mut(bolt).remove::<(
        PhantomBolt,
        PhantomDedupKey,
        PhantomDamagedCells,
        LifetimeEndBehavior,
    )>();

    // Reposition back before cell_x
    app.world_mut()
        .entity_mut(bolt)
        .insert(Position2D(Vec2::new(0.0, 78.0)));

    tick(&mut app);

    // Normal Reflect at cell_x (dedup set was removed — fresh hit is allowed)
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        vel.0.y < 0.0,
        "reverted bolt must reflect off cell_x, got vy={}",
        vel.0.y
    );
    assert!(
        app.world().get::<LastImpact>(bolt).is_some(),
        "LastImpact must be stamped on normal reflect"
    );

    // Two total damage messages on cell_x (one phantom + one normal)
    let total = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(bolt) && m.target == cell_x)
        .count();
    assert_eq!(
        total, 2,
        "cell_x must receive 2 damage messages (phantom + normal), got {total}"
    );
}

// ── T13 edge case 3c — become_normal called twice is idempotent ───────────

#[test]
fn become_normal_called_twice_is_idempotent_and_subsequent_collision_reflects() {
    let mut app = test_app_with_damage_and_wall_messages();

    let cell_entity = spawn_cell_with_health(&mut app, 0.0, 100.0, 1000.0);

    let bolt = spawn_phantom_bolt_for_cell_collision(&mut app, 0.0, 78.0, 0.0, 6000.0, 8.0);
    app.world_mut().entity_mut(bolt).insert(BaseSpeed(6000.0));

    // Sentinel: become_phantom stub must have inserted PhantomBolt — if absent,
    // both reverts are no-ops and the bolt was never phantom.
    assert!(
        app.world().get::<PhantomBolt>(bolt).is_some(),
        "pre-condition: PhantomBolt must be present — become_phantom stub must have inserted it"
    );

    // First revert
    app.world_mut().entity_mut(bolt).remove::<(
        PhantomBolt,
        PhantomDedupKey,
        PhantomDamagedCells,
        LifetimeEndBehavior,
    )>();

    // Second revert (idempotent — removing absent components must not panic)
    app.world_mut().entity_mut(bolt).remove::<(
        PhantomBolt,
        PhantomDedupKey,
        PhantomDamagedCells,
        LifetimeEndBehavior,
    )>();

    assert!(
        app.world().get::<PhantomBolt>(bolt).is_none(),
        "PhantomBolt must remain absent after double revert"
    );

    tick(&mut app);

    // Bolt reflects normally
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        vel.0.y < 0.0,
        "bolt after double-revert must reflect normally, got vy={}",
        vel.0.y
    );

    let damages = app
        .world()
        .resource::<DamageDealtCellMessages>()
        .0
        .iter()
        .filter(|m| m.dealer == Some(bolt) && m.target == cell_entity)
        .count();
    assert_eq!(
        damages, 1,
        "exactly one damage message on normal reflect after double-revert"
    );
}
