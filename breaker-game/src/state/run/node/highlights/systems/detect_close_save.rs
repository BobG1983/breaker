//! System to detect `CloseSave` highlights when a bolt is saved near the bottom boundary.

use bevy::prelude::*;

use crate::{
    prelude::*,
    state::run::{definition::HighlightConfig, messages::HighlightTriggered, resources::*},
};

/// Reads [`BumpPerformed`] messages and detects `CloseSave` highlights
/// when the bumped bolt is within `close_save_pixels` of the bottom boundary.
///
/// Records the highlight in [`RunStats`] and emits [`HighlightTriggered`].
pub(crate) fn detect_close_save(
    mut reader: MessageReader<BumpPerformed>,
    bolt_query: Query<&Position2D, (With<Bolt>, Without<BoltServing>)>,
    playfield: Res<PlayfieldConfig>,
    config: Res<HighlightConfig>,
    mut stats: ResMut<RunStats>,
    run_state: Res<NodeOutcome>,
    mut writer: MessageWriter<HighlightTriggered>,
) {
    let bottom = playfield.bottom();

    for msg in reader.read() {
        let Some(bolt_entity) = msg.bolt else {
            continue;
        };
        let Ok(position) = bolt_query.get(bolt_entity) else {
            continue;
        };

        let bolt_y = position.0.y;
        let distance = bolt_y - bottom;

        if distance >= 0.0 && distance < config.close_save_pixels {
            // Always emit for juice/VFX feedback
            writer.write(HighlightTriggered {
                kind: HighlightKind::CloseSave,
            });

            // Record in stats — selection happens at run-end
            stats.highlights.push(RunHighlight {
                kind: HighlightKind::CloseSave,
                node_index: run_state.node_index,
                value: distance,
                detail: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use rantzsoft_spatial2d::components::{Position2D, Spatial2D};

    use super::*;
    use crate::{
        breaker::messages::BumpGrade,
        shared::GameDrawLayer,
        state::run::resources::{HighlightKind, RunHighlight},
    };

    #[derive(Resource)]
    struct TestMessages(Vec<BumpPerformed>);

    fn enqueue_messages(msg_res: Res<TestMessages>, mut writer: MessageWriter<BumpPerformed>) {
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
            .add_message::<BumpPerformed>()
            .add_message::<HighlightTriggered>()
            .init_resource::<RunStats>()
            .init_resource::<NodeOutcome>()
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
                    detect_close_save,
                    collect_highlight_triggered,
                )
                    .chain(),
            );
        app
    }

    use crate::shared::test_utils::tick;

    // --- Behavior 1: CloseSave detected when bolt is near bottom ---

    #[test]
    fn close_save_detected_when_bolt_within_threshold() {
        let mut app = test_app();
        // PlayfieldConfig height=1080 → bottom = -540.0
        // Bolt at y=-525.0, distance = -525.0 - (-540.0) = 15.0
        // Default close_save_pixels = 20.0, 15.0 < 20.0 → detected
        let bolt_entity = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(100.0, -525.0)),
                Spatial2D,
                GameDrawLayer::Bolt,
            ))
            .id();
        app.insert_resource(TestMessages(vec![BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt_entity),
            breaker: Entity::PLACEHOLDER,
        }]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let close_save = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::CloseSave);
        assert!(
            close_save.is_some(),
            "should detect CloseSave when bolt at y=-525.0, bottom=-540.0, distance=15.0 < threshold=20.0"
        );
        let highlight = close_save.unwrap();
        assert!(
            (highlight.value - 15.0).abs() < 1.0,
            "highlight value should be approximately 15.0 (distance from bottom), got {}",
            highlight.value
        );

        let captured = app.world().resource::<CapturedHighlightTriggered>();
        let msg = captured
            .0
            .iter()
            .find(|h| h.kind == HighlightKind::CloseSave);
        assert!(
            msg.is_some(),
            "should emit HighlightTriggered with CloseSave kind"
        );
    }

    // --- Behavior 2: CloseSave NOT detected when bolt is far from bottom ---

    #[test]
    fn close_save_not_detected_when_bolt_beyond_threshold() {
        let mut app = test_app();
        // Bolt at y=-510.0, distance = -510.0 - (-540.0) = 30.0
        // Default close_save_pixels = 20.0, 30.0 > 20.0 → NOT detected
        let bolt_entity = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(100.0, -510.0)),
                Spatial2D,
                GameDrawLayer::Bolt,
            ))
            .id();
        app.insert_resource(TestMessages(vec![BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt_entity),
            breaker: Entity::PLACEHOLDER,
        }]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let close_save = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::CloseSave);
        assert!(
            close_save.is_none(),
            "should NOT detect CloseSave when bolt at y=-510.0, distance=30.0 > threshold=20.0"
        );

        let captured = app.world().resource::<CapturedHighlightTriggered>();
        let msg = captured
            .0
            .iter()
            .find(|h| h.kind == HighlightKind::CloseSave);
        assert!(
            msg.is_none(),
            "should NOT emit HighlightTriggered when bolt is far from bottom"
        );
    }

    // --- Behavior 3: Skips when bolt entity not found ---

    #[test]
    fn skips_when_bolt_entity_not_found() {
        let mut app = test_app();
        // BumpPerformed with no bolt — no matching bolt entity exists
        app.insert_resource(TestMessages(vec![BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: None,
            breaker: Entity::PLACEHOLDER,
        }]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        assert!(
            stats.highlights.is_empty(),
            "should not panic and should produce no highlights when bolt entity is missing"
        );
    }

    // --- Behavior 4: Multiple CloseSave entries allowed ---

    #[test]
    fn multiple_close_save_entries_allowed_across_bumps() {
        let mut app = test_app();
        // Pre-fill highlights with an existing CloseSave from a previous bump
        app.world_mut()
            .resource_mut::<RunStats>()
            .highlights
            .push(RunHighlight {
                kind: HighlightKind::CloseSave,
                node_index: 0,
                value: 10.0,
                detail: None,
            });

        let bolt_entity = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(100.0, -525.0)),
                Spatial2D,
                GameDrawLayer::Bolt,
            ))
            .id();
        app.insert_resource(TestMessages(vec![BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt_entity),
            breaker: Entity::PLACEHOLDER,
        }]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let close_save_count = stats
            .highlights
            .iter()
            .filter(|h| h.kind == HighlightKind::CloseSave)
            .count();
        assert!(
            close_save_count >= 2,
            "should allow multiple CloseSave highlights (no dedup — selection happens at run-end). Got {close_save_count}"
        );

        // HighlightTriggered should still be emitted
        let captured = app.world().resource::<CapturedHighlightTriggered>();
        let msg = captured
            .0
            .iter()
            .find(|h| h.kind == HighlightKind::CloseSave);
        assert!(
            msg.is_some(),
            "should still emit HighlightTriggered for CloseSave"
        );
    }

    // --- Behavior 5: No cap during detection — stored beyond old cap ---

    #[test]
    fn stores_highlight_beyond_old_cap() {
        let mut app = test_app();
        // Pre-fill to old cap of 5 — system previously would not add more
        {
            let mut stats = app.world_mut().resource_mut::<RunStats>();
            for i in 0..5 {
                stats.highlights.push(RunHighlight {
                    kind: HighlightKind::MassDestruction,
                    node_index: i,
                    value: 10.0,
                    detail: None,
                });
            }
        }

        let bolt_entity = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(100.0, -525.0)),
                Spatial2D,
                GameDrawLayer::Bolt,
            ))
            .id();
        app.insert_resource(TestMessages(vec![BumpPerformed {
            grade: BumpGrade::Perfect,
            bolt: Some(bolt_entity),
            breaker: Entity::PLACEHOLDER,
        }]));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let close_save = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::CloseSave);
        assert!(
            close_save.is_some(),
            "CloseSave should be stored even when 5 highlights already exist — selection happens at run-end"
        );
        assert!(
            stats.highlights.len() > 5,
            "highlight count should grow beyond old cap of 5. Got {}",
            stats.highlights.len()
        );

        // HighlightTriggered should STILL be emitted
        let captured = app.world().resource::<CapturedHighlightTriggered>();
        let msg = captured
            .0
            .iter()
            .find(|h| h.kind == HighlightKind::CloseSave);
        assert!(
            msg.is_some(),
            "should emit HighlightTriggered for CloseSave"
        );
    }
}
