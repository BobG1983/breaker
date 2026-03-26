use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::helpers::*;
use crate::{
    bolt::components::{Bolt, BoltBaseSpeed, BoltMaxSpeed},
    effect::{
        definition::{Effect, EffectNode, ImpactTarget, Trigger},
        effect_nodes::until::system::*,
        effects::speed_boost::{ActiveSpeedBoosts, apply_speed_boosts},
    },
};

// =========================================================================
// Sub-Feature A: Until Node — tick_until_timers (behaviors 5-6, 9-11a)
// =========================================================================

// --- Behavior 5: tick_until_timers decrements remaining each tick ---

#[test]
fn tick_until_timers_decrements_remaining_by_dt() {
    let mut app = test_app();
    app.add_systems(FixedUpdate, tick_until_timers);

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 800.0)),
            BoltBaseSpeed(400.0),
            BoltMaxSpeed(800.0),
            UntilTimers(vec![UntilTimerEntry {
                remaining: 3.0,
                children: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 2.0 })],
            }]),
        ))
        .id();

    tick(&mut app);

    let timers = app.world().get::<UntilTimers>(bolt).unwrap();
    let dt = 1.0 / 64.0; // MinimalPlugins default fixed timestep
    let expected = 3.0 - dt;
    assert!(
        (timers.0[0].remaining - expected).abs() < f32::EPSILON,
        "remaining should be {expected}, got {}",
        timers.0[0].remaining
    );
}

// --- Behavior 6: tick_until_timers reverses SpeedBoost on expiry ---

#[test]
fn tick_until_timers_reverses_speed_boost_on_expiry() {
    let mut app = test_app();
    app.add_systems(FixedUpdate, tick_until_timers);

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 800.0)),
            BoltBaseSpeed(400.0),
            BoltMaxSpeed(800.0),
            UntilTimers(vec![UntilTimerEntry {
                remaining: 0.01, // Will expire on next tick (dt = 1/64 > 0.01)
                children: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 2.0 })],
            }]),
        ))
        .id();

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    // Reversal: 800.0 / 2.0 = 400.0
    assert!(
        (vel.0.y - 400.0).abs() < 1.0,
        "velocity should be reversed to 400.0, got {}",
        vel.0.y
    );

    let timers = app.world().get::<UntilTimers>(bolt);
    // The entry should be removed. Since it was the only entry, UntilTimers
    // component should be removed entirely (behavior 11a).
    assert!(
        timers.is_none(),
        "UntilTimers component should be removed when empty"
    );
}

// --- Behavior 10: Non-reversible effect (SpawnBolts) — reversal is no-op ---

#[test]
fn tick_until_timers_non_reversible_effect_is_noop_on_expiry() {
    let mut app = test_app();
    app.add_systems(FixedUpdate, tick_until_timers);

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            UntilTimers(vec![UntilTimerEntry {
                remaining: 0.01,
                children: vec![EffectNode::Do(Effect::SpawnBolts {
                    count: 1,
                    lifespan: None,
                    inherit: false,
                })],
            }]),
        ))
        .id();

    tick(&mut app);

    // Timer entry should be removed. No panic, no velocity change.
    let timers = app.world().get::<UntilTimers>(bolt);
    assert!(
        timers.is_none(),
        "UntilTimers should be removed after expiry of non-reversible effect"
    );

    // Velocity should be unchanged (no reversal for SpawnBolts)
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (vel.0.y - 400.0).abs() < f32::EPSILON,
        "velocity should be unchanged for non-reversible effect, got {}",
        vel.0.y
    );
}

// --- Behavior 11a: Empty UntilTimers component is removed ---

#[test]
fn tick_until_timers_removes_component_when_last_entry_expires() {
    let mut app = test_app();
    app.add_systems(FixedUpdate, tick_until_timers);

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            UntilTimers(vec![UntilTimerEntry {
                remaining: 0.01,
                children: vec![EffectNode::Do(Effect::SpawnBolts {
                    count: 1,
                    lifespan: None,
                    inherit: false,
                })],
            }]),
        ))
        .id();

    tick(&mut app);

    assert!(
        app.world().get::<UntilTimers>(bolt).is_none(),
        "UntilTimers component should be removed when all entries expire"
    );
}

// --- Behavior 9: Multiple buffs stack multiplicatively, removed independently ---

#[test]
fn tick_until_timers_removes_only_expired_entry_independently() {
    let mut app = test_app();
    app.add_systems(FixedUpdate, tick_until_timers);

    // Two Until buffs: SpeedBoost(2.0) expiring at 0.01 and SpeedBoost(1.5) expiring at 5.0
    // Base 400 * 2.0 * 1.5 = 1200, clamped to 800. When 2.0x expires: 800 / 2.0 = 400
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 800.0)),
            BoltBaseSpeed(400.0),
            BoltMaxSpeed(800.0),
            UntilTimers(vec![
                UntilTimerEntry {
                    remaining: 0.01, // Expires this tick
                    children: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 2.0 })],
                },
                UntilTimerEntry {
                    remaining: 5.0, // Still active
                    children: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.5 })],
                },
            ]),
        ))
        .id();

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    // Only the 2.0x buff expired: 800 / 2.0 = 400.0
    assert!(
        (vel.0.y - 400.0).abs() < 1.0,
        "velocity should be 400.0 after 2.0x buff expires, got {}",
        vel.0.y
    );

    // One entry should remain
    let timers = app.world().get::<UntilTimers>(bolt).unwrap();
    assert_eq!(timers.0.len(), 1, "only the unexpired entry should remain");
}

// =========================================================================
// IMPORTANT: Behavior 9 edge — both buffs expire same frame
// =========================================================================

#[test]
fn tick_until_timers_both_expire_same_frame_divides_sequentially() {
    let mut app = test_app();
    app.add_systems(FixedUpdate, tick_until_timers);

    // Two buffs both expiring on the same tick:
    // SpeedBoost(2.0) and SpeedBoost(1.5)
    // Starting velocity 600.0: 600.0 / 2.0 / 1.5 = 200.0
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 600.0)),
            BoltBaseSpeed(200.0),
            BoltMaxSpeed(800.0),
            UntilTimers(vec![
                UntilTimerEntry {
                    remaining: 0.01, // dt = 1/64 > 0.01 — expires this tick
                    children: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 2.0 })],
                },
                UntilTimerEntry {
                    remaining: 0.01, // Also expires this tick
                    children: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.5 })],
                },
            ]),
        ))
        .id();

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    // Both expired: 600.0 / 2.0 / 1.5 = 200.0
    assert!(
        (vel.0.y - 200.0).abs() < 1.0,
        "velocity should be 200.0 after both buffs expire, got {}",
        vel.0.y
    );

    // Both entries removed — component should be gone
    let timers = app.world().get::<UntilTimers>(bolt);
    assert!(
        timers.is_none(),
        "UntilTimers should be removed when all entries expire"
    );
}

// =========================================================================
// IMPORTANT: Behavior 6 edge — remaining exactly 0.0
// =========================================================================

#[test]
fn tick_until_timers_remaining_zero_triggers_removal_on_next_tick() {
    let mut app = test_app();
    app.add_systems(FixedUpdate, tick_until_timers);

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 800.0)),
            BoltBaseSpeed(400.0),
            BoltMaxSpeed(800.0),
            UntilTimers(vec![UntilTimerEntry {
                remaining: 0.0, // Exactly zero — should expire on next tick
                children: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 2.0 })],
            }]),
        ))
        .id();

    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    // Reversal: 800.0 / 2.0 = 400.0
    assert!(
        (vel.0.y - 400.0).abs() < 1.0,
        "velocity should be reversed to 400.0, got {}",
        vel.0.y
    );

    // Entry should be removed
    let timers = app.world().get::<UntilTimers>(bolt);
    assert!(
        timers.is_none(),
        "UntilTimers should be removed after zero-remaining entry expires"
    );
}

// =========================================================================
// Nested When/Once children inside Until entries
// =========================================================================

// --- Behavior: Timer expiry removes entry with nested When child ---

#[test]
fn tick_until_timers_nested_when_removed_on_expiry() {
    let mut app = test_app();
    app.add_systems(FixedUpdate, tick_until_timers);

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            UntilTimers(vec![UntilTimerEntry {
                remaining: 0.01, // dt = 1/64 > 0.01 — expires this tick
                children: vec![EffectNode::When {
                    trigger: Trigger::Impact(ImpactTarget::Cell),
                    then: vec![EffectNode::Do(Effect::Shockwave {
                        base_range: 64.0,
                        range_per_level: 0.0,
                        stacks: 1,
                        speed: 400.0,
                    })],
                }],
            }]),
        ))
        .id();

    tick(&mut app);

    // No reversal needed for When nodes — they're just removed.
    // UntilTimers component should be removed (empty after removal).
    assert!(
        app.world().get::<UntilTimers>(bolt).is_none(),
        "UntilTimers should be removed when nested When child's timer expires"
    );

    // Velocity should be unchanged (When is not a reversible effect)
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (vel.0.y - 400.0).abs() < f32::EPSILON,
        "velocity should be unchanged after When child removal, got {}",
        vel.0.y
    );
}

// --- Behavior: Timer expiry with mixed Do + When children ---

#[test]
fn tick_until_timers_mixed_children_reverses_do_ignores_when() {
    let mut app = test_app();
    app.add_systems(FixedUpdate, tick_until_timers);

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 600.0)),
            BoltBaseSpeed(400.0),
            BoltMaxSpeed(800.0),
            UntilTimers(vec![UntilTimerEntry {
                remaining: 0.01, // Expires this tick
                children: vec![
                    EffectNode::Do(Effect::SpeedBoost { multiplier: 1.5 }),
                    EffectNode::When {
                        trigger: Trigger::Impact(ImpactTarget::Cell),
                        then: vec![EffectNode::Do(Effect::Shockwave {
                            base_range: 48.0,
                            range_per_level: 0.0,
                            stacks: 1,
                            speed: 400.0,
                        })],
                    },
                ],
            }]),
        ))
        .id();

    tick(&mut app);

    // SpeedBoost reversed: 600.0 / 1.5 = 400.0
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (vel.0.y - 400.0).abs() < 1.0,
        "SpeedBoost should be reversed to 400.0, got {}",
        vel.0.y
    );

    // Entry removed — When node is simply gone (no reversal needed)
    assert!(
        app.world().get::<UntilTimers>(bolt).is_none(),
        "UntilTimers should be removed after mixed children entry expires"
    );
}

// =========================================================================
// Vec-based Until reversal — removes entries from ActiveSpeedBoosts
// =========================================================================

// --- Test 11: Timer expiry removes speed boost entry from vec ---

#[test]
fn tick_until_timers_removes_speed_boost_from_vec_on_expiry() {
    let mut app = test_app();
    app.add_systems(FixedUpdate, (tick_until_timers, apply_speed_boosts).chain());

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 1200.0)),
            BoltBaseSpeed(400.0),
            BoltMaxSpeed(2000.0),
            ActiveSpeedBoosts(vec![1.5, 2.0]),
            UntilTimers(vec![UntilTimerEntry {
                remaining: 0.01, // dt = 1/64 > 0.01 — expires this tick
                children: vec![EffectNode::Do(Effect::SpeedBoost { multiplier: 1.5 })],
            }]),
        ))
        .id();

    tick(&mut app);

    // The 1.5 entry should be removed, leaving [2.0]
    let boosts = app
        .world()
        .get::<ActiveSpeedBoosts>(bolt)
        .expect("bolt should have ActiveSpeedBoosts");
    assert_eq!(
        boosts.0,
        vec![2.0],
        "ActiveSpeedBoosts should be [2.0] after 1.5 expires, got {:?}",
        boosts.0
    );

    // Velocity should be recalculated: 400.0 * 2.0 = 800.0
    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (vel.speed() - 800.0).abs() < 1.0,
        "velocity should be recalculated to 800.0 (400 * 2.0), got {:.1}",
        vel.speed()
    );
}
