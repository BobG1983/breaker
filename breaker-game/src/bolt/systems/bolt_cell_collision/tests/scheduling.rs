//! Scheduling tests for `bolt_cell_collision`.
//!
//! Production guarantee: `BoltSystems::CellCollision` runs before
//! `EffectV3Systems::Bridge`, and `EffectV3Systems::Tick` is configured
//! `.before(DmgSystems::EmitDamage)` in `EffectV3Plugin` — so
//! `bolt_cell_collision` writes `DamageDealt<Cell>` before the damage pipeline
//! begins flushing.
//!
//! Post-W6, these tests pin the functional consequence: the cell's `Hp.current`
//! after one `tick(...)` matches the single-application formula
//! `final_hp = hp − base × boost × vuln`. `bolt_cell_collision` now emits RAW
//! `base_damage` and the crate pipeline (`apply_damage_boosts::<Cell>` +
//! `apply_vulnerable::<Cell>`) multiplies once.

use bevy::prelude::*;

use super::{
    super::system::bolt_cell_collision,
    helpers::{
        spawn_cell_with_health as spawn_cell_with_hp,
        spawn_vulnerable_cell as spawn_cell_with_hp_and_vuln,
    },
};
use crate::{
    bolt::{
        BoltPlugin,
        test_utils::{damage_stack, default_bolt_definition, spawn_bolt},
    },
    cells::resources::CellConfig,
    prelude::*,
};

// ── Shared helpers (local to scheduling tests) ──────────────────────────────

/// Builds a `TestAppBuilder` wired for the W6 bolt-cell-collision scheduling
/// tests: state hierarchy in Playing, physics, playfield, effects pipeline
/// (which installs `RantzDmgPlugin` and registers every `Dmgable`), plus
/// `BoltPlugin` so the PRODUCTION `bolt_cell_collision` scheduling is under
/// test.
fn scheduling_test_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_physics()
        .with_playfield()
        .with_bolt_registry()
        .with_breaker_registry()
        .with_cell_registry()
        .with_resource::<crate::input::resources::InputActions>()
        .with_effects_pipeline()
        .build();
    app.add_plugins(BoltPlugin);
    app
}

/// Reads the cell's `Hp.current`. Returns `None` if the entity or component
/// is absent (e.g. despawned after death).
fn read_hp(app: &App, cell: Entity) -> Option<f32> {
    app.world().get::<Hp>(cell).map(|h| h.current)
}

// ── Behavior 1 — bolt_cell_collision damage boost applies same-tick ─────────

/// With `DamageBoostStack(2.0)` on the bolt, the cell's HP after a single
/// tick matches the post-W6 single-application formula:
/// `100.0 − 10.0 × 2.0 × 1.0 == 80.0`.
#[test]
fn bolt_cell_collision_applies_damage_boost_in_same_tick() {
    let mut app = scheduling_test_app();
    let bc = default_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    let cell_entity = spawn_cell_with_hp(&mut app, 0.0, cell_y, 100.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(damage_stack(&[2.0]));

    tick(&mut app);

    let hp = read_hp(&app, cell_entity).unwrap_or(f32::NAN);
    assert!(
        (hp - 80.0).abs() < 1e-5,
        "Post-W6 single-application: final_hp = 100.0 − 10.0 × 2.0 × 1.0 == 80.0, got {hp}"
    );
}

/// Behavior 1 edge case: `DamageBoostStack(&[2.0, 1.5])` aggregates to 3.0.
/// Post-W6: `100.0 − 10.0 × 3.0 × 1.0 == 70.0`.
#[test]
fn bolt_cell_collision_applies_aggregated_damage_boost_in_same_tick() {
    let mut app = scheduling_test_app();
    let bc = default_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    let cell_entity = spawn_cell_with_hp(&mut app, 0.0, cell_y, 100.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(damage_stack(&[2.0, 1.5]));

    tick(&mut app);

    let hp = read_hp(&app, cell_entity).unwrap_or(f32::NAN);
    assert!(
        (hp - 70.0).abs() < 1e-5,
        "Post-W6: final_hp = 100.0 − 10.0 × 3.0 × 1.0 == 70.0, got {hp}"
    );
}

/// `system_in_set` helper sanity check: returns `false` when the system is
/// NOT scheduled in the given schedule. Post-W6, `bolt_cell_collision` does
/// NOT tag `DmgSystems::EmitDamage` — it is ordered transitively via
/// `BoltSystems::CellCollision → EffectV3Systems::Bridge →
/// EffectV3Systems::Tick.before(DmgSystems::EmitDamage)`. This negative
/// assertion is a smoke test for the `system_in_set` helper itself: without
/// `BoltPlugin` the system isn't scheduled at all, so the helper must return
/// false. (The helper's positive return is vacuously not tested here; this
/// only pins that it doesn't silently return true for unscheduled systems.)
#[test]
fn system_in_set_returns_false_when_bolt_plugin_not_installed() {
    // Post-W6, bolt_cell_collision does NOT tag DmgSystems::EmitDamage — it is
    // ordered transitively via BoltSystems::CellCollision → EffectV3Systems::Bridge
    // → EffectV3Systems::Tick.before(DmgSystems::EmitDamage). This negative assertion
    // is a smoke test for the `system_in_set` helper itself: without BoltPlugin the
    // system isn't scheduled at all, so the helper must return false. (The helper's
    // positive return is vacuously not tested here; this only pins that it doesn't
    // silently return true for unscheduled systems.)
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_physics()
        .with_playfield()
        .with_bolt_registry()
        .with_breaker_registry()
        .with_cell_registry()
        .with_resource::<crate::input::resources::InputActions>()
        .with_effects_pipeline()
        .build();

    assert!(
        !system_in_set(
            &mut app,
            FixedUpdate,
            bolt_cell_collision,
            DmgSystems::EmitDamage,
        ),
        "Without BoltPlugin, bolt_cell_collision is not scheduled at all. \
         system_in_set must return false in this case — otherwise this \
         helper sanity check is vacuous."
    );
}

// ── Behavior 2 — bolt_cell_collision vulnerability applies same-tick ────────

/// Behavior 2 (HP delta): With `VulnerableStack(3.0)` on the cell and NO
/// `DamageBoostStack` on the bolt, the cell's HP after a single tick matches
/// the post-W6 single-application formula:
/// `100.0 − 10.0 × 1.0 × 3.0 == 70.0`.
#[test]
fn bolt_cell_collision_applies_vulnerable_stack_in_same_tick() {
    let mut app = scheduling_test_app();
    let bc = default_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    let cell_entity = spawn_cell_with_hp_and_vuln(&mut app, 0.0, cell_y, 100.0, 3.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);

    tick(&mut app);

    let hp = read_hp(&app, cell_entity).unwrap_or(f32::NAN);
    assert!(
        (hp - 70.0).abs() < 1e-5,
        "Post-W6: final_hp = 100.0 − 10.0 × 1.0 × 3.0 == 70.0, got {hp}"
    );
}

/// Behavior 2 edge case: BOTH `DamageBoostStack(2.0)` on bolt AND
/// `VulnerableStack(3.0)` on cell. Post-W6 single-application formula:
/// `100.0 − 10.0 × 2.0 × 3.0 == 40.0` — the cell SURVIVES with reduced HP.
/// (Pre-W6 double-application produced `−260.0`, catastrophically killing
/// the cell. Post-W6 the pipeline multiplies boost and vulnerability once
/// each and the cell stays alive.)
#[test]
fn bolt_cell_collision_applies_boost_and_vulnerability_same_tick_survives_with_reduced_hp() {
    let mut app = scheduling_test_app();
    let bc = default_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    let cell_entity = spawn_cell_with_hp_and_vuln(&mut app, 0.0, cell_y, 100.0, 3.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(damage_stack(&[2.0]));

    tick(&mut app);

    let hp = read_hp(&app, cell_entity).unwrap_or(f32::NAN);
    assert!(
        (hp - 40.0).abs() < 1e-5,
        "Post-W6: final_hp = 100.0 − 10.0 × 2.0 × 3.0 == 40.0 (cell SURVIVES); got {hp}"
    );
}
