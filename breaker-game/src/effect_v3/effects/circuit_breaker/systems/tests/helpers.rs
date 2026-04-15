use bevy::prelude::*;

use super::super::system::*;
use crate::{
    breaker::messages::BumpGrade,
    effect_v3::effects::circuit_breaker::components::CircuitBreakerCounter, prelude::*,
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

pub(super) fn circuit_breaker_app() -> App {
    TestAppBuilder::new()
        .with_message::<BumpPerformed>()
        .with_resource::<TestBumpMessages>()
        .with_resource::<GameRng>()
        .with_system(
            FixedUpdate,
            (
                inject_bumps.before(tick_circuit_breaker),
                tick_circuit_breaker,
            ),
        )
        .build()
}

pub(super) fn spawn_counter(app: &mut App, remaining: u32, bumps_required: u32) -> Entity {
    app.world_mut()
        .spawn(CircuitBreakerCounter {
            remaining,
            bumps_required,
            spawn_count: 2,
            inherit: false,
            shockwave_range: 64.0,
            shockwave_speed: 200.0,
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
