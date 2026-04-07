//! Tests for `SpawnExtraPrimaryBreakers` frame mutation.

use crate::lifecycle::tests::helpers::*;

// -------------------------------------------------------------------------
// apply_debug_frame_mutations -- SpawnExtraPrimaryBreakers at matching frame
// -------------------------------------------------------------------------

/// When `frame_mutations` has `SpawnExtraPrimaryBreakers(1)` at frame 5 and
/// the current frame is 5, exactly 1 entity with `PrimaryBreaker` must be
/// spawned.
///
/// Edge case: at `ScenarioFrame(4)` (one before target), 0 `PrimaryBreaker`
/// entities should exist -- verified by the non-matching frame test.
#[test]
fn apply_debug_frame_mutations_spawn_extra_primary_breakers_at_matching_frame() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        disallowed_failures: vec![],
        frame_mutations: Some(vec![FrameMutation {
            frame: 5,
            mutation: MutationKind::SpawnExtraPrimaryBreakers(1),
        }]),
        ..Default::default()
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(5))
        .add_systems(Update, apply_debug_frame_mutations);

    // Single update only -- avoid double-spawn
    app.update();

    let count = app
        .world_mut()
        .query_filtered::<Entity, With<PrimaryBreaker>>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 1,
        "expected 1 PrimaryBreaker entity from SpawnExtraPrimaryBreakers(1), got {count}"
    );
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations -- SpawnExtraPrimaryBreakers does NOT apply
// at non-matching frame
// -------------------------------------------------------------------------

/// When `frame_mutations` has `SpawnExtraPrimaryBreakers(1)` at frame 10 but
/// the current frame is 5, no `PrimaryBreaker` entities should be spawned.
///
/// Edge case: frame 11 (one after target) also produces 0 entities.
#[test]
fn apply_debug_frame_mutations_spawn_extra_primary_breakers_skips_non_matching_frame() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        disallowed_failures: vec![],
        frame_mutations: Some(vec![FrameMutation {
            frame: 10,
            mutation: MutationKind::SpawnExtraPrimaryBreakers(1),
        }]),
        ..Default::default()
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig {
            definition: definition.clone(),
        })
        .insert_resource(ScenarioFrame(5))
        .add_systems(Update, apply_debug_frame_mutations);

    app.update();

    let count = app
        .world_mut()
        .query_filtered::<Entity, With<PrimaryBreaker>>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 0,
        "SpawnExtraPrimaryBreakers should not fire at frame 5 (mutation at frame 10)"
    );

    // Also verify frame 11 (one after target) does not fire
    let mut app_after = App::new();
    app_after
        .add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(11))
        .add_systems(Update, apply_debug_frame_mutations);

    app_after.update();

    let count_after = app_after
        .world_mut()
        .query_filtered::<Entity, With<PrimaryBreaker>>()
        .iter(app_after.world())
        .count();
    assert_eq!(
        count_after, 0,
        "SpawnExtraPrimaryBreakers should not fire at frame 11 (mutation at frame 10)"
    );
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations -- SpawnExtraPrimaryBreakers(0) spawns nothing
// -------------------------------------------------------------------------

/// When `frame_mutations` has `SpawnExtraPrimaryBreakers(0)` at frame 5 and
/// the current frame is 5, no `PrimaryBreaker` entities should be spawned.
#[test]
fn apply_debug_frame_mutations_spawn_extra_primary_breakers_zero_spawns_nothing() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        disallowed_failures: vec![],
        frame_mutations: Some(vec![FrameMutation {
            frame: 5,
            mutation: MutationKind::SpawnExtraPrimaryBreakers(0),
        }]),
        ..Default::default()
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(5))
        .add_systems(Update, apply_debug_frame_mutations);

    app.update();

    let count = app
        .world_mut()
        .query_filtered::<Entity, With<PrimaryBreaker>>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 0,
        "SpawnExtraPrimaryBreakers(0) should spawn no PrimaryBreaker entities, got {count}"
    );
}
