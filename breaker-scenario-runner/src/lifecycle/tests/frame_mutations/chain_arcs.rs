//! Tests for `SpawnExtraChainArcs` frame mutation.

use super::super::helpers::*;

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
        disallowed_failures: vec![],
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
        disallowed_failures: vec![],
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
        disallowed_failures: vec![],
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
        disallowed_failures: vec![],
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
