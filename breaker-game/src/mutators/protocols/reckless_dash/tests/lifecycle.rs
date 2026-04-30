//! Integration — `OriginalBoltLossBehavior` `LifeLoss` round-trip across
//! three ticks. Each phase is tested in isolation elsewhere; this test
//! composes them so a regression in any tick-to-tick handoff is caught here.
//! Only `LifeLoss` is composed end-to-end here; `TimeLoss` is covered in
//! isolation by `double_penalty.rs`.

use super::helpers::{
    build_reckless_dash_e2e_app, force_dash_state, original_behavior,
    seed_active_protocols_with_reckless_dash, spawn_breaker_for_e2e, write_bolt_lost,
};
use crate::{
    breaker::components::{BoltLossBehavior, DashState},
    prelude::*,
};

#[test]
fn full_lifecycle_life_loss_round_trip() {
    let mut app = build_reckless_dash_e2e_app();
    seed_active_protocols_with_reckless_dash(&mut app, 0.7, 4.0, true);

    let breaker = spawn_breaker_for_e2e(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        Some(Hp {
            current:  3.0,
            starting: 3.0,
            max:      None,
        }),
        DashState::Idle,
        DashState::Idle,
    );
    let bolt = app.world_mut().spawn_empty().id();

    // ── Phase 1: Enter dash ──────────────────────────────────────────────────

    force_dash_state(&mut app, breaker, DashState::Dashing);
    tick(&mut app);

    let live_after_enter = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior after dash enter");
    assert_eq!(
        live_after_enter,
        BoltLossBehavior::LifeLoss(2),
        "BoltLossBehavior must double to LifeLoss(2) after dash enter, got {live_after_enter:?}"
    );

    // Overlay must carry the PRE-doubled original — if it stores LifeLoss(2),
    // Phase 3 restore would write LifeLoss(2) instead of LifeLoss(1).
    assert_eq!(
        original_behavior(&app, breaker),
        Some(BoltLossBehavior::LifeLoss(1)),
        "OriginalBoltLossBehavior must store the pre-doubled LifeLoss(1), \
         not the doubled LifeLoss(2) — overlay must reflect the original value"
    );

    let hp_current_after_enter = app
        .world()
        .get::<Hp>(breaker)
        .expect("breaker must have Hp after dash enter")
        .current;
    assert!(
        (hp_current_after_enter - 3.0_f32).abs() < f32::EPSILON,
        "dash enter must NOT reduce Hp.current — no BoltLost was written this tick, \
         got {hp_current_after_enter}"
    );

    // ── Phase 2: BoltLost while dashing ─────────────────────────────────────

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    let hp_current_after_bolt_lost = app
        .world()
        .get::<Hp>(breaker)
        .expect("breaker must have Hp after BoltLost")
        .current;
    // 1.0 = 3.0 − 2.0; if 2.0 the original LifeLoss(1) was used instead of doubled.
    assert!(
        (hp_current_after_bolt_lost - 1.0_f32).abs() < f32::EPSILON,
        "Hp.current must be 1.0 after BoltLost with doubled LifeLoss(2) \
         (3.0 − 2.0 = 1.0) — if 2.0, the original LifeLoss(1) was incorrectly used, \
         got {hp_current_after_bolt_lost}"
    );

    let live_after_bolt_lost = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior after BoltLost");
    assert_eq!(
        live_after_bolt_lost,
        BoltLossBehavior::LifeLoss(2),
        "live BoltLossBehavior must remain LifeLoss(2) while still dashing — \
         handle_bolt_lost must not restore the original; got {live_after_bolt_lost:?}"
    );
    assert_eq!(
        original_behavior(&app, breaker),
        Some(BoltLossBehavior::LifeLoss(1)),
        "OriginalBoltLossBehavior must still be present after mid-dash BoltLost — \
         the overlay must persist until dash exit"
    );

    // ── Phase 3: Exit dash ───────────────────────────────────────────────────

    force_dash_state(&mut app, breaker, DashState::Idle);
    tick(&mut app);

    let live_after_exit = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior after dash exit");
    assert_eq!(
        live_after_exit,
        BoltLossBehavior::LifeLoss(1),
        "BoltLossBehavior must be restored to LifeLoss(1) after dash exit — \
         got {live_after_exit:?}"
    );

    // Asserting only the live value is insufficient: an orphan overlay would still
    // restore the value but leak the component into the next node's enter transition.
    assert_eq!(
        original_behavior(&app, breaker),
        None,
        "OriginalBoltLossBehavior must be absent after dash exit — \
         overlay orphan would leak LifeLoss(1) into the next node's enter transition"
    );

    let hp_current_after_exit = app
        .world()
        .get::<Hp>(breaker)
        .expect("breaker must have Hp after dash exit")
        .current;
    assert!(
        (hp_current_after_exit - 1.0_f32).abs() < f32::EPSILON,
        "Hp.current must remain 1.0 after dash exit — BoltLossBehavior restoration \
         must not itself apply or undo a penalty, got {hp_current_after_exit}"
    );
}

#[test]
fn post_exit_bolt_lost_applies_restored_behavior() {
    // Arrange: breaker with LifeLoss(1) and Hp=3.0 in Idle state.
    // Starting from 3.0 makes LifeLoss(1) vs LifeLoss(2) distinguishable:
    //   restored LifeLoss(1): 3.0 − 1.0 = 2.0
    //   stale doubled LifeLoss(2): 3.0 − 2.0 = 1.0
    let mut app = build_reckless_dash_e2e_app();
    seed_active_protocols_with_reckless_dash(&mut app, 0.7, 4.0, true);

    let breaker = spawn_breaker_for_e2e(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        Some(Hp {
            current:  3.0,
            starting: 3.0,
            max:      None,
        }),
        DashState::Idle,
        DashState::Idle,
    );
    let bolt = app.world_mut().spawn_empty().id();

    // Phase 1: Enter dash — behavior doubles to LifeLoss(2).
    force_dash_state(&mut app, breaker, DashState::Dashing);
    tick(&mut app);

    // Phase 2: Exit dash — behavior restored to LifeLoss(1).
    force_dash_state(&mut app, breaker, DashState::Idle);
    tick(&mut app);

    // Phase 3: BoltLost fires after the overlay is gone.
    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    let hp_current = app
        .world()
        .get::<Hp>(breaker)
        .expect("breaker must have Hp after post-exit BoltLost")
        .current;
    // Restored LifeLoss(1): 3.0 − 1.0 = 2.0.
    // Stale doubled LifeLoss(2) would yield 3.0 − 2.0 = 1.0 instead.
    assert!(
        (hp_current - 2.0_f32).abs() < f32::EPSILON,
        "Hp.current must be 2.0 after post-exit BoltLost with restored LifeLoss(1) \
         (3.0 − 1.0 = 2.0) — if 1.0, stale doubled LifeLoss(2) was used, \
         got {hp_current}"
    );

    let live_behavior = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior after post-exit BoltLost");
    assert_eq!(
        live_behavior,
        BoltLossBehavior::LifeLoss(1),
        "BoltLossBehavior must remain LifeLoss(1) after post-exit BoltLost — \
         handle_bolt_lost must not re-double; got {live_behavior:?}"
    );

    assert_eq!(
        original_behavior(&app, breaker),
        None,
        "OriginalBoltLossBehavior must remain absent after post-exit BoltLost — \
         handle_bolt_lost must not create a new overlay"
    );
}
