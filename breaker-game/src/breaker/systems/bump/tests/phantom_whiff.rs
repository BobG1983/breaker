//! Phantom whiff upstream gate tests (Wave 4 Standard Tier F1, behavior #10).
//!
//! `grade_bump` must NOT emit `BumpWhiffed` when a phantom entity's active
//! window expires. Today no gate exists so the phantom emits a whiff (RED).
//! The regression guard (real breaker whiffs) passes today.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    breaker::{
        components::{
            BumpEarlyWindow, BumpLateWindow, BumpPerfectCooldown, BumpPerfectWindow, BumpState,
            BumpWeakCooldown, PhantomBreaker,
        },
        definition::BreakerDefinition,
        messages::BumpWhiffed,
        systems::bump::grade_bump,
    },
    prelude::*,
};

fn phantom_whiff_test_app() -> App {
    TestAppBuilder::new()
        .with_message::<BoltImpactBreaker>()
        .with_message::<BumpPerformed>()
        .with_message::<BumpWhiffed>()
        .with_resource::<CapturedBumps>()
        .with_resource::<CapturedWhiffs>()
        .insert_resource(TestHitMessage(None))
        .with_system(
            FixedUpdate,
            (
                enqueue_hit.before(grade_bump),
                grade_bump,
                capture_bumps.after(grade_bump),
                capture_whiffs.after(grade_bump),
            ),
        )
        .build()
}

fn bump_window_components(
    def: &BreakerDefinition,
) -> (
    BumpPerfectWindow,
    BumpEarlyWindow,
    BumpLateWindow,
    BumpPerfectCooldown,
    BumpWeakCooldown,
) {
    (
        BumpPerfectWindow(def.perfect_window),
        BumpEarlyWindow(def.early_window),
        BumpLateWindow(def.late_window),
        BumpPerfectCooldown(def.perfect_bump_cooldown),
        BumpWeakCooldown(def.weak_bump_cooldown),
    )
}

/// Behavior #10 phantom assertion: phantom window expiry does NOT emit `BumpWhiffed`.
#[test]
fn grade_bump_does_not_whiff_on_phantom_window_expiry() {
    let mut app = phantom_whiff_test_app();
    let def = BreakerDefinition::default();

    let phantom_entity = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);
    app.world_mut()
        .entity_mut(phantom_entity)
        .insert(PhantomBreaker)
        .insert(BumpState {
            active: true,
            timer: 0.0,
            ..Default::default()
        })
        .insert(bump_window_components(&def));

    // No BoltImpactBreaker injected — only the expiry branch runs.
    tick(&mut app);

    let whiff_count = app.world().resource::<CapturedWhiffs>().0;
    assert_eq!(
        whiff_count, 0,
        "phantom window expiry must NOT emit BumpWhiffed (got {whiff_count}); today grade_bump has no phantom filter (RED)"
    );
}

/// Behavior #10 regression guard: real breaker window expiry still emits `BumpWhiffed`.
#[test]
fn grade_bump_still_whiffs_on_real_window_expiry() {
    let mut app = phantom_whiff_test_app();
    let def = BreakerDefinition::default();

    let real_entity = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);
    app.world_mut()
        .entity_mut(real_entity)
        .insert(BumpState {
            active: true,
            timer: 0.0,
            ..Default::default()
        })
        .insert(bump_window_components(&def));

    tick(&mut app);

    let whiff_count = app.world().resource::<CapturedWhiffs>().0;
    assert_eq!(
        whiff_count, 1,
        "real breaker window expiry should emit exactly one BumpWhiffed (got {whiff_count})"
    );
}

/// Edge case: one phantom + one real both with expired windows — only one whiff fires.
#[test]
fn grade_bump_mixed_entities_only_real_whiffs() {
    let mut app = phantom_whiff_test_app();
    let def = BreakerDefinition::default();

    let phantom_entity = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);
    app.world_mut()
        .entity_mut(phantom_entity)
        .insert(PhantomBreaker)
        .insert(BumpState {
            active: true,
            timer: 0.0,
            ..Default::default()
        })
        .insert(bump_window_components(&def));

    let real_entity = crate::breaker::test_utils::spawn_breaker(&mut app, 50.0, 0.0);
    app.world_mut()
        .entity_mut(real_entity)
        .insert(BumpState {
            active: true,
            timer: 0.0,
            ..Default::default()
        })
        .insert(bump_window_components(&def));

    tick(&mut app);

    let whiff_count = app.world().resource::<CapturedWhiffs>().0;
    assert_eq!(
        whiff_count, 1,
        "with one phantom + one real both expiring, only 1 whiff should fire (got {whiff_count})"
    );
}
