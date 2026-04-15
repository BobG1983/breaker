use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::super::system::*;
use crate::{
    breaker::messages::{BumpGrade, BumpPerformed},
    effect_v3::{
        effects::{ShockwaveConfig, SpeedBoostConfig, entropy_engine::components::EntropyCounter},
        types::EffectType,
    },
    shared::{rng::GameRng, test_utils::TestAppBuilder},
};

// -- Helpers ----------------------------------------------------------

/// Resource to inject `BumpPerformed` messages into the test app.
#[derive(Resource, Default)]
pub(super) struct TestBumpMessages(pub(super) Vec<BumpPerformed>);

/// System that writes `BumpPerformed` messages from the test resource.
pub(super) fn inject_bumps(
    messages: Res<TestBumpMessages>,
    mut writer: MessageWriter<BumpPerformed>,
) {
    for msg in &messages.0 {
        writer.write(msg.clone());
    }
}

pub(super) fn entropy_app() -> App {
    TestAppBuilder::new()
        .with_message::<BumpPerformed>()
        .with_resource::<TestBumpMessages>()
        .insert_resource(GameRng::from_seed(42))
        .with_system(
            FixedUpdate,
            (
                inject_bumps.before(tick_entropy_engine),
                tick_entropy_engine,
            ),
        )
        .build()
}

pub(super) fn make_shockwave_effect() -> (OrderedFloat<f32>, Box<EffectType>) {
    (
        OrderedFloat(1.0),
        Box::new(EffectType::Shockwave(ShockwaveConfig {
            base_range:      OrderedFloat(48.0),
            range_per_level: OrderedFloat(0.0),
            stacks:          1,
            speed:           OrderedFloat(150.0),
        })),
    )
}

pub(super) fn make_speed_boost_effect() -> (OrderedFloat<f32>, Box<EffectType>) {
    (
        OrderedFloat(1.0),
        Box::new(EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        })),
    )
}

pub(super) fn spawn_counter(
    app: &mut App,
    count: u32,
    max_effects: u32,
    pool: Vec<(OrderedFloat<f32>, Box<EffectType>)>,
) -> Entity {
    app.world_mut()
        .spawn(EntropyCounter {
            count,
            max_effects,
            pool,
        })
        .id()
}

pub(super) fn queue_bump(app: &mut App) {
    let breaker = app.world_mut().spawn_empty().id();
    app.world_mut()
        .resource_mut::<TestBumpMessages>()
        .0
        .push(BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: None,
            breaker,
        });
}

pub(super) fn reset_app() -> App {
    TestAppBuilder::new()
        .with_system(FixedUpdate, reset_entropy_counter)
        .build()
}
