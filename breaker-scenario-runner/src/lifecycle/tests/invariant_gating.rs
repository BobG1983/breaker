use super::helpers::*;

/// Minimal app that registers `check_bolt_speed_accurate` under the new
/// `playing_state_gate`, with `entered_playing = true` and the given
/// `NodeState` value forced as the current state. Avoids the full
/// lifecycle plugin so the state isn't driven by other systems mid-tick.
fn bolt_speed_gate_test_app(initial_state: NodeState) -> App {
    use bevy::state::app::StatesPlugin;

    use crate::{invariants::check_bolt_speed_accurate, lifecycle::systems::playing_state_gate};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .insert_resource(ViolationLog::default())
        .insert_resource(ScenarioFrame(1))
        .insert_resource(ScenarioStats {
            entered_playing: true,
            ..Default::default()
        })
        .insert_resource(State::new(initial_state))
        .add_systems(
            FixedUpdate,
            check_bolt_speed_accurate.run_if(playing_state_gate),
        );
    app
}

// -------------------------------------------------------------------------
// ScenarioLifecycle — invariant system registration
// -------------------------------------------------------------------------

/// `check_bolt_in_bounds` is defined in `invariants.rs` but must be registered
/// by [`ScenarioLifecycle`]. A bolt entity at y = 500.0 is above the top
/// bound of a 700-unit-tall playfield (top = 350.0). After one tick the
/// [`ViolationLog`] must contain exactly one entry with
/// [`InvariantKind::BoltInBounds`].
#[test]
fn check_bolt_in_bounds_is_registered_in_scenario_lifecycle() {
    let mut app = lifecycle_test_app();

    // Override playfield so top() = 350.0
    app.world_mut().insert_resource(PlayfieldConfig {
        width:                800.0,
        height:               700.0,
        background_color_rgb: [0.0, 0.0, 0.0],
        wall_thickness:       180.0,
        zone_fraction:        0.667,
    });

    // Set entered_playing so invariant checkers are active
    app.world_mut()
        .resource_mut::<ScenarioStats>()
        .entered_playing = true;

    // Spawn bolt well above the top bound
    app.world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 500.0))));

    // Satisfy BreakerCountReasonable (expects exactly 1 PrimaryBreaker)
    app.world_mut().spawn(PrimaryBreaker);

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0.len(),
        1,
        "expected 1 BoltInBounds violation from ScenarioLifecycle, got {}",
        log.0.len()
    );
    assert_eq!(
        log.0[0].invariant,
        InvariantKind::BoltInBounds,
        "expected BoltInBounds invariant kind"
    );
}

/// `check_no_nan` is defined in `invariants.rs` but must be registered by
/// [`ScenarioLifecycle`]. A bolt entity with `f32::NAN` in its x position
/// must produce a [`ViolationEntry`] with [`InvariantKind::NoNaN`] after one tick.
///
/// This test FAILS until `check_no_nan` is added to `ScenarioLifecycle::build()`.
#[test]
fn check_no_nan_is_registered_in_scenario_lifecycle() {
    let mut app = lifecycle_test_app();

    // Set entered_playing so invariant checkers are active
    app.world_mut()
        .resource_mut::<ScenarioStats>()
        .entered_playing = true;

    app.world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(f32::NAN, 0.0))));

    // Satisfy BreakerCountReasonable (expects exactly 1 PrimaryBreaker)
    app.world_mut().spawn(PrimaryBreaker);

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        !log.0.is_empty(),
        "expected at least one NoNaN violation from ScenarioLifecycle, got none"
    );
    assert_eq!(
        log.0[0].invariant,
        InvariantKind::NoNaN,
        "expected NoNaN invariant kind"
    );
}

// -------------------------------------------------------------------------
// Invariant checker gating — entered_playing
// -------------------------------------------------------------------------

/// Invariant checkers must NOT produce violations when
/// `ScenarioStats::entered_playing` is `false`. This simulates the
/// `GameState::Loading` phase where entities may not be fully initialized.
///
/// Given: `entered_playing = false`, bolt at (0.0, 999.0) — well above
/// the top bound (350.0 for a 700.0-height playfield). Despite the bolt
/// being clearly out of bounds, the checker must NOT fire because the
/// game has not yet entered `Playing`.
#[test]
fn invariant_checkers_do_not_fire_when_entered_playing_is_false() {
    use crate::invariants::{ScenarioStats, ScenarioTagBolt, check_bolt_in_bounds};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ViolationLog::default())
        .insert_resource(ScenarioFrame(1))
        .insert_resource(breaker::shared::PlayfieldConfig {
            width:                800.0,
            height:               700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness:       180.0,
            zone_fraction:        0.667,
        })
        .insert_resource(ScenarioStats {
            entered_playing: false,
            ..Default::default()
        })
        .add_systems(FixedUpdate, check_bolt_in_bounds);

    // Bolt at y = 999.0 is well above top bound (350.0). Without the
    // entered_playing gate this would fire a BoltInBounds violation.
    app.world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 999.0))));

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "expected no violations when entered_playing is false, but got {}: {:?}",
        log.0.len(),
        log.0.iter().map(|e| &e.message).collect::<Vec<_>>()
    );
}

/// Invariant checkers MUST produce violations when
/// `ScenarioStats::entered_playing` is `true` and a bolt is out of bounds.
///
/// Given: `entered_playing = true`, bolt at (0.0, 999.0) — above the top
/// bound (350.0 for a 700.0-height playfield).
///
/// This is the control test that confirms the checker fires normally
/// when the gate condition is met.
#[test]
fn invariant_checkers_fire_when_entered_playing_is_true() {
    use crate::invariants::{ScenarioStats, ScenarioTagBolt, check_bolt_in_bounds};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ViolationLog::default())
        .insert_resource(ScenarioFrame(1))
        .insert_resource(breaker::shared::PlayfieldConfig {
            width:                800.0,
            height:               700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness:       180.0,
            zone_fraction:        0.667,
        })
        .insert_resource(ScenarioStats {
            entered_playing: true,
            ..Default::default()
        })
        .add_systems(FixedUpdate, check_bolt_in_bounds);

    // Bolt at y = 999.0 is above top bound (350.0) — should produce a violation.
    app.world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 999.0))));

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        !log.0.is_empty(),
        "expected at least one BoltInBounds violation when entered_playing is true and bolt is OOB"
    );
    assert_eq!(
        log.0[0].invariant,
        InvariantKind::BoltInBounds,
        "expected BoltInBounds invariant kind"
    );
}

/// Invariant checkers must remain gated across multiple frames while
/// `entered_playing` is `false`. Even after 5 ticks with an OOB bolt,
/// the `ViolationLog` must stay empty.
#[test]
fn invariant_checkers_remain_gated_across_multiple_frames_while_not_playing() {
    use crate::invariants::{ScenarioStats, ScenarioTagBolt, check_bolt_in_bounds};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ViolationLog::default())
        .insert_resource(ScenarioFrame(1))
        .insert_resource(breaker::shared::PlayfieldConfig {
            width:                800.0,
            height:               700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness:       180.0,
            zone_fraction:        0.667,
        })
        .insert_resource(ScenarioStats {
            entered_playing: false,
            ..Default::default()
        })
        .add_systems(FixedUpdate, check_bolt_in_bounds);

    // Bolt far above top bound — would fire if not gated
    app.world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 999.0))));

    for _ in 0..5 {
        tick(&mut app);
    }

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "expected no violations after 5 ticks with entered_playing=false, but got {}: {:?}",
        log.0.len(),
        log.0.iter().map(|e| &e.message).collect::<Vec<_>>()
    );
}

// -------------------------------------------------------------------------
// ScenarioStats — invariant_checks incremented by invariant system
// -------------------------------------------------------------------------

/// After one tick with a tagged bolt present, `ScenarioStats::invariant_checks`
/// must be greater than zero. The `check_bolt_in_bounds` system must increment
/// the counter when it runs.
#[test]
fn scenario_stats_invariant_checks_incremented_after_one_tick() {
    use crate::invariants::{ScenarioStats, ScenarioTagBolt, check_bolt_in_bounds};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ViolationLog::default())
        .insert_resource(ScenarioFrame::default())
        .insert_resource(breaker::shared::PlayfieldConfig::default())
        .insert_resource(ScenarioStats {
            entered_playing: true,
            ..Default::default()
        })
        .add_systems(FixedUpdate, check_bolt_in_bounds);

    app.world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 0.0))));

    tick(&mut app);

    let stats = app.world().resource::<ScenarioStats>();
    assert!(
        stats.invariant_checks > 0,
        "expected invariant_checks > 0 after one tick with bolt entity, got {}",
        stats.invariant_checks
    );
}

// -------------------------------------------------------------------------
// BoltSpeedAccurate — current-NodeState gate (playing_state_gate)
// -------------------------------------------------------------------------

/// `check_bolt_speed_accurate` must NOT fire when the current `NodeState`
/// is not `Playing`, even if `entered_playing` is `true`. This protects
/// against transient mismatches during `Teardown` / `ChipSelect` /
/// `Loading`, when `sync_bolt_speed_to_stack` (the system that maintains
/// the stack-vs-velocity invariant) does not run.
///
/// Concretely: if `Until(NodeEndOccurred, Fire(SpeedBoost(1.3)))` reverses
/// on `OnEnter(NodeState::Teardown)`, the stack clears but the bolt
/// velocity remains at `base * 1.3` until the next `Playing` tick. The
/// game-side invariant only holds during Playing, so the checker must
/// mirror that.
#[test]
fn bolt_speed_accurate_does_not_fire_outside_node_state_playing() {
    use rantzsoft_spatial2d::components::{BaseSpeed, MaxSpeed, MinSpeed};

    let mut app = bolt_speed_gate_test_app(NodeState::Teardown);

    // Spawn a fully-equipped bolt with mismatched velocity (520 vs base
    // 400 — would otherwise trip BoltSpeedAccurate). Adding the system
    // requires `playing_state_gate` semantics, so this asserts via the
    // registered system in `bolt_speed_gate_test_app`.
    app.world_mut().spawn((
        ScenarioTagBolt,
        Position2D(Vec2::new(0.0, 0.0)),
        Velocity2D(Vec2::new(520.0, 0.0)),
        BaseSpeed(400.0),
        MinSpeed(200.0),
        MaxSpeed(800.0),
    ));

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        !log.0
            .iter()
            .any(|v| v.invariant == InvariantKind::BoltSpeedAccurate),
        "expected no BoltSpeedAccurate violations when NodeState != Playing, got: {:?}",
        log.0
            .iter()
            .filter(|v| v.invariant == InvariantKind::BoltSpeedAccurate)
            .map(|v| &v.message)
            .collect::<Vec<_>>(),
    );
}

/// Control test: `check_bolt_speed_accurate` MUST fire when the current
/// `NodeState` is `Playing` and the velocity is mismatched. This pairs
/// with the gating test above to confirm the new gate isn't suppressing
/// legitimate violations.
#[test]
fn bolt_speed_accurate_fires_in_node_state_playing() {
    use rantzsoft_spatial2d::components::{BaseSpeed, MaxSpeed, MinSpeed};

    let mut app = bolt_speed_gate_test_app(NodeState::Playing);

    app.world_mut().spawn((
        ScenarioTagBolt,
        Position2D(Vec2::new(0.0, 0.0)),
        Velocity2D(Vec2::new(520.0, 0.0)),
        BaseSpeed(400.0),
        MinSpeed(200.0),
        MaxSpeed(800.0),
    ));

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0
            .iter()
            .any(|v| v.invariant == InvariantKind::BoltSpeedAccurate),
        "expected at least one BoltSpeedAccurate violation in NodeState::Playing with mismatched velocity, got 0",
    );
}

/// `playing_state_gate` returns `false` when `State<NodeState>` resource
/// is absent (e.g. minimal test apps that didn't add `StatesPlugin`).
/// The gate must be well-defined in this case so the registered system
/// is never invoked with stale or surprising state.
#[test]
fn bolt_speed_accurate_does_not_fire_when_node_state_resource_absent() {
    use rantzsoft_spatial2d::components::{BaseSpeed, MaxSpeed, MinSpeed};

    use crate::{invariants::check_bolt_speed_accurate, lifecycle::systems::playing_state_gate};

    // Build the same test surface as `bolt_speed_gate_test_app` but
    // intentionally omit `StatesPlugin` and `State::new(...)` so the
    // `Option<Res<State<NodeState>>>` parameter resolves to `None`.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ViolationLog::default())
        .insert_resource(ScenarioFrame(1))
        .insert_resource(ScenarioStats {
            entered_playing: true,
            ..Default::default()
        })
        .add_systems(
            FixedUpdate,
            check_bolt_speed_accurate.run_if(playing_state_gate),
        );

    app.world_mut().spawn((
        ScenarioTagBolt,
        Position2D(Vec2::new(0.0, 0.0)),
        Velocity2D(Vec2::new(520.0, 0.0)),
        BaseSpeed(400.0),
        MinSpeed(200.0),
        MaxSpeed(800.0),
    ));

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        !log.0
            .iter()
            .any(|v| v.invariant == InvariantKind::BoltSpeedAccurate),
        "expected no BoltSpeedAccurate violations when State<NodeState> is absent, got {} violation(s)",
        log.0
            .iter()
            .filter(|v| v.invariant == InvariantKind::BoltSpeedAccurate)
            .count(),
    );
}

/// `enforce_frozen_velocity` must NOT mutate `Velocity2D` when
/// `ScenarioPhysicsFrozen.velocity` is `None`. The position-only freeze
/// path (e.g. a frozen breaker) must leave bolt velocity alone.
#[test]
fn enforce_frozen_velocity_skips_when_pinned_velocity_is_none() {
    use crate::{invariants::ScenarioPhysicsFrozen, lifecycle::systems::enforce_frozen_velocity};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(FixedUpdate, enforce_frozen_velocity);

    let original_velocity = Vec2::new(123.0, -456.0);
    let entity = app
        .world_mut()
        .spawn((
            ScenarioPhysicsFrozen {
                target:   Vec2::new(0.0, 0.0),
                velocity: None,
            },
            Velocity2D(original_velocity),
        ))
        .id();

    tick(&mut app);

    let velocity = app
        .world()
        .entity(entity)
        .get::<Velocity2D>()
        .expect("Velocity2D should still be present");
    assert_eq!(
        velocity.0, original_velocity,
        "enforce_frozen_velocity must not mutate Velocity2D when pinned.velocity is None",
    );
}

// -------------------------------------------------------------------------
// ScenarioStats — entered_playing set by mark_entered_playing_on_spawn_complete
// -------------------------------------------------------------------------

/// When `SpawnNodeComplete` fires, `mark_entered_playing_on_spawn_complete`
/// sets `ScenarioStats::entered_playing` to `true`.
#[test]
fn scenario_stats_entered_playing_set_on_spawn_node_complete() {
    use breaker::state::run::node::messages::SpawnNodeComplete;

    use crate::invariants::ScenarioStats;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<ScenarioStats>()
        .add_message::<SpawnNodeComplete>()
        .add_systems(Update, mark_entered_playing_on_spawn_complete);

    // Send SpawnNodeComplete message
    app.world_mut()
        .resource_mut::<Messages<SpawnNodeComplete>>()
        .write(SpawnNodeComplete);

    app.update();

    let stats = app.world().resource::<ScenarioStats>();
    assert!(
        stats.entered_playing,
        "expected entered_playing == true after SpawnNodeComplete"
    );
}
