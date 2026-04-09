use bevy::prelude::*;

use crate::{
    bolt::messages::BoltImpactBreaker,
    breaker::{
        components::{
            BumpEarlyWindow, BumpLateWindow, BumpPerfectCooldown, BumpPerfectWindow,
            BumpWeakCooldown, SettleDuration,
        },
        definition::BreakerDefinition,
        messages::{BumpPerformed, BumpWhiffed},
        systems::bump::{grade_bump, update_bump},
    },
    input::resources::{GameAction, InputActions},
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
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<InputActions>()
        .add_message::<BumpPerformed>()
        .add_message::<BumpWhiffed>()
        .init_resource::<CapturedBumps>()
        .init_resource::<CapturedWhiffs>()
        .insert_resource(TestInputActive(false))
        .add_systems(
            FixedUpdate,
            (
                set_bump_action.before(update_bump),
                update_bump,
                (capture_bumps, capture_whiffs).after(update_bump),
            ),
        );
    app
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
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<BoltImpactBreaker>()
        .add_message::<BumpPerformed>()
        .add_message::<BumpWhiffed>()
        .init_resource::<CapturedBumps>()
        .insert_resource(TestHitMessage(None))
        .add_systems(
            FixedUpdate,
            (
                enqueue_hit.before(grade_bump),
                grade_bump,
                capture_bumps.after(grade_bump),
            ),
        );
    app
}

/// App that runs both `update_bump` and `grade_bump` with production ordering,
/// plus a hit injector and message captures.
pub(super) fn combined_bump_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<InputActions>()
        .add_message::<BoltImpactBreaker>()
        .add_message::<BumpPerformed>()
        .add_message::<BumpWhiffed>()
        .init_resource::<CapturedBumps>()
        .init_resource::<CapturedWhiffs>()
        .insert_resource(TestInputActive(false))
        .insert_resource(TestHitMessage(None))
        .add_systems(
            FixedUpdate,
            (
                set_bump_action.before(update_bump),
                enqueue_hit.before(grade_bump),
                update_bump,
                grade_bump.after(update_bump),
                (capture_bumps, capture_whiffs).after(grade_bump),
            ),
        );
    app
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
