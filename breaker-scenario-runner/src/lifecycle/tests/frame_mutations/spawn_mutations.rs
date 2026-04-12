//! Tests for `SpawnExtraEntities` frame mutation.

use crate::lifecycle::tests::helpers::*;

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
        disallowed_failures: vec![],
        frame_mutations: Some(vec![FrameMutation {
            frame:    10,
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
