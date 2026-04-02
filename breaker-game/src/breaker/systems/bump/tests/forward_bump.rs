use bevy::prelude::*;

use super::helpers::*;
use crate::{
    breaker::{
        components::{Breaker, BumpState},
        definition::BreakerDefinition,
        messages::{BumpPerformed, BumpWhiffed},
    },
    input::resources::InputActions,
};

#[test]
fn input_opens_forward_window() {
    let mut app = update_bump_test_app();
    let config = BreakerDefinition::default();

    let entity = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&config)
                .headless()
                .primary()
                .build(),
        )
        .id();

    app.insert_resource(TestInputActive(true));
    tick(&mut app);

    let bump = app.world().get::<BumpState>(entity).unwrap();
    assert!(bump.active);
    assert!(
        (bump.timer - (config.early_window + config.perfect_window)).abs() < 0.02,
        "timer should be near early_window + perfect_window, got {}",
        bump.timer
    );
}

#[test]
fn input_on_cooldown_ignored() {
    let mut app = update_bump_test_app();
    let config = BreakerDefinition::default();

    let entity = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&config)
                .headless()
                .primary()
                .build(),
        )
        .id();
    app.world_mut().entity_mut(entity).insert(BumpState {
        cooldown: 0.5,
        ..Default::default()
    });

    app.insert_resource(TestInputActive(true));
    tick(&mut app);

    let bump = app.world().get::<BumpState>(entity).unwrap();
    assert!(!bump.active, "bump should not activate while on cooldown");
}

#[test]
fn input_while_active_ignored() {
    let mut app = update_bump_test_app();
    let config = BreakerDefinition::default();

    let entity = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&config)
                .headless()
                .primary()
                .build(),
        )
        .id();
    app.world_mut().entity_mut(entity).insert(BumpState {
        active: true,
        timer: config.early_window, // mid-window
        ..Default::default()
    });

    let timer_before = config.early_window;
    app.insert_resource(TestInputActive(true));
    tick(&mut app);

    let bump = app.world().get::<BumpState>(entity).unwrap();
    assert!(bump.active, "should still be active");
    // Timer should have ticked down, not been reset
    assert!(
        bump.timer < timer_before,
        "timer should tick down, not reset"
    );
}

#[test]
fn forward_window_expiry_sends_whiff_and_sets_cooldown() {
    let mut app = combined_bump_test_app();
    let config = BreakerDefinition::default();

    let entity = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&config)
                .headless()
                .primary()
                .build(),
        )
        .id();
    app.world_mut().entity_mut(entity).insert(BumpState {
        active: true,
        timer: 0.001, // about to expire
        ..Default::default()
    });

    app.insert_resource(TestInputActive(false));
    tick(&mut app);

    let bump = app.world().get::<BumpState>(entity).unwrap();
    assert!(!bump.active, "should have expired");
    assert!(
        (bump.cooldown - config.weak_bump_cooldown).abs() < f32::EPSILON,
        "whiff should set weak cooldown, got {}",
        bump.cooldown
    );

    let captured = app.world().resource::<CapturedBumps>();
    assert!(captured.0.is_empty(), "no BumpPerformed on whiff");

    let whiffs = app.world().resource::<CapturedWhiffs>();
    assert_eq!(whiffs.0, 1, "should send one BumpWhiffed message");
}

#[test]
fn post_hit_timer_ticks_down() {
    let mut app = update_bump_test_app();
    let config = BreakerDefinition::default();

    let entity = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&config)
                .headless()
                .primary()
                .build(),
        )
        .id();
    app.world_mut().entity_mut(entity).insert(BumpState {
        post_hit_timer: 0.1,
        ..Default::default()
    });

    app.insert_resource(TestInputActive(false));
    tick(&mut app);

    let bump = app.world().get::<BumpState>(entity).unwrap();
    assert!(bump.post_hit_timer < 0.1, "post_hit_timer should tick down");
}

#[test]
fn cooldown_ticks_down() {
    let mut app = update_bump_test_app();
    let config = BreakerDefinition::default();

    let entity = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&config)
                .headless()
                .primary()
                .build(),
        )
        .id();
    app.world_mut().entity_mut(entity).insert(BumpState {
        cooldown: 0.1,
        ..Default::default()
    });

    app.insert_resource(TestInputActive(false));
    tick(&mut app);

    let bump = app.world().get::<BumpState>(entity).unwrap();
    assert!(bump.cooldown < 0.1, "cooldown should tick down");
}

// ── BoltServing guard tests ────────────────────────────────────

#[test]
fn bump_while_serving_does_not_open_forward_window() {
    use crate::bolt::components::BoltServing;

    let mut app = update_bump_test_app();
    let config = BreakerDefinition::default();

    let entity = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&config)
                .headless()
                .primary()
                .build(),
        )
        .id();

    // Spawn a serving bolt
    app.world_mut().spawn(BoltServing);

    app.insert_resource(TestInputActive(true));
    tick(&mut app);

    let bump = app.world().get::<BumpState>(entity).unwrap();
    assert!(
        !bump.active,
        "forward window should not open while bolt is serving"
    );
}

#[test]
fn bump_without_serving_bolt_opens_forward_window() {
    // Regression guard: normal bump still works when no BoltServing exists
    let mut app = update_bump_test_app();
    let config = BreakerDefinition::default();

    let entity = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&config)
                .headless()
                .primary()
                .build(),
        )
        .id();

    // No BoltServing entity
    app.insert_resource(TestInputActive(true));
    tick(&mut app);

    let bump = app.world().get::<BumpState>(entity).unwrap();
    assert!(
        bump.active,
        "forward window should open when no bolt is serving"
    );
}

// ── FixedUpdate input loss test ─────────────────────────────────

/// App that mirrors production scheduling: input in `PreUpdate`, bump in `FixedUpdate`.
fn fixed_schedule_bump_app() -> App {
    use crate::input::systems::clear_input_actions;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<InputActions>()
        .add_message::<BumpPerformed>()
        .add_message::<BumpWhiffed>()
        .add_message::<crate::bolt::messages::BoltImpactBreaker>()
        .init_resource::<CapturedBumps>()
        .init_resource::<CapturedWhiffs>()
        .insert_resource(TestInputActive(false));

    // PreUpdate: populate InputActions (like read_input_actions)
    app.add_systems(PreUpdate, set_bump_action);

    // FixedPostUpdate: clear after FixedUpdate consumes actions
    app.add_systems(FixedPostUpdate, clear_input_actions);

    // FixedUpdate: process bumps (production schedule)
    app.add_systems(FixedUpdate, super::super::update_bump);

    // Update: capture results
    app.add_systems(Update, (capture_bumps, capture_whiffs));

    app
}

#[test]
fn bump_not_lost_when_fixed_update_skips_frame() {
    let mut app = fixed_schedule_bump_app();
    let config = BreakerDefinition::default();

    let entity = app
        .world_mut()
        .spawn(
            Breaker::builder()
                .definition(&config)
                .headless()
                .primary()
                .build(),
        )
        .id();

    // Frame 1: bump input active, but FixedUpdate won't run (no overstep).
    app.insert_resource(TestInputActive(true));
    tick(&mut app);

    // Frame 2: input no longer active, accumulate overstep so FixedUpdate runs.
    app.insert_resource(TestInputActive(false));
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    tick(&mut app);

    let bump = app.world().get::<BumpState>(entity).unwrap();
    assert!(
        bump.active,
        "bump input should not be lost when FixedUpdate skips a frame"
    );
}
