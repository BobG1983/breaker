use bevy::prelude::*;

use crate::{
    bolt::messages::BoltImpactBreaker,
    breaker::{
        components::{
            BumpEarlyWindow, BumpLateWindow, BumpPerfectCooldown, BumpPerfectWindow,
            BumpWeakCooldown, SettleDuration,
        },
        definition::BreakerDefinition,
        messages::{BumpPerformed, BumpWhiffed, NoBump},
        systems::bump::{grade_bump, update_bump},
    },
    input::resources::{GameAction, InputActions},
    shared::test_utils::TestAppBuilder,
};

#[derive(Resource)]
pub(super) struct TestInputActive(pub bool);

pub(super) fn set_bump_action(mut actions: ResMut<InputActions>, active: Res<TestInputActive>) {
    if active.0 {
        actions.0.push(GameAction::Bump);
    }
}

#[derive(Resource, Default)]
pub(super) struct CapturedBumps(pub Vec<BumpPerformed>);

#[derive(Resource, Default)]
pub(super) struct CapturedWhiffs(pub u32);

pub(super) fn capture_bumps(
    mut reader: MessageReader<BumpPerformed>,
    mut captured: ResMut<CapturedBumps>,
) {
    for msg in reader.read() {
        captured.0.push(msg.clone());
    }
}

pub(super) fn capture_whiffs(
    mut reader: MessageReader<BumpWhiffed>,
    mut captured: ResMut<CapturedWhiffs>,
) {
    for _msg in reader.read() {
        captured.0 += 1;
    }
}

#[derive(Resource, Default)]
pub(super) struct CapturedNoBumps(pub Vec<NoBump>);

pub(super) fn capture_no_bumps(
    mut reader: MessageReader<NoBump>,
    mut captured: ResMut<CapturedNoBumps>,
) {
    for msg in reader.read() {
        captured.0.push(msg.clone());
    }
}

pub(super) fn bump_param_bundle(
    def: &BreakerDefinition,
) -> (
    BumpPerfectWindow,
    BumpEarlyWindow,
    BumpLateWindow,
    BumpPerfectCooldown,
    BumpWeakCooldown,
    SettleDuration,
) {
    (
        BumpPerfectWindow(def.perfect_window),
        BumpEarlyWindow(def.early_window),
        BumpLateWindow(def.late_window),
        BumpPerfectCooldown(def.perfect_bump_cooldown),
        BumpWeakCooldown(def.weak_bump_cooldown),
        SettleDuration(def.settle_duration),
    )
}

pub(super) fn update_bump_test_app() -> App {
    TestAppBuilder::new()
        .with_resource::<InputActions>()
        .with_message::<BumpPerformed>()
        .with_message::<BumpWhiffed>()
        .with_message::<NoBump>()
        .with_resource::<CapturedBumps>()
        .with_resource::<CapturedWhiffs>()
        .insert_resource(TestInputActive(false))
        .with_system(
            FixedUpdate,
            (
                set_bump_action.before(update_bump),
                update_bump,
                (capture_bumps, capture_whiffs).after(update_bump),
            ),
        )
        .build()
}

/// Like `update_bump_test_app` but also registers `NoBump` message capture.
pub(super) fn update_bump_with_no_bump_test_app() -> App {
    TestAppBuilder::new()
        .with_resource::<InputActions>()
        .with_message::<BumpPerformed>()
        .with_message::<BumpWhiffed>()
        .with_message::<NoBump>()
        .with_resource::<CapturedBumps>()
        .with_resource::<CapturedWhiffs>()
        .with_resource::<CapturedNoBumps>()
        .insert_resource(TestInputActive(false))
        .with_system(
            FixedUpdate,
            (
                set_bump_action.before(update_bump),
                update_bump,
                (capture_bumps, capture_whiffs, capture_no_bumps).after(update_bump),
            ),
        )
        .build()
}

pub(super) use crate::shared::test_utils::tick;

#[derive(Resource)]
pub(super) struct TestHitMessage(pub Option<BoltImpactBreaker>);

pub(super) fn enqueue_hit(
    msg_res: Res<TestHitMessage>,
    mut writer: MessageWriter<BoltImpactBreaker>,
) {
    if let Some(msg) = msg_res.0.clone() {
        writer.write(msg);
    }
}

pub(super) fn grade_bump_test_app() -> App {
    TestAppBuilder::new()
        .with_message::<BoltImpactBreaker>()
        .with_message::<BumpPerformed>()
        .with_message::<BumpWhiffed>()
        .with_resource::<CapturedBumps>()
        .insert_resource(TestHitMessage(None))
        .with_system(
            FixedUpdate,
            (
                enqueue_hit.before(grade_bump),
                grade_bump,
                capture_bumps.after(grade_bump),
            ),
        )
        .build()
}

/// App that runs both `update_bump` and `grade_bump` with production ordering,
/// plus a hit injector and message captures.
pub(super) fn combined_bump_test_app() -> App {
    TestAppBuilder::new()
        .with_resource::<InputActions>()
        .with_message::<BoltImpactBreaker>()
        .with_message::<BumpPerformed>()
        .with_message::<BumpWhiffed>()
        .with_message::<NoBump>()
        .with_resource::<CapturedBumps>()
        .with_resource::<CapturedWhiffs>()
        .insert_resource(TestInputActive(false))
        .insert_resource(TestHitMessage(None))
        .with_system(
            FixedUpdate,
            (
                set_bump_action.before(update_bump),
                enqueue_hit.before(grade_bump),
                update_bump,
                grade_bump.after(update_bump),
                (capture_bumps, capture_whiffs).after(grade_bump),
            ),
        )
        .build()
}

#[derive(Resource)]
pub(super) struct TestBumpMessage(pub Option<BumpPerformed>);

pub(super) fn enqueue_bump(
    msg_res: Res<TestBumpMessage>,
    mut writer: MessageWriter<BumpPerformed>,
) {
    if let Some(msg) = msg_res.0.clone() {
        writer.write(msg);
    }
}
