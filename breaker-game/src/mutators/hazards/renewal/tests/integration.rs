//! Group G — Full pipeline integration with `apply_heal::<Cell>`.
//!
//! Wires `renewal_tick` AND `apply_heal::<Cell>` so that emitted messages
//! flow end-to-end. Observes post-tick `Hp.current`.

use std::time::Duration;

use bevy::prelude::*;
use rantzsoft_dmg::{RantzDmgAppExt, RantzDmgPlugin};

use super::{
    super::system::{RenewalTimer, renewal_tick},
    helpers::{
        add_renewal_stacks, attach_timer, canonical_config, install_renewal_config, spawn_cell,
        spawn_cell_with_max, test_app_playing, tick_with_dt,
    },
};
use crate::{cells::components::Cell, prelude::*};

/// Builder for Group G: wires `renewal_tick` before `apply_heal::<Cell>`
/// in `DmgSystems::ApplyHeal`. Leaves out the production
/// `.after(DmgSystems::ApplyKill)` ordering to keep the focus
/// on the `renewal_tick` → `ApplyHeal` step; the full chained registration
/// is covered by Group F.
fn test_app_pipeline() -> App {
    let mut app = test_app_playing();
    app.add_plugins(RantzDmgPlugin);
    let _ = app.register_dmgable::<Cell>();
    app.add_systems(FixedUpdate, renewal_tick.before(DmgSystems::ApplyHeal));
    app
}

// ── Behavior 27 — Damaged cell heals to starting end-to-end ───────────────

#[test]
fn pipeline_damaged_cell_heals_to_starting() {
    let mut app = test_app_pipeline();
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 30.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 100.0).abs() < f32::EPSILON,
        "cell should heal to starting (100.0), got {}",
        hp.current
    );
    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        (timer.remaining - 10.0).abs() < 1e-5,
        "timer should reset to 10.0 after damaged-cell heal lands via pipeline, got {}",
        timer.remaining
    );
}

#[test]
fn pipeline_one_hp_cell_heals_to_starting() {
    // Edge: start at 1.0 — ends at 100.0.
    let mut app = test_app_pipeline();
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 1.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 100.0).abs() < f32::EPSILON,
        "cell at 1.0 HP should heal to 100.0, got {}",
        hp.current
    );
}

// ── Behavior 28 — Full-HP cell unchanged end-to-end ──────────────────────

#[test]
fn pipeline_full_hp_cell_unchanged() {
    let mut app = test_app_pipeline();
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 100.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 100.0).abs() < f32::EPSILON,
        "full-HP cell should remain 100.0, got {}",
        hp.current
    );
    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        (timer.remaining - 10.0).abs() < 1e-5,
        "timer should reset to 10.0 even when no heal is emitted, got {}",
        timer.remaining
    );
}

// ── Behavior 29 — HealCap::Starting honoured with elevated hp.max ────────

#[test]
fn pipeline_heal_cap_is_starting_when_max_is_elevated() {
    // Volatility lifted max to 200.0. HealCap::Starting still clamps at
    // hp.starting = 100.0.
    let mut app = test_app_pipeline();
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell_with_max(&mut app, 30.0, 100.0, 200.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 100.0).abs() < f32::EPSILON,
        "HealCap::Starting must cap at 100.0 even with max = 200.0, got {}",
        hp.current
    );
}

#[test]
fn pipeline_heal_cap_starting_at_partial_damage_with_elevated_max() {
    // Edge: current=50, starting=100, max=200 — ends at 100.0.
    let mut app = test_app_pipeline();
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell_with_max(&mut app, 50.0, 100.0, 200.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 100.0).abs() < f32::EPSILON,
        "50 → 100 clamped at starting, got {}",
        hp.current
    );
}

// ── Behavior 30 — Over-HP cell is NOT reduced by Renewal's tick ─────────

#[test]
fn pipeline_over_hp_cell_is_not_reduced() {
    // hp.current > starting (e.g. prior Max-cap heal). Renewal must not
    // emit any message, so apply_heal's `>= ceiling` guard is not reached.
    let mut app = test_app_pipeline();
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell_with_max(&mut app, 120.0, 100.0, 150.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 120.0).abs() < f32::EPSILON,
        "over-HP cell must not be reduced; got {}",
        hp.current
    );
}

#[test]
fn pipeline_full_hp_cell_with_elevated_max_unchanged() {
    // Edge: current=100, starting=100, max=150 — unchanged, no message.
    let mut app = test_app_pipeline();
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell_with_max(&mut app, 100.0, 100.0, 150.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 100.0).abs() < f32::EPSILON,
        "full-HP cell with elevated max must remain 100.0; got {}",
        hp.current
    );
}

// ── Behavior 31 — Dead cell not revived by Renewal ──────────────────────

#[test]
fn pipeline_dead_cell_is_not_revived() {
    // Post-W2 semantics: a spawn-time HP=0 cell is picked up by the crate
    // kill pipeline (`detect_deaths` → `handle_kill` →
    // `process_despawn_requests` in `FixedPostUpdate`) and despawned in
    // the same tick. Renewal cannot revive a non-existent entity — that's
    // the invariant being pinned.
    let mut app = test_app_pipeline();
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 0.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    assert!(
        app.world().get_entity(cell).is_err(),
        "spawn-time HP=0 cell must be despawned by the crate pipeline; \
         Renewal cannot revive a despawned entity"
    );
}

#[test]
fn pipeline_dead_cell_with_dead_marker_is_not_revived() {
    // Edge: defence-in-depth — `apply_heal<Cell>`'s `Without<Dead>` filter
    // also keeps the heal from landing even if a message somehow got through.
    let mut app = test_app_pipeline();
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 0.0, 100.0);
    attach_timer(&mut app, cell, 0.05);
    app.world_mut().entity_mut(cell).insert(Dead);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        hp.current.abs() < f32::EPSILON,
        "cell with Dead marker must not revive; got {}",
        hp.current
    );
}

// ── Behavior 32 — Timer decrements across ticks; heal lands on expiry ────

#[test]
fn pipeline_heal_lands_only_on_expiry_tick_across_three_ticks() {
    let mut app = test_app_pipeline();
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 30.0, 100.0);
    attach_timer(&mut app, cell, 0.25);

    // Tick 1: dt=0.1 → remaining 0.15, HP unchanged.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));
    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 30.0).abs() < f32::EPSILON,
        "tick 1 HP should be 30.0, got {}",
        hp.current
    );

    // Tick 2: dt=0.1 → remaining 0.05, HP unchanged.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));
    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 30.0).abs() < f32::EPSILON,
        "tick 2 HP should be 30.0, got {}",
        hp.current
    );

    // Tick 3: dt=0.1 → expiry, HP heals to 100.0.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));
    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 100.0).abs() < f32::EPSILON,
        "tick 3 HP should be 100.0 (expiry heal), got {}",
        hp.current
    );
    // And timer reset to 10.0.
    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        (timer.remaining - 10.0).abs() < 1e-5,
        "timer should reset to 10.0 on expiry tick, got {}",
        timer.remaining
    );
}
