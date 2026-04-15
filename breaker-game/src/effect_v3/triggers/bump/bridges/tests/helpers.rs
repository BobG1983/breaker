use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::super::system::*;
use crate::{
    breaker::messages::{BumpWhiffed, NoBump},
    effect_v3::{
        effects::SpeedBoostConfig,
        types::{BumpTarget, EffectType, ParticipantTarget, Terminal, Tree, Trigger},
    },
    prelude::*,
};

// -- Message injection resources -----------------------------------------

/// Resource to inject `NoBump` messages into the test app.
#[derive(Resource, Default)]
pub(super) struct TestNoBumpMessages(pub(super) Vec<NoBump>);

/// System that writes `NoBump` messages from the test resource.
pub(super) fn inject_no_bumps(
    messages: Res<TestNoBumpMessages>,
    mut writer: MessageWriter<NoBump>,
) {
    for msg in &messages.0 {
        writer.write(msg.clone());
    }
}

/// Resource to inject `BumpPerformed` messages into the test app.
#[derive(Resource, Default)]
pub(super) struct TestBumpPerformedMessages(pub(super) Vec<BumpPerformed>);

/// System that writes `BumpPerformed` messages from the test resource.
pub(super) fn inject_bump_performed(
    messages: Res<TestBumpPerformedMessages>,
    mut writer: MessageWriter<BumpPerformed>,
) {
    for msg in &messages.0 {
        writer.write(msg.clone());
    }
}

/// Resource to inject `BumpWhiffed` messages into the test app.
#[derive(Resource, Default)]
pub(super) struct TestBumpWhiffedMessages(pub(super) Vec<BumpWhiffed>);

/// System that writes `BumpWhiffed` messages from the test resource.
pub(super) fn inject_bump_whiffed(
    messages: Res<TestBumpWhiffedMessages>,
    mut writer: MessageWriter<BumpWhiffed>,
) {
    for msg in &messages.0 {
        writer.write(msg.clone());
    }
}

// -- App builders --------------------------------------------------------

pub(super) fn bridge_test_app() -> App {
    TestAppBuilder::new()
        .with_message::<NoBump>()
        .with_resource::<TestNoBumpMessages>()
        .with_system(
            FixedUpdate,
            (
                inject_no_bumps.before(on_no_bump_occurred),
                on_no_bump_occurred,
            ),
        )
        .build()
}

pub(super) fn bump_performed_test_app<M>(
    systems: impl IntoScheduleConfigs<bevy::ecs::system::ScheduleSystem, M>,
) -> App {
    TestAppBuilder::new()
        .with_message::<BumpPerformed>()
        .with_resource::<TestBumpPerformedMessages>()
        .with_system(FixedUpdate, systems)
        .build()
}

pub(super) fn bump_whiff_occurred_test_app() -> App {
    TestAppBuilder::new()
        .with_message::<BumpWhiffed>()
        .with_resource::<TestBumpWhiffedMessages>()
        .with_system(
            FixedUpdate,
            (
                inject_bump_whiffed.before(on_bump_whiff_occurred),
                on_bump_whiff_occurred,
            ),
        )
        .build()
}

pub(super) fn tick(app: &mut App) {
    crate::shared::test_utils::tick(app);
}

// -- Tree helpers --------------------------------------------------------

pub(super) fn no_bump_speed_tree(name: &str, multiplier: f32) -> (String, Tree) {
    (
        name.to_string(),
        Tree::When(
            Trigger::NoBumpOccurred,
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(multiplier),
            }))),
        ),
    )
}

pub(super) fn no_bump_on_target_tree(
    name: &str,
    target: BumpTarget,
    multiplier: f32,
) -> (String, Tree) {
    (
        name.to_string(),
        Tree::When(
            Trigger::NoBumpOccurred,
            Box::new(Tree::On(
                ParticipantTarget::Bump(target),
                Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(multiplier),
                })),
            )),
        ),
    )
}

pub(super) fn speed_tree(name: &str, trigger: Trigger, multiplier: f32) -> (String, Tree) {
    (
        name.to_string(),
        Tree::When(
            trigger,
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(multiplier),
            }))),
        ),
    )
}

pub(super) fn on_target_tree(
    name: &str,
    trigger: Trigger,
    target: BumpTarget,
    multiplier: f32,
) -> (String, Tree) {
    (
        name.to_string(),
        Tree::When(
            trigger,
            Box::new(Tree::On(
                ParticipantTarget::Bump(target),
                Terminal::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(multiplier),
                })),
            )),
        ),
    )
}
