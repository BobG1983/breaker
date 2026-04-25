//! Group F — System registration, scheduling, and gating.
//!
//! These use `register(&mut app)` to test the wiring, NOT hand-wired
//! systems. Includes 26A — `register`-wired zero-stack gate.

use std::time::Duration;

use rantzsoft_dmg::{RantzDmgAppExt, RantzDmgPlugin};

use super::{
    super::system::{RenewalTimer, register},
    helpers::{
        add_renewal_stacks, attach_timer, canonical_config, heal_collector_len, heals_for_cell,
        install_renewal_config, spawn_cell, test_app_playing, tick_with_dt,
    },
};
use crate::{
    cells::components::Cell,
    mutators::hazards::{definition::HazardKind, resources::ActiveHazards},
    prelude::*,
};

// ── Behavior 24 — register schedules tick in HandleKill → ApplyHeal window ─

#[test]
fn register_schedules_tick_system_that_emits_heal_on_expiry() {
    let mut app = test_app_playing();
    register(&mut app);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 30.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    assert_eq!(heal_collector_len(&app), 1);
    let msgs = heals_for_cell(&app, cell);
    assert_eq!(msgs.len(), 1);
    assert!((msgs[0].amount - 70.0).abs() < f32::EPSILON);
    assert!(matches!(msgs[0].cap, HealCap::Starting));
    assert_eq!(
        msgs[0].source,
        Some(SourceId::hazard(HazardKind::Renewal).build())
    );
}

#[test]
fn register_ordering_fires_renewal_before_apply_heal_in_one_tick() {
    // Edge: wire `apply_heal::<Cell>` via `register_dmgable::<Cell>`; the
    // registered `renewal_tick` runs before it, so HP reaches 100.0 in a
    // single tick.
    let mut app = test_app_playing();
    register(&mut app);
    app.add_plugins(RantzDmgPlugin);
    let _ = app.register_dmgable::<Cell>();
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 1);
    let cell = spawn_cell(&mut app, 30.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    assert_eq!(
        heal_collector_len(&app),
        1,
        "renewal_tick must emit exactly one HealDealt<Cell> message, not mutate Hp directly"
    );
    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 100.0).abs() < f32::EPSILON,
        "renewal_tick must run before apply_heal in one tick; got {}",
        hp.current
    );
}

// ── Behavior 25 — hazard_active(Renewal) false → system gated off ─────────

#[test]
fn system_skipped_when_renewal_run_condition_false() {
    let mut app = test_app_playing();
    register(&mut app);
    install_renewal_config(&mut app, canonical_config());
    // Stack a DIFFERENT hazard — Renewal remains inactive.
    for _ in 0..3 {
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Volatility);
    }
    assert_eq!(
        app.world()
            .resource::<ActiveHazards>()
            .stacks(HazardKind::Renewal),
        0
    );
    let cell = spawn_cell(&mut app, 30.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    assert_eq!(heal_collector_len(&app), 0);
    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        (timer.remaining - 0.05).abs() < 1e-5,
        "gated-off system must not tick timers; got {}",
        timer.remaining
    );
}

#[test]
fn system_runs_when_renewal_stack_added_after_initial_skip() {
    // Edge: with 0 Renewal stacks, no heal. Add 1 stack and tick again —
    // the timer expires and emits exactly 1 message.
    let mut app = test_app_playing();
    register(&mut app);
    install_renewal_config(&mut app, canonical_config());
    // No Renewal stacks yet; add a Volatility stack so `ActiveHazards` is
    // not totally empty (not required but mirrors a real run start).
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Volatility);
    let cell = spawn_cell(&mut app, 30.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));
    assert_eq!(heal_collector_len(&app), 0);

    // Add a Renewal stack and tick again.
    add_renewal_stacks(&mut app, 1);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    assert_eq!(heal_collector_len(&app), 1);
}

// ── Behavior 26 — NOT in NodeState::Playing → system gated off ────────────

#[test]
fn system_skipped_when_not_in_node_playing() {
    // Build app with state hierarchy but NOT driven into NodeState::Playing.
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ActiveHazards>()
        .with_message_capture::<HealDealt<Cell>>()
        .build();
    register(&mut app);
    install_renewal_config(&mut app, canonical_config());
    add_renewal_stacks(&mut app, 3);
    let cell = spawn_cell(&mut app, 30.0, 100.0);
    attach_timer(&mut app, cell, 0.05);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    assert_eq!(heal_collector_len(&app), 0);
    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        (timer.remaining - 0.05).abs() < 1e-5,
        "gated-off system must not tick timers; got {}",
        timer.remaining
    );
}

// ── Behavior 26A — register-wired zero-stack gate no-ops attach ──────────

#[test]
fn register_wired_zero_stacks_attach_does_not_insert_timer() {
    // The ENTIRE system set (including renewal_attach_timers) is gated by
    // hazard_active(Renewal). With 0 stacks, attach does not run.
    let mut app = test_app_playing();
    register(&mut app);
    install_renewal_config(&mut app, canonical_config());
    // 0 Renewal stacks — do NOT call add_renewal_stacks.
    let cell = spawn_cell(&mut app, 50.0, 100.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    assert!(
        app.world().get::<RenewalTimer>(cell).is_none(),
        "attach must not run when Renewal is inactive"
    );
}

#[test]
fn register_wired_attach_runs_once_stack_added() {
    // Edge: add 1 stack, tick again — timer appears.
    let mut app = test_app_playing();
    register(&mut app);
    install_renewal_config(&mut app, canonical_config());
    let cell = spawn_cell(&mut app, 50.0, 100.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));
    assert!(app.world().get::<RenewalTimer>(cell).is_none());

    add_renewal_stacks(&mut app, 1);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    // `.chain()` puts attach BEFORE tick in the same frame: attach inserts
    // `RenewalTimer { remaining: 10.0 }`, then tick decrements by dt=0.1
    // (no expiry since 10.0 > 0.0). Net observable value: 9.9.
    let timer = app.world().get::<RenewalTimer>(cell).unwrap();
    assert!(
        (timer.remaining - 9.9).abs() < 1e-5,
        "attach inserts stack-1 duration (10.0) and chained tick decrements \
         by dt=0.1 in the same frame → expect 9.9, got {}",
        timer.remaining
    );
}
