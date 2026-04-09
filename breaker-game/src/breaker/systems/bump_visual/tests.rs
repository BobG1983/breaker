use bevy::prelude::*;

use super::*;
use crate::{
    breaker::{
        components::{Breaker, BumpFeedback, BumpFeedbackState, BumpState},
        definition::BreakerDefinition,
        test_utils::default_breaker_definition,
    },
    input::resources::{GameAction, InputActions},
    shared::test_utils::TestAppBuilder,
};

fn default_bump_feedback() -> BumpFeedback {
    let config = default_breaker_definition();
    BumpFeedback {
        duration: config.bump_visual_duration,
        peak: config.bump_visual_peak,
        peak_fraction: config.bump_visual_peak_fraction,
        rise_ease: config.bump_visual_rise_ease,
        fall_ease: config.bump_visual_fall_ease,
    }
}

fn test_bump_offset(timer_fraction: f32) -> f32 {
    let params = default_bump_feedback();
    let visual = BumpFeedbackState {
        timer: params.duration * timer_fraction,
        duration: params.duration,
        peak_offset: params.peak,
    };
    bump_offset(&visual, &params)
}

#[test]
fn bump_offset_starts_at_zero() {
    assert!(test_bump_offset(1.0).abs() < f32::EPSILON);
}

#[test]
fn bump_offset_ends_at_zero() {
    assert!(test_bump_offset(0.0).abs() < 1e-5);
}

#[test]
fn bump_offset_positive_mid_animation() {
    assert!(
        test_bump_offset(0.5) > 0.0,
        "offset should be positive during animation"
    );
}

#[test]
fn bump_offset_at_peak_fraction_equals_peak() {
    let params = default_bump_feedback();
    let timer = params.duration * (1.0 - params.peak_fraction);
    let visual = BumpFeedbackState {
        timer,
        duration: params.duration,
        peak_offset: params.peak,
    };
    let offset = bump_offset(&visual, &params);
    assert!(
        (offset - params.peak).abs() < 0.01,
        "offset at peak_fraction should equal peak_offset, got {offset}"
    );
}

#[test]
fn bump_offset_asymmetric_shape() {
    let params = default_bump_feedback();
    let rise_mid = bump_offset(
        &BumpFeedbackState {
            timer: params.duration * (1.0 - 0.15),
            duration: params.duration,
            peak_offset: params.peak,
        },
        &params,
    );

    let fall_mid = bump_offset(
        &BumpFeedbackState {
            timer: params.duration * (1.0 - 0.65),
            duration: params.duration,
            peak_offset: params.peak,
        },
        &params,
    );

    // With CubicOut rise (fast start) and QuadraticIn fall (lingers near peak),
    // both should be well above 50% of peak at their respective midpoints.
    assert!(
        rise_mid > params.peak * 0.5,
        "CubicOut rise at 50% should be above 50% of peak, got {rise_mid}"
    );
    assert!(
        fall_mid > params.peak * 0.5,
        "QuadraticIn fall at 50% should still be above 50% of peak (lingering), got {fall_mid}"
    );
}

fn trigger_test_app() -> App {
    TestAppBuilder::new()
        .with_resource::<InputActions>()
        .with_system(FixedUpdate, trigger_bump_visual)
        .build()
}

use crate::shared::test_utils::tick;

fn set_bump_action(app: &mut App) {
    app.world_mut()
        .resource_mut::<InputActions>()
        .0
        .push(GameAction::Bump);
}

#[test]
fn trigger_inserts_bump_visual_on_bump_action() {
    let mut app = trigger_test_app();

    let def = BreakerDefinition::default();
    let entity = {
        let world = app.world_mut();
        let entity = Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(&mut world.commands());
        world.flush();
        entity
    };

    set_bump_action(&mut app);
    tick(&mut app);

    assert!(
        app.world().get::<BumpFeedbackState>(entity).is_some(),
        "BumpFeedbackState should be inserted when Bump action is active"
    );
}

#[test]
fn trigger_skips_without_bump_action() {
    let mut app = trigger_test_app();

    let def = BreakerDefinition::default();
    let entity = {
        let world = app.world_mut();
        let entity = Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(&mut world.commands());
        world.flush();
        entity
    };

    // No Bump action set
    tick(&mut app);

    assert!(
        app.world().get::<BumpFeedbackState>(entity).is_none(),
        "BumpFeedbackState should not be inserted without Bump action"
    );
}

#[test]
fn trigger_fires_during_cooldown() {
    let mut app = trigger_test_app();

    let def = BreakerDefinition::default();
    let entity = {
        let world = app.world_mut();
        let entity = Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(&mut world.commands());
        world.flush();
        entity
    };
    app.world_mut().entity_mut(entity).insert(BumpState {
        cooldown: 0.5,
        ..Default::default()
    });

    set_bump_action(&mut app);
    tick(&mut app);

    assert!(
        app.world().get::<BumpFeedbackState>(entity).is_some(),
        "BumpFeedbackState should fire even during cooldown"
    );
}

#[test]
fn trigger_does_not_retrigger_while_animating() {
    let mut app = trigger_test_app();
    let params = default_bump_feedback();

    let def = BreakerDefinition::default();
    let entity = {
        let world = app.world_mut();
        let entity = Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(&mut world.commands());
        world.flush();
        entity
    };
    app.world_mut()
        .entity_mut(entity)
        .insert(BumpFeedbackState {
            timer: 0.1,
            duration: params.duration,
            peak_offset: params.peak,
        });

    set_bump_action(&mut app);
    tick(&mut app);

    let visual = app
        .world()
        .get::<BumpFeedbackState>(entity)
        .expect("should still have BumpFeedbackState");
    assert!(
        (visual.timer - 0.1).abs() < f32::EPSILON,
        "timer should be unchanged — trigger should not overwrite existing animation"
    );
}

fn animate_test_app() -> App {
    TestAppBuilder::new()
        .with_system(FixedUpdate, animate_bump_visual)
        .build()
}

#[test]
fn animate_applies_position2d_y_offset_during_animation() {
    // Given: Breaker with active BumpFeedbackState, BreakerBaseY(-250.0),
    //        Position2D(Vec2::new(0.0, -250.0))
    // When: animate_bump_visual runs
    // Then: Position2D.0.y > -250.0 (popped up)
    use rantzsoft_spatial2d::components::Position2D;

    let mut app = animate_test_app();
    let config = BreakerDefinition::default();
    let params = default_bump_feedback();

    let def = BreakerDefinition::default();
    let entity = {
        let world = app.world_mut();
        let entity = Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(&mut world.commands());
        world.flush();
        entity
    };
    app.world_mut()
        .entity_mut(entity)
        .insert(BumpFeedbackState {
            timer: params.duration,
            duration: params.duration,
            peak_offset: params.peak,
        });

    tick(&mut app);

    let pos = app
        .world_mut()
        .query_filtered::<&Position2D, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");

    assert!(
        pos.0.y > config.y_position,
        "breaker should pop upward during animation, Position2D.y={} base={}",
        pos.0.y,
        config.y_position
    );
}

#[test]
fn animate_removes_bump_visual_when_done() {
    use rantzsoft_spatial2d::components::Position2D;

    let mut app = animate_test_app();
    let config = BreakerDefinition::default();
    let params = default_bump_feedback();

    let def = BreakerDefinition::default();
    let entity = {
        let world = app.world_mut();
        let entity = Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(&mut world.commands());
        world.flush();
        entity
    };
    app.world_mut()
        .entity_mut(entity)
        .insert(BumpFeedbackState {
            // Zero timer — will expire on next tick
            timer: 0.0,
            duration: params.duration,
            peak_offset: params.peak,
        });

    tick(&mut app);

    assert!(
        app.world().get::<BumpFeedbackState>(entity).is_none(),
        "BumpFeedbackState should be removed after animation completes"
    );

    let pos = app.world().get::<Position2D>(entity).expect("should exist");
    assert!(
        (pos.0.y - config.y_position).abs() < f32::EPSILON,
        "breaker should return to base y after animation, Position2D.y={} expected={}",
        pos.0.y,
        config.y_position
    );
}

#[test]
fn animate_snaps_position2d_to_base_after_expiry() {
    // Edge case: animation complete -> Position2D.0.y snaps to -250.0
    use rantzsoft_spatial2d::components::Position2D;

    let mut app = animate_test_app();
    let config = BreakerDefinition::default();
    let params = default_bump_feedback();

    // Start with an offset Y to verify the snap overrides it
    let def = BreakerDefinition::default();
    let entity = {
        let world = app.world_mut();
        let entity = Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(&mut world.commands());
        world.flush();
        entity
    };
    app.world_mut().entity_mut(entity).insert((
        Position2D(Vec2::new(0.0, config.y_position + 5.0)),
        BumpFeedbackState {
            // Near-expired timer — will complete within a few test updates
            timer: 0.0001,
            duration: params.duration,
            peak_offset: params.peak,
        },
    ));

    // A few ticks to let the timer expire and commands flush
    for _ in 0..5 {
        tick(&mut app);
    }

    let pos = app
        .world_mut()
        .query_filtered::<&Position2D, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");

    assert!(
        (pos.0.y - config.y_position).abs() < f32::EPSILON,
        "breaker should snap to base y after animation, Position2D.y={} base={}",
        pos.0.y,
        config.y_position
    );
}
