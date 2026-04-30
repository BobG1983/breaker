use super::super::{
    super::system::{OriginalBoltLossBehavior, RecklessDashConfig},
    helpers::{
        build_reckless_dash_transition_app, build_reckless_dash_transition_app_no_config,
        force_dash_state, install_reckless_dash_config, seed_active_protocols_with_reckless_dash,
        spawn_breaker_for_transition,
    },
};
use crate::{
    breaker::components::{BoltLossBehavior, DashState},
    prelude::*,
};

// ── Behavior 7 — double_penalty: false short-circuits mutation on dash enter ─

#[test]
fn double_penalty_false_suppresses_overlay_and_doubling_on_dash_enter() {
    let mut app = build_reckless_dash_transition_app();
    install_reckless_dash_config(
        &mut app,
        RecklessDashConfig {
            risky_zone_start:  0.7,
            damage_multiplier: 4.0,
            double_penalty:    false,
        },
    );
    seed_active_protocols_with_reckless_dash(&mut app, 0.7, 4.0, true);

    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        DashState::Idle,
        DashState::Idle,
    );

    force_dash_state(&mut app, breaker, DashState::Dashing);
    tick(&mut app);

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    assert_eq!(
        live,
        BoltLossBehavior::LifeLoss(1),
        "double_penalty:false must NOT double BoltLossBehavior"
    );
    assert!(
        app.world()
            .get::<OriginalBoltLossBehavior>(breaker)
            .is_none(),
        "double_penalty:false must NOT insert OriginalBoltLossBehavior"
    );
}

// ── Behavior 7 (edge case) — TimeLoss(5.0) also suppressed ─────────────────

#[test]
fn double_penalty_false_suppresses_timeloss_doubling_on_dash_enter() {
    let mut app = build_reckless_dash_transition_app();
    install_reckless_dash_config(
        &mut app,
        RecklessDashConfig {
            risky_zone_start:  0.7,
            damage_multiplier: 4.0,
            double_penalty:    false,
        },
    );
    seed_active_protocols_with_reckless_dash(&mut app, 0.7, 4.0, true);

    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::TimeLoss(5.0),
        DashState::Idle,
        DashState::Idle,
    );

    force_dash_state(&mut app, breaker, DashState::Dashing);
    tick(&mut app);

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    let BoltLossBehavior::TimeLoss(actual) = live else {
        panic!("expected TimeLoss variant, got {live:?}");
    };
    assert!(
        (actual - 5.0).abs() < f32::EPSILON,
        "double_penalty:false must leave TimeLoss(5.0) unchanged, got {actual}"
    );
    assert!(
        app.world()
            .get::<OriginalBoltLossBehavior>(breaker)
            .is_none(),
        "double_penalty:false must NOT insert OriginalBoltLossBehavior"
    );
}

// ── Behavior 9 — RecklessDashConfig absent → safe no-op ────────────────────

#[test]
fn absent_config_on_dash_enter_is_safe_noop_no_overlay_inserted() {
    let mut app = build_reckless_dash_transition_app_no_config();
    seed_active_protocols_with_reckless_dash(&mut app, 0.7, 4.0, true);

    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        DashState::Idle,
        DashState::Idle,
    );

    force_dash_state(&mut app, breaker, DashState::Dashing);
    tick(&mut app); // must not panic

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    assert_eq!(
        live,
        BoltLossBehavior::LifeLoss(1),
        "absent config must not mutate BoltLossBehavior"
    );
    assert!(
        app.world()
            .get::<OriginalBoltLossBehavior>(breaker)
            .is_none(),
        "absent config must not insert OriginalBoltLossBehavior"
    );
}
