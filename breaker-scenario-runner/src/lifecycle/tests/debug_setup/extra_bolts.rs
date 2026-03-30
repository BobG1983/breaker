//! Tests for extra tagged bolt spawning via `apply_debug_setup`.

use super::super::helpers::*;

// -------------------------------------------------------------------------
// apply_debug_setup — spawns extra tagged bolts
// -------------------------------------------------------------------------

/// When `extra_tagged_bolts: Some(5)`, `apply_debug_setup` must spawn 5 extra
/// `ScenarioTagBolt` entities. Combined with the 1 existing tagged bolt, the
/// total must be 6. The extra entities must NOT have `Bolt`, `BoltVelocity`,
/// `BoltMinSpeed`, or `BoltMaxSpeed` components.
#[test]
fn apply_debug_setup_spawns_extra_tagged_bolts() {
    use breaker::bolt::components::{BoltMaxSpeed, BoltMinSpeed};

    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        debug_setup: Some(DebugSetup {
            extra_tagged_bolts: Some(5),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut app = debug_setup_app(definition);
    app.add_systems(Update, apply_debug_setup);

    // Spawn one existing tagged bolt
    app.world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 0.0))));

    // Single update: apply_debug_setup runs as OnEnter in production (fires once).
    // Two updates would run the system twice, doubling the spawned count.
    app.update();

    // Count all ScenarioTagBolt entities
    let tagged_count = app
        .world_mut()
        .query_filtered::<Entity, With<ScenarioTagBolt>>()
        .iter(app.world())
        .count();
    assert_eq!(
        tagged_count, 6,
        "expected 6 total ScenarioTagBolt entities (1 original + 5 extra), got {tagged_count}"
    );

    // Verify extra bolts do NOT have physics components
    let bolts_with_bolt_component = app
        .world_mut()
        .query_filtered::<Entity, (With<ScenarioTagBolt>, With<Bolt>)>()
        .iter(app.world())
        .count();
    assert_eq!(
        bolts_with_bolt_component, 0,
        "extra tagged bolts must NOT have Bolt component, found {bolts_with_bolt_component}"
    );

    let bolts_with_velocity = app
        .world_mut()
        .query_filtered::<Entity, (With<ScenarioTagBolt>, With<Velocity2D>)>()
        .iter(app.world())
        .count();
    assert_eq!(
        bolts_with_velocity, 0,
        "extra tagged bolts must NOT have Velocity2D component, found {bolts_with_velocity}"
    );

    let bolts_with_min_speed = app
        .world_mut()
        .query_filtered::<Entity, (With<ScenarioTagBolt>, With<BoltMinSpeed>)>()
        .iter(app.world())
        .count();
    assert_eq!(
        bolts_with_min_speed, 0,
        "extra tagged bolts must NOT have BoltMinSpeed component, found {bolts_with_min_speed}"
    );

    let bolts_with_max_speed = app
        .world_mut()
        .query_filtered::<Entity, (With<ScenarioTagBolt>, With<BoltMaxSpeed>)>()
        .iter(app.world())
        .count();
    assert_eq!(
        bolts_with_max_speed, 0,
        "extra tagged bolts must NOT have BoltMaxSpeed component, found {bolts_with_max_speed}"
    );
}

// -------------------------------------------------------------------------
// apply_debug_setup — spawns zero extra bolts when Some(0)
// -------------------------------------------------------------------------

/// When `extra_tagged_bolts: Some(0)`, no extra entities should be spawned.
/// Total `ScenarioTagBolt` count remains 1.
#[test]
fn apply_debug_setup_spawns_zero_extra_bolts_when_some_zero() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        debug_setup: Some(DebugSetup {
            extra_tagged_bolts: Some(0),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut app = debug_setup_app(definition);
    app.add_systems(Update, apply_debug_setup);

    app.world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 0.0))));

    app.update();
    app.update();

    let tagged_count = app
        .world_mut()
        .query_filtered::<Entity, With<ScenarioTagBolt>>()
        .iter(app.world())
        .count();
    assert_eq!(
        tagged_count, 1,
        "expected 1 ScenarioTagBolt entity (no extras from Some(0)), got {tagged_count}"
    );
}
