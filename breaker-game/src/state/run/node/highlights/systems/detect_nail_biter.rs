//! System to detect `NailBiter` highlights when a node is cleared while a bolt
//! is near the bottom boundary.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    bolt::components::{Bolt, BoltServing},
    shared::PlayfieldConfig,
    state::run::{
        definition::HighlightConfig, messages::HighlightTriggered, node::messages::NodeCleared,
        resources::*,
    },
};

/// Reads [`NodeCleared`] messages and detects `NailBiter` highlights
/// when any active bolt is within `nail_biter_pixels` of the bottom boundary.
///
/// Records the highlight in [`RunStats`] and emits [`HighlightTriggered`].
pub(crate) fn detect_nail_biter(
    mut reader: MessageReader<NodeCleared>,
    bolt_query: Query<&Position2D, (With<Bolt>, Without<BoltServing>)>,
    playfield: Res<PlayfieldConfig>,
    config: Res<HighlightConfig>,
    mut stats: ResMut<RunStats>,
    run_state: Res<RunState>,
    mut writer: MessageWriter<HighlightTriggered>,
) {
    let bottom = playfield.bottom();

    for _msg in reader.read() {
        // Find the closest bolt to the bottom boundary
        let min_distance = bolt_query
            .iter()
            .map(|position| position.0.y - bottom)
            .reduce(f32::min);

        let Some(min_distance) = min_distance else {
            continue;
        };

        if min_distance >= 0.0 && min_distance < config.nail_biter_pixels {
            // Always emit for juice/VFX feedback
            writer.write(HighlightTriggered {
                kind: HighlightKind::NailBiter,
            });

            // Record in stats — dedup by kind
            let already = stats
                .highlights
                .iter()
                .any(|h| h.kind == HighlightKind::NailBiter);
            if !already {
                stats.highlights.push(RunHighlight {
                    kind: HighlightKind::NailBiter,
                    node_index: run_state.node_index,
                    value: min_distance,
                    detail: None,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rantzsoft_spatial2d::components::{Position2D, Spatial2D};

    use super::*;
    use crate::{
        shared::GameDrawLayer,
        state::run::resources::{HighlightKind, RunHighlight},
    };

    #[derive(Resource)]
    struct TestMessages(Vec<NodeCleared>);

    fn enqueue_messages(msg_res: Res<TestMessages>, mut writer: MessageWriter<NodeCleared>) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

    #[derive(Resource, Default)]
    struct CapturedHighlightTriggered(Vec<HighlightTriggered>);

    fn collect_highlight_triggered(
        mut reader: MessageReader<HighlightTriggered>,
        mut captured: ResMut<CapturedHighlightTriggered>,
    ) {
        for msg in reader.read() {
            captured.0.push(msg.clone());
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<NodeCleared>()
            .add_message::<HighlightTriggered>()
            .init_resource::<RunStats>()
            .init_resource::<RunState>()
            .insert_resource(HighlightConfig::default())
            .insert_resource(PlayfieldConfig {
                height: 1080.0,
                ..Default::default()
            })
            .init_resource::<CapturedHighlightTriggered>()
            .add_systems(
                FixedUpdate,
                (
                    enqueue_messages,
                    detect_nail_biter,
                    collect_highlight_triggered,
                )
                    .chain(),
            );
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // --- Behavior 15: NailBiter detected when bolt near bottom at node clear ---

    #[test]
    fn nail_biter_detected_when_bolt_within_threshold() {
        let mut app = test_app();
        // PlayfieldConfig height=1080 → bottom = -540.0
        // Bolt at y=-515.0, distance = -515.0 - (-540.0) = 25.0
        // Default nail_biter_pixels = 30.0, 25.0 < 30.0 → detected
        app.world_mut().spawn((
            Bolt,
            Position2D(Vec2::new(50.0, -515.0)),
            Spatial2D,
            GameDrawLayer::Bolt,
        ));
        app.insert_resource(TestMessages(vec![NodeCleared]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let nail_biter = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::NailBiter);
        assert!(
            nail_biter.is_some(),
            "should detect NailBiter when bolt at y=-515.0, bottom=-540.0, distance=25.0 < threshold=30.0"
        );

        let captured = app.world().resource::<CapturedHighlightTriggered>();
        let msg = captured
            .0
            .iter()
            .find(|h| h.kind == HighlightKind::NailBiter);
        assert!(
            msg.is_some(),
            "should emit HighlightTriggered with NailBiter kind"
        );
    }

    // --- Behavior 16: NailBiter NOT detected when bolt far from bottom ---

    #[test]
    fn nail_biter_not_detected_when_bolt_far_from_bottom() {
        let mut app = test_app();
        // Bolt at y=-400.0, distance = -400.0 - (-540.0) = 140.0
        // Default nail_biter_pixels = 30.0, 140.0 > 30.0 → NOT detected
        app.world_mut().spawn((
            Bolt,
            Position2D(Vec2::new(50.0, -400.0)),
            Spatial2D,
            GameDrawLayer::Bolt,
        ));
        app.insert_resource(TestMessages(vec![NodeCleared]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let nail_biter = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::NailBiter);
        assert!(
            nail_biter.is_none(),
            "should NOT detect NailBiter when bolt at y=-400.0, distance=140.0 > threshold=30.0"
        );
    }

    // --- Behavior 17: Multiple bolts — any one within threshold triggers detection ---

    #[test]
    fn nail_biter_detected_when_any_bolt_within_threshold() {
        let mut app = test_app();
        // bolt_a far from bottom, bolt_b near bottom
        app.world_mut().spawn((
            Bolt,
            Position2D(Vec2::new(50.0, -200.0)),
            Spatial2D,
            GameDrawLayer::Bolt,
        ));
        app.world_mut().spawn((
            Bolt,
            Position2D(Vec2::new(50.0, -525.0)),
            Spatial2D,
            GameDrawLayer::Bolt,
        ));
        app.insert_resource(TestMessages(vec![NodeCleared]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let nail_biter = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::NailBiter);
        assert!(
            nail_biter.is_some(),
            "should detect NailBiter when at least one bolt (y=-525.0) is within threshold"
        );
    }

    // --- Behavior 18: Excludes BoltServing ---

    #[test]
    fn nail_biter_excludes_bolt_serving() {
        let mut app = test_app();
        // Bolt with BoltServing at y=-535.0 should be excluded from the query
        app.world_mut().spawn((
            Bolt,
            BoltServing,
            Position2D(Vec2::new(50.0, -535.0)),
            Spatial2D,
            GameDrawLayer::Bolt,
        ));
        app.insert_resource(TestMessages(vec![NodeCleared]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let nail_biter = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::NailBiter);
        assert!(
            nail_biter.is_none(),
            "should NOT detect NailBiter for bolts with BoltServing component"
        );
    }

    // --- Behavior 19: No bolts in world — no detection, no panic ---

    #[test]
    fn no_bolts_in_world_no_detection_no_panic() {
        let mut app = test_app();
        // No bolt entities spawned
        app.insert_resource(TestMessages(vec![NodeCleared]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        assert!(
            stats.highlights.is_empty(),
            "should not detect NailBiter and should not panic when no bolts exist"
        );
    }

    // --- Behavior 20: Dedup — once per run in RunStats ---

    #[test]
    fn dedup_nail_biter_only_once_in_run_stats() {
        let mut app = test_app();
        // Pre-fill with existing NailBiter
        app.world_mut()
            .resource_mut::<RunStats>()
            .highlights
            .push(RunHighlight {
                kind: HighlightKind::NailBiter,
                node_index: 0,
                value: 20.0,
                detail: None,
            });

        app.world_mut().spawn((
            Bolt,
            Position2D(Vec2::new(50.0, -525.0)),
            Spatial2D,
            GameDrawLayer::Bolt,
        ));
        app.insert_resource(TestMessages(vec![NodeCleared]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let nail_biter_count = stats
            .highlights
            .iter()
            .filter(|h| h.kind == HighlightKind::NailBiter)
            .count();
        assert_eq!(
            nail_biter_count, 1,
            "should NOT add a second NailBiter highlight (still 1 from pre-fill)"
        );

        // HighlightTriggered should still be emitted
        let captured = app.world().resource::<CapturedHighlightTriggered>();
        let msg = captured
            .0
            .iter()
            .find(|h| h.kind == HighlightKind::NailBiter);
        assert!(
            msg.is_some(),
            "should still emit HighlightTriggered even when not adding to highlights"
        );
    }
}
