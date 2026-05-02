use bevy::prelude::*;

use crate::{
    breaker::{
        builder::core::types::BreakerPhantomParams,
        components::{DashState, DashStateTimer, PhantomBreaker, SettleDuration},
        messages::{BumpGrade, BumpPerformed},
        systems::bump::perfect_bump_dash_cancel,
        test_utils::default_breaker_definition,
    },
    prelude::*,
};

// ── Inline enqueue helper (mirrors bump/tests/helpers.rs enqueue_bump pattern) ──

#[derive(Resource, Default)]
struct PendingBumpPerformed(Option<BumpPerformed>);

fn enqueue_bump(res: Res<PendingBumpPerformed>, mut writer: MessageWriter<BumpPerformed>) {
    if let Some(msg) = res.0.clone() {
        writer.write(msg);
    }
}

fn test_app() -> App {
    TestAppBuilder::new()
        .with_message::<BumpPerformed>()
        .insert_resource(PendingBumpPerformed(None))
        .with_system(
            FixedUpdate,
            (
                enqueue_bump.before(perfect_bump_dash_cancel),
                perfect_bump_dash_cancel,
            ),
        )
        .build()
}

// ── Wave 3 Behavior 5: Phantom-emitted Perfect does NOT cancel real dash ──

#[test]
fn phantom_perfect_does_not_cancel_real_dash() {
    let def = default_breaker_definition();
    let mut app = test_app();

    let real = {
        let world = app.world_mut();
        let e = Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(&mut world.commands());
        world.flush();
        e
    };
    let phantom = {
        let world = app.world_mut();
        let e = Breaker::builder()
            .definition(&def)
            .phantom(BreakerPhantomParams {
                lifespan:          2.5,
                phantom_color_rgb: [0.4, 0.8, 1.0],
                flicker_frequency: 4.0,
                flicker_min_alpha: 0.3,
            })
            .headless()
            .extra()
            .spawn(&mut world.commands());
        world.flush();
        e
    };

    assert!(
        app.world().get::<PhantomBreaker>(phantom).is_some(),
        "phantom entity must have PhantomBreaker marker"
    );

    // Put the real breaker into active Dashing state with 0.5 s remaining
    app.world_mut()
        .entity_mut(real)
        .insert((DashState::Dashing, DashStateTimer { remaining: 0.5 }));

    // Enqueue a Perfect BumpPerformed whose source is the PHANTOM
    app.world_mut().resource_mut::<PendingBumpPerformed>().0 = Some(BumpPerformed {
        grade:   BumpGrade::Perfect,
        bolt:    None,
        breaker: phantom,
    });

    tick(&mut app);

    // The real breaker's dash must NOT have been cancelled
    let real_state = app.world().get::<DashState>(real).unwrap();
    assert_eq!(
        *real_state,
        DashState::Dashing,
        "real breaker should still be Dashing after a phantom-emitted Perfect bump; \
         got {:?}",
        *real_state
    );

    let real_timer = app.world().get::<DashStateTimer>(real).unwrap();
    assert!(
        (real_timer.remaining - 0.5).abs() < f32::EPSILON,
        "real DashStateTimer.remaining must be unchanged (0.5) after phantom Perfect; \
         got {}",
        real_timer.remaining
    );

    // The phantom's own state is still Idle (no side effect from the inner query)
    let phantom_state = app.world().get::<DashState>(phantom).unwrap();
    assert_eq!(
        *phantom_state,
        DashState::Idle,
        "phantom DashState must remain Idle; got {:?}",
        *phantom_state
    );
}

// ── Wave 3 Behavior 5 sibling: real-emitted Perfect DOES cancel real dash ──

#[test]
fn real_perfect_still_cancels_real_dash() {
    let def = default_breaker_definition();
    let mut app = test_app();

    let real = {
        let world = app.world_mut();
        let e = Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(&mut world.commands());
        world.flush();
        e
    };

    let settle_duration = app
        .world()
        .get::<SettleDuration>(real)
        .expect("real breaker must have SettleDuration from builder")
        .0;

    // Put the real breaker into active Dashing
    app.world_mut()
        .entity_mut(real)
        .insert((DashState::Dashing, DashStateTimer { remaining: 0.5 }));

    // Enqueue a Perfect BumpPerformed whose source is the REAL breaker
    app.world_mut().resource_mut::<PendingBumpPerformed>().0 = Some(BumpPerformed {
        grade:   BumpGrade::Perfect,
        bolt:    None,
        breaker: real,
    });

    tick(&mut app);

    // The real breaker's dash must have been cancelled — transitioned to Settling
    let real_state = app.world().get::<DashState>(real).unwrap();
    assert_eq!(
        *real_state,
        DashState::Settling,
        "real breaker should transition to Settling after a real-emitted Perfect bump; \
         got {:?}",
        *real_state
    );

    let real_timer = app.world().get::<DashStateTimer>(real).unwrap();
    assert!(
        (real_timer.remaining - settle_duration).abs() < f32::EPSILON,
        "DashStateTimer.remaining should be set to SettleDuration ({}) after cancel; \
         got {}",
        settle_duration,
        real_timer.remaining
    );
}
