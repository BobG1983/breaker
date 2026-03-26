use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::helpers::*;
use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    effect::{
        definition::{Effect, EffectNode},
        effect_nodes::until::system::*,
        effects::{
            bolt_size_boost::ActiveSizeBoosts, bump_force_boost::ActiveBumpForces,
            piercing::ActivePiercings,
        },
    },
};

// =========================================================================
// Vec-based Until reversal — Piercing, SizeBoost, BumpForce
// =========================================================================

// --- Test 13: Timer expiry removes piercing entry from ActivePiercings ---

#[test]
fn reverse_piercing_removes_from_active_piercings() {
    let mut app = test_app();
    app.add_systems(FixedUpdate, tick_until_timers);

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            ActivePiercings(vec![2, 1]),
            UntilTimers(vec![UntilTimerEntry {
                remaining: 0.01, // dt = 1/64 > 0.01 — expires this tick
                children: vec![EffectNode::Do(Effect::Piercing(2))],
            }]),
        ))
        .id();

    tick(&mut app);

    // The 2 entry should be removed, leaving [1]
    let piercings = app
        .world()
        .get::<ActivePiercings>(bolt)
        .expect("bolt should have ActivePiercings");
    assert_eq!(
        piercings.0,
        vec![1],
        "ActivePiercings should be [1] after 2 is removed, got {:?}",
        piercings.0
    );
}

// --- Test 14: Timer expiry removes size boost entry from ActiveSizeBoosts ---

#[test]
fn reverse_size_boost_removes_from_active_size_boosts() {
    let mut app = test_app();
    app.add_systems(FixedUpdate, tick_until_timers);

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            ActiveSizeBoosts(vec![0.5, 0.3]),
            UntilTimers(vec![UntilTimerEntry {
                remaining: 0.01, // dt = 1/64 > 0.01 — expires this tick
                children: vec![EffectNode::Do(Effect::SizeBoost(0.5))],
            }]),
        ))
        .id();

    tick(&mut app);

    // The 0.5 entry should be removed, leaving [0.3]
    let boosts = app
        .world()
        .get::<ActiveSizeBoosts>(bolt)
        .expect("bolt should have ActiveSizeBoosts");
    assert_eq!(
        boosts.0,
        vec![0.3],
        "ActiveSizeBoosts should be [0.3] after 0.5 is removed, got {:?}",
        boosts.0
    );
}

// --- Test 15: Timer expiry removes bump force entry from ActiveBumpForces ---

#[test]
fn reverse_bump_force_removes_from_active_bump_forces() {
    let mut app = test_app();
    app.add_systems(FixedUpdate, tick_until_timers);

    let breaker = app
        .world_mut()
        .spawn((
            Breaker,
            ActiveBumpForces(vec![10.0, 15.0]),
            UntilTimers(vec![UntilTimerEntry {
                remaining: 0.01, // dt = 1/64 > 0.01 — expires this tick
                children: vec![EffectNode::Do(Effect::BumpForce(10.0))],
            }]),
        ))
        .id();

    tick(&mut app);

    // The 10.0 entry should be removed, leaving [15.0]
    let forces = app
        .world()
        .get::<ActiveBumpForces>(breaker)
        .expect("breaker should have ActiveBumpForces");
    assert_eq!(
        forces.0,
        vec![15.0],
        "ActiveBumpForces should be [15.0] after 10.0 is removed, got {:?}",
        forces.0
    );
}
