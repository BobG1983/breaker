//! Group T11–T12 — cross-frame dedup via `PhantomDamagedCells`.
//!
//! Post-Wave-3B contract:
//! - Phantom hits a cell → damage emitted once, cell added to `PhantomDamagedCells`.
//! - Phantom hits the SAME cell on a later frame → suppressed entirely (no damage,
//!   no `BoltImpactCell`).

use bevy::prelude::*;
use rantzsoft_spatial2d::components::BaseSpeed;

use super::helpers::*;
use crate::{
    bolt::{
        components::{LastImpact, PhantomDamagedCells, PiercingRemaining},
        systems::bolt_cell_collision::tests::helpers::*,
        test_utils::piercing_stack,
    },
    prelude::*,
};

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
