use bevy::prelude::*;

use super::system::*;
use crate::{
    bolt::{
        components::Bolt,
        messages::{BoltHitBreaker, BoltHitCell},
    },
    breaker::messages::BumpPerformed,
};

// --- Test infrastructure ---

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_observer(handle_ramping_damage);
    app
}

fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

// --- Helper: enqueue messages via resource ---

#[derive(Resource, Default)]
struct TestBoltHitCell(Vec<BoltHitCell>);

fn enqueue_bolt_hit_cell(msg_res: Res<TestBoltHitCell>, mut writer: MessageWriter<BoltHitCell>) {
    for msg in &msg_res.0 {
        writer.write(msg.clone());
    }
}

#[derive(Resource, Default)]
struct TestBoltHitBreaker(Vec<BoltHitBreaker>);

fn enqueue_bolt_hit_breaker(
    msg_res: Res<TestBoltHitBreaker>,
    mut writer: MessageWriter<BoltHitBreaker>,
) {
    for msg in &msg_res.0 {
        writer.write(msg.clone());
    }
}

#[derive(Resource, Default)]
struct TestBumpPerformed(Vec<BumpPerformed>);

fn enqueue_bump_performed(
    msg_res: Res<TestBumpPerformed>,
    mut writer: MessageWriter<BumpPerformed>,
) {
    for msg in &msg_res.0 {
        writer.write(msg.clone());
    }
}

fn test_app_message() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_observer(handle_ramping_damage)
        .add_message::<BoltHitCell>()
        .add_message::<BoltHitBreaker>()
        .add_message::<BumpPerformed>()
        .init_resource::<TestBoltHitCell>()
        .init_resource::<TestBoltHitBreaker>()
        .init_resource::<TestBumpPerformed>()
        .add_systems(
            FixedUpdate,
            (
                enqueue_bolt_hit_cell.before(increment_ramping_damage),
                enqueue_bolt_hit_breaker.before(reset_ramping_damage),
                enqueue_bump_performed.before(reset_ramping_damage),
                increment_ramping_damage,
                reset_ramping_damage,
            ),
        );
    app
}

// =========================================================================
// Behavior 7: handle_ramping_damage inserts RampingDamageState on bolt
// =========================================================================

#[test]
fn handle_ramping_damage_inserts_state_on_bolt() {
    let mut app = test_app();
    let bolt = app.world_mut().spawn(Bolt).id();

    app.world_mut().commands().trigger(RampingDamageApplied {
        bonus_per_hit: 0.02,
        max_stacks: 2,
        chip_name: "Basic Amp".to_owned(),
    });
    app.world_mut().flush();

    let state = app
        .world()
        .get::<RampingDamageState>(bolt)
        .expect("bolt should have RampingDamageState after RampingDamageApplied");
    assert!(
        (state.current_bonus - 0.0).abs() < f32::EPSILON,
        "current_bonus should be 0.0, got {}",
        state.current_bonus
    );
    assert!(
        (state.bonus_per_hit - 0.02).abs() < f32::EPSILON,
        "bonus_per_hit should be 0.02, got {}",
        state.bonus_per_hit
    );
}

// =========================================================================
// Behavior 8: handle_ramping_damage stacks additively
// =========================================================================

#[test]
fn handle_ramping_damage_stacks_additively() {
    let mut app = test_app();
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            RampingDamageState {
                current_bonus: 0.0,
                bonus_per_hit: 0.02,
            },
        ))
        .id();

    app.world_mut().commands().trigger(RampingDamageApplied {
        bonus_per_hit: 0.04,
        max_stacks: 2,
        chip_name: "Potent Amp".to_owned(),
    });
    app.world_mut().flush();

    let state = app
        .world()
        .get::<RampingDamageState>(bolt)
        .expect("bolt should still have RampingDamageState");
    assert!(
        (state.bonus_per_hit - 0.06).abs() < 1e-6,
        "bonus_per_hit should be 0.06 (0.02 + 0.04), got {}",
        state.bonus_per_hit
    );
}

// =========================================================================
// Behavior 9: increment_ramping_damage on cell hit
// =========================================================================

#[test]
fn increment_ramping_damage_on_cell_hit() {
    let mut app = test_app_message();
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            RampingDamageState {
                current_bonus: 0.0,
                bonus_per_hit: 0.04,
            },
        ))
        .id();
    let cell = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<TestBoltHitCell>()
        .0
        .push(BoltHitCell { cell, bolt });

    tick(&mut app);

    let state = app
        .world()
        .get::<RampingDamageState>(bolt)
        .expect("bolt should have RampingDamageState");
    assert!(
        (state.current_bonus - 0.04).abs() < f32::EPSILON,
        "current_bonus should be 0.04 after one cell hit, got {}",
        state.current_bonus
    );
}

// =========================================================================
// Behavior 10: increment_ramping_damage grows without cap
// =========================================================================

#[test]
fn increment_ramping_damage_grows_without_cap() {
    let mut app = test_app_message();
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            RampingDamageState {
                current_bonus: 0.38,
                bonus_per_hit: 0.04,
            },
        ))
        .id();
    let cell = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<TestBoltHitCell>()
        .0
        .push(BoltHitCell { cell, bolt });

    tick(&mut app);

    let state = app
        .world()
        .get::<RampingDamageState>(bolt)
        .expect("bolt should have RampingDamageState");
    assert!(
        (state.current_bonus - 0.42).abs() < 1e-6,
        "current_bonus should be 0.42 (0.38 + 0.04) without cap, got {}",
        state.current_bonus
    );
}

// =========================================================================
// Behavior 11: reset_ramping_damage on non-bump breaker impact
// =========================================================================

#[test]
fn reset_ramping_damage_on_non_bump_breaker_impact() {
    let mut app = test_app_message();
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            RampingDamageState {
                current_bonus: 0.2,
                bonus_per_hit: 0.04,
            },
        ))
        .id();

    // Send BoltHitBreaker but NO BumpPerformed
    app.world_mut()
        .resource_mut::<TestBoltHitBreaker>()
        .0
        .push(BoltHitBreaker { bolt });

    tick(&mut app);

    let state = app
        .world()
        .get::<RampingDamageState>(bolt)
        .expect("bolt should have RampingDamageState");
    assert!(
        (state.current_bonus - 0.0).abs() < f32::EPSILON,
        "current_bonus should be reset to 0.0 on non-bump breaker impact, got {}",
        state.current_bonus
    );
    // bonus_per_hit should be unchanged
    assert!(
        (state.bonus_per_hit - 0.04).abs() < f32::EPSILON,
        "bonus_per_hit should be unchanged at 0.04, got {}",
        state.bonus_per_hit
    );
}

// =========================================================================
// Behavior 12: reset_ramping_damage does NOT reset when bump occurred
// =========================================================================

#[test]
fn reset_ramping_damage_preserves_bonus_when_bump_performed() {
    let mut app = test_app_message();
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            RampingDamageState {
                current_bonus: 0.2,
                bonus_per_hit: 0.04,
            },
        ))
        .id();

    // Send BOTH BoltHitBreaker and BumpPerformed for the same bolt
    app.world_mut()
        .resource_mut::<TestBoltHitBreaker>()
        .0
        .push(BoltHitBreaker { bolt });
    app.world_mut()
        .resource_mut::<TestBumpPerformed>()
        .0
        .push(BumpPerformed {
            grade: crate::breaker::messages::BumpGrade::Perfect,
            bolt: Some(bolt),
        });

    tick(&mut app);

    let state = app
        .world()
        .get::<RampingDamageState>(bolt)
        .expect("bolt should have RampingDamageState");
    assert!(
        (state.current_bonus - 0.2).abs() < f32::EPSILON,
        "current_bonus should be preserved at 0.2 when bump was performed, got {}",
        state.current_bonus
    );
}
