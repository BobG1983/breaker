use super::helpers::*;

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
