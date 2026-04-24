//! Scheduling tests for `bolt_cell_collision`.
//!
//! Production guarantee: `BoltSystems::CellCollision` runs before
//! `EffectV3Systems::Bridge`, and `EffectV3Systems::Tick` is configured
//! `.before(DmgSystems::EmitDamage)` in `EffectV3Plugin` — so
//! `bolt_cell_collision` writes `DamageDealt<Cell>` before the damage pipeline
//! begins flushing.
//!
//! These tests pin the functional consequence: the cell's `Hp.current` after
//! one `tick(...)` matches the pre-W6 double-application formula
//! `final_hp = hp − base × boost² × vuln²`. W6 will correct the formula; until
//! then the buggy values are intentional.

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

/// Builds a `TestAppBuilder` wired for the W5 bolt-cell-collision scheduling
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

/// Returns true iff the cell entity no longer exists in the world.
/// Under the full damage pipeline, catastrophic damage flows
/// `apply_damage → detect_deaths → handle_kill → DespawnEntity →
/// process_despawn_requests` in a single `app.update()` — so a killed
/// cell is despawned, not merely marked `Dead`.
fn is_despawned(app: &App, cell: Entity) -> bool {
    app.world().get_entity(cell).is_err()
}

// ── Behavior 1 — bolt_cell_collision damage boost applies same-tick ─────────

/// With `DamageBoostStack(2.0)` on the bolt, the cell's HP after a single
/// tick matches the pre-W6 double-application formula:
/// `100.0 − 10.0 × 2.0² × 1.0² == 60.0`.
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
        (hp - 60.0).abs() < 1e-5,
        "Pre-W6 double-application: final_hp = 100.0 − 10.0 × 2.0² × 1.0² == 60.0, got {hp}"
    );
}

/// Behavior 1 edge case: `DamageBoostStack(&[2.0, 1.5])` aggregates to 3.0.
/// Pre-W6: `100.0 − 10.0 × 3.0² × 1.0² == 100.0 − 90.0 == 10.0`.
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
        (hp - 10.0).abs() < 1e-5,
        "Pre-W6: final_hp = 100.0 − 10.0 × 3.0² × 1.0² == 10.0, got {hp}"
    );
}

/// Behavior 1 negative-assertion smoke test: `system_in_set` returns `false`
/// when the system is NOT scheduled in the given schedule. Build an app
/// WITHOUT `BoltPlugin` so `bolt_cell_collision` is not scheduled; assert the
/// helper returns `false`. Without this smoke test, every Section A / B
/// assertion would be vacuous if the helper silently returned `true` in all
/// cases.
#[test]
fn system_in_set_returns_false_when_bolt_plugin_not_installed() {
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
         system_in_set must return false in this case — otherwise every \
         Section A/B scheduling assertion is vacuous."
    );
}

// ── Behavior 2 — bolt_cell_collision vulnerability applies same-tick ────────

/// Behavior 2 (HP delta): With `VulnerableStack(3.0)` on the cell and NO
/// `DamageBoostStack` on the bolt, the cell's HP after a single tick matches
/// the pre-W6 double-application formula:
/// `100.0 − 10.0 × 1.0² × 3.0² == 10.0`.
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
        (hp - 10.0).abs() < 1e-5,
        "Pre-W6: final_hp = 100.0 − 10.0 × 1.0² × 3.0² == 10.0, got {hp}"
    );
}

/// Behavior 2 edge case: BOTH `DamageBoostStack(2.0)` on bolt AND
/// `VulnerableStack(3.0)` on cell. Pre-W6 raw formula:
/// `100.0 − 10.0 × 2.0² × 3.0² = −260.0`. Under the full damage pipeline,
/// a catastrophic hit flows `apply_damage → detect_deaths → handle_kill →
/// DespawnEntity → process_despawn_requests` in a single `app.update()`,
/// so the cell is DESPAWNED by tick end (not merely marked `Dead`). The
/// observable assertion here is "cell entity no longer exists."
#[test]
fn bolt_cell_collision_applies_boost_and_vulnerability_same_tick_clamped() {
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

    assert!(
        is_despawned(&app, cell_entity),
        "Pre-W6 catastrophic damage (100.0 − 10.0 × 2.0² × 3.0² = −260.0) \
         must kill the cell. Under the full damage pipeline the cell is \
         despawned in the same tick; expected cell entity to no longer exist"
    );
}
