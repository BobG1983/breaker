use super::helpers::*;

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — no-op when frame_mutations is None
// -------------------------------------------------------------------------

/// When `frame_mutations` is `None`, `apply_debug_frame_mutations` must
/// do nothing and not panic at any frame.
#[test]
fn apply_debug_frame_mutations_noop_when_none() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        ..Default::default()
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(5))
        .add_systems(Update, apply_debug_frame_mutations);

    // Must not panic
    app.update();
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — SetBreakerState at matching frame
// -------------------------------------------------------------------------

/// When `frame_mutations` has `SetBreakerState(Braking)` at frame 3 and
/// the current frame is 3, the breaker entity's `BreakerState` must
/// become `BreakerState::Braking`.
#[test]
fn apply_debug_frame_mutations_set_breaker_state_at_matching_frame() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        frame_mutations: Some(vec![FrameMutation {
            frame: 3,
            mutation: MutationKind::SetBreakerState(ScenarioBreakerState::Braking),
        }]),
        ..Default::default()
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(3))
        .add_systems(Update, apply_debug_frame_mutations);

    let entity = app
        .world_mut()
        .spawn((ScenarioTagBreaker, BreakerState::Idle))
        .id();

    app.update();

    let state = app
        .world()
        .entity(entity)
        .get::<BreakerState>()
        .expect("entity must still have BreakerState");
    assert_eq!(
        *state,
        BreakerState::Braking,
        "expected BreakerState::Braking at frame 3, got {state:?}"
    );
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — SetBreakerState does NOT apply at non-matching frame
// -------------------------------------------------------------------------

/// When `frame_mutations` has `SetBreakerState(Braking)` at frame 3 but
/// the current frame is 2, the breaker must remain `Idle`.
#[test]
fn apply_debug_frame_mutations_set_breaker_state_skips_non_matching_frame() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        frame_mutations: Some(vec![FrameMutation {
            frame: 3,
            mutation: MutationKind::SetBreakerState(ScenarioBreakerState::Braking),
        }]),
        ..Default::default()
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(2))
        .add_systems(Update, apply_debug_frame_mutations);

    let entity = app
        .world_mut()
        .spawn((ScenarioTagBreaker, BreakerState::Idle))
        .id();

    app.update();

    let state = app
        .world()
        .entity(entity)
        .get::<BreakerState>()
        .expect("entity must still have BreakerState");
    assert_eq!(
        *state,
        BreakerState::Idle,
        "expected BreakerState::Idle at frame 2 (mutation at frame 3), got {state:?}"
    );
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — SetTimerRemaining at matching frame
// -------------------------------------------------------------------------

/// When `frame_mutations` has `SetTimerRemaining(61.0)` at frame 5 and
/// the current frame is 5, `NodeTimer.remaining` must be set to 61.0.
#[test]
fn apply_debug_frame_mutations_set_timer_remaining_at_matching_frame() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        frame_mutations: Some(vec![FrameMutation {
            frame: 5,
            mutation: MutationKind::SetTimerRemaining(61.0),
        }]),
        ..Default::default()
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(5))
        .insert_resource(NodeTimer {
            remaining: 55.0,
            total: 60.0,
        })
        .add_systems(Update, apply_debug_frame_mutations);

    app.update();

    let timer = app.world().resource::<NodeTimer>();
    assert!(
        (timer.remaining - 61.0_f32).abs() < f32::EPSILON,
        "expected NodeTimer.remaining == 61.0, got {}",
        timer.remaining
    );
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — SetTimerRemaining no-op when no NodeTimer
// -------------------------------------------------------------------------

/// When `frame_mutations` has `SetTimerRemaining(61.0)` at frame 5 but
/// no `NodeTimer` resource exists, the system must not panic.
#[test]
fn apply_debug_frame_mutations_set_timer_remaining_noop_when_no_timer() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        frame_mutations: Some(vec![FrameMutation {
            frame: 5,
            mutation: MutationKind::SetTimerRemaining(61.0),
        }]),
        ..Default::default()
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(5))
        .add_systems(Update, apply_debug_frame_mutations);

    // Deliberately do NOT insert NodeTimer — must not panic
    app.update();
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — SpawnExtraEntities at matching frame
// -------------------------------------------------------------------------

/// When `frame_mutations` has `SpawnExtraEntities(5)` at frame 10 and
/// the current frame is 10, 5 new entities with `Transform` must be spawned.
#[test]
fn apply_debug_frame_mutations_spawn_extra_entities_at_matching_frame() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        frame_mutations: Some(vec![FrameMutation {
            frame: 10,
            mutation: MutationKind::SpawnExtraEntities(5),
        }]),
        ..Default::default()
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(10))
        .add_systems(Update, apply_debug_frame_mutations);

    // Single update only — avoid double-spawn from running the system twice
    app.update();

    let count = app
        .world_mut()
        .query_filtered::<Entity, With<Transform>>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 5,
        "expected 5 entities with Transform from SpawnExtraEntities(5), got {count}"
    );
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — MoveBolt at matching frame
// -------------------------------------------------------------------------

/// When `frame_mutations` has `MoveBolt(999.0, 999.0)` at frame 5 and
/// the current frame is 5, the tagged bolt must be moved to (999.0, 999.0).
#[test]
fn apply_debug_frame_mutations_move_bolt_at_matching_frame() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        frame_mutations: Some(vec![FrameMutation {
            frame: 5,
            mutation: MutationKind::MoveBolt(999.0, 999.0),
        }]),
        ..Default::default()
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(5))
        .add_systems(Update, apply_debug_frame_mutations);

    let entity = app
        .world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 0.0))))
        .id();

    app.update();

    let position = app
        .world()
        .entity(entity)
        .get::<Position2D>()
        .expect("entity must still have Position2D");
    assert_eq!(
        position.0,
        Vec2::new(999.0, 999.0),
        "expected bolt at (999.0, 999.0), got {:?}",
        position.0
    );
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — TogglePause sets NextState to Paused
// -------------------------------------------------------------------------

/// When `frame_mutations` has `TogglePause` at frame 3, the current frame
/// is 3, and the game is in `PlayingState::Active`, the system must set
/// `NextState<PlayingState>` to `Paused`.
#[test]
fn apply_debug_frame_mutations_toggle_pause_sets_paused() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        frame_mutations: Some(vec![FrameMutation {
            frame: 3,
            mutation: MutationKind::TogglePause,
        }]),
        ..Default::default()
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .init_state::<GameState>()
        .add_sub_state::<PlayingState>()
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(3))
        .add_systems(Update, apply_debug_frame_mutations);

    // Drive into GameState::Playing so PlayingState becomes active.
    // Single update: state transition activates PlayingState, then the mutation
    // system runs (frame=3 matches) and sets NextState<PlayingState> to Paused.
    // A second update would process that transition, then the system would toggle
    // back to Active (because TogglePause runs again at the same frame).
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();

    // Check that NextState<PlayingState> is set to Paused
    let next = app.world().resource::<NextState<PlayingState>>();
    assert!(
        matches!(next, NextState::Pending(PlayingState::Paused)),
        "expected NextState::Pending(Paused), got: {next:?}"
    );
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — multiple mutations on same frame
// -------------------------------------------------------------------------

/// When two mutations are scheduled for the same frame, both must apply.
/// Given `SetBreakerState(Braking)` and `MoveBolt(100.0, 200.0)` both at
/// frame 5, the breaker state must change AND the bolt must move.
#[test]
fn apply_debug_frame_mutations_multiple_mutations_on_same_frame() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        frame_mutations: Some(vec![
            FrameMutation {
                frame: 5,
                mutation: MutationKind::SetBreakerState(ScenarioBreakerState::Braking),
            },
            FrameMutation {
                frame: 5,
                mutation: MutationKind::MoveBolt(100.0, 200.0),
            },
        ]),
        ..Default::default()
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(5))
        .add_systems(Update, apply_debug_frame_mutations);

    let breaker_entity = app
        .world_mut()
        .spawn((ScenarioTagBreaker, BreakerState::Idle))
        .id();
    let bolt_entity = app
        .world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 0.0))))
        .id();

    app.update();

    let state = app
        .world()
        .entity(breaker_entity)
        .get::<BreakerState>()
        .expect("breaker entity must still have BreakerState");
    assert_eq!(
        *state,
        BreakerState::Braking,
        "expected BreakerState::Braking from first mutation, got {state:?}"
    );

    let position = app
        .world()
        .entity(bolt_entity)
        .get::<Position2D>()
        .expect("bolt entity must still have Position2D");
    assert_eq!(
        position.0,
        Vec2::new(100.0, 200.0),
        "expected bolt at (100.0, 200.0) from second mutation, got {:?}",
        position.0
    );
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — SpawnExtraChainArcs at matching frame
// -------------------------------------------------------------------------

/// When `frame_mutations` has `SpawnExtraChainArcs(3)` at frame 30 and the
/// current frame is 30, 3 `ChainLightningChain` entities and 3
/// `ChainLightningArc` entities must be spawned (6 total). Each entity
/// must also have a `CleanupOnNodeExit` marker.
#[test]
fn apply_debug_frame_mutations_spawn_extra_chain_arcs_at_matching_frame() {
    use breaker::{
        effect::effects::chain_lightning::{ChainLightningArc, ChainLightningChain},
        shared::CleanupOnNodeExit,
    };

    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        frame_mutations: Some(vec![FrameMutation {
            frame: 30,
            mutation: MutationKind::SpawnExtraChainArcs(3),
        }]),
        ..Default::default()
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(30))
        .add_systems(Update, apply_debug_frame_mutations);

    // Single update only — avoid double-spawn
    app.update();

    let chain_count = app
        .world_mut()
        .query_filtered::<Entity, With<ChainLightningChain>>()
        .iter(app.world())
        .count();
    assert_eq!(
        chain_count, 3,
        "expected 3 ChainLightningChain entities from SpawnExtraChainArcs(3), got {chain_count}"
    );

    let arc_count = app
        .world_mut()
        .query_filtered::<Entity, With<ChainLightningArc>>()
        .iter(app.world())
        .count();
    assert_eq!(
        arc_count, 3,
        "expected 3 ChainLightningArc entities from SpawnExtraChainArcs(3), got {arc_count}"
    );

    // Verify CleanupOnNodeExit marker on chain entities
    let chain_cleanup_count = app
        .world_mut()
        .query_filtered::<Entity, (With<ChainLightningChain>, With<CleanupOnNodeExit>)>()
        .iter(app.world())
        .count();
    assert_eq!(
        chain_cleanup_count, 3,
        "all ChainLightningChain entities must have CleanupOnNodeExit"
    );

    // Verify CleanupOnNodeExit marker on arc entities
    let arc_cleanup_count = app
        .world_mut()
        .query_filtered::<Entity, (With<ChainLightningArc>, With<CleanupOnNodeExit>)>()
        .iter(app.world())
        .count();
    assert_eq!(
        arc_cleanup_count, 3,
        "all ChainLightningArc entities must have CleanupOnNodeExit"
    );
}

/// When `frame_mutations` has `SpawnExtraChainArcs(0)` at frame 30, no
/// entities should be spawned.
#[test]
fn apply_debug_frame_mutations_spawn_extra_chain_arcs_zero_spawns_nothing() {
    use breaker::effect::effects::chain_lightning::{ChainLightningArc, ChainLightningChain};

    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        frame_mutations: Some(vec![FrameMutation {
            frame: 30,
            mutation: MutationKind::SpawnExtraChainArcs(0),
        }]),
        ..Default::default()
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(30))
        .add_systems(Update, apply_debug_frame_mutations);

    app.update();

    let chain_count = app
        .world_mut()
        .query_filtered::<Entity, With<ChainLightningChain>>()
        .iter(app.world())
        .count();
    assert_eq!(
        chain_count, 0,
        "SpawnExtraChainArcs(0) should spawn no ChainLightningChain entities"
    );

    let arc_count = app
        .world_mut()
        .query_filtered::<Entity, With<ChainLightningArc>>()
        .iter(app.world())
        .count();
    assert_eq!(
        arc_count, 0,
        "SpawnExtraChainArcs(0) should spawn no ChainLightningArc entities"
    );
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — SpawnExtraChainArcs does NOT apply at non-matching frame
// -------------------------------------------------------------------------

/// When `frame_mutations` has `SpawnExtraChainArcs(5)` at frame 30 but the
/// current frame is 29, no entities should be spawned.
#[test]
fn apply_debug_frame_mutations_spawn_extra_chain_arcs_skips_non_matching_frame() {
    use breaker::effect::effects::chain_lightning::{ChainLightningArc, ChainLightningChain};

    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        frame_mutations: Some(vec![FrameMutation {
            frame: 30,
            mutation: MutationKind::SpawnExtraChainArcs(5),
        }]),
        ..Default::default()
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(29))
        .add_systems(Update, apply_debug_frame_mutations);

    app.update();

    let chain_count = app
        .world_mut()
        .query_filtered::<Entity, With<ChainLightningChain>>()
        .iter(app.world())
        .count();
    assert_eq!(
        chain_count, 0,
        "SpawnExtraChainArcs should not fire at frame 29 (mutation at frame 30)"
    );

    let arc_count = app
        .world_mut()
        .query_filtered::<Entity, With<ChainLightningArc>>()
        .iter(app.world())
        .count();
    assert_eq!(
        arc_count, 0,
        "SpawnExtraChainArcs should not fire at frame 29 (mutation at frame 30)"
    );
}

/// Frame 31 also does not trigger the mutation at frame 30.
#[test]
fn apply_debug_frame_mutations_spawn_extra_chain_arcs_skips_frame_after() {
    use breaker::effect::effects::chain_lightning::{ChainLightningArc, ChainLightningChain};

    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        frame_mutations: Some(vec![FrameMutation {
            frame: 30,
            mutation: MutationKind::SpawnExtraChainArcs(5),
        }]),
        ..Default::default()
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(31))
        .add_systems(Update, apply_debug_frame_mutations);

    app.update();

    let chain_count = app
        .world_mut()
        .query_filtered::<Entity, With<ChainLightningChain>>()
        .iter(app.world())
        .count();
    assert_eq!(
        chain_count, 0,
        "SpawnExtraChainArcs should not fire at frame 31 (mutation at frame 30)"
    );

    let arc_count = app
        .world_mut()
        .query_filtered::<Entity, With<ChainLightningArc>>()
        .iter(app.world())
        .count();
    assert_eq!(
        arc_count, 0,
        "SpawnExtraChainArcs should not fire at frame 31 (mutation at frame 30)"
    );
}
