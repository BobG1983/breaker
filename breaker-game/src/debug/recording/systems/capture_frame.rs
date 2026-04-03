//! System that captures input actions each fixed-update frame.

use bevy::prelude::*;

use crate::{
    debug::recording::resources::{
        RecordedFrame, RecordingBuffer, RecordingConfig, RecordingFrame,
    },
    input::resources::InputActions,
    state::run::node::ActiveNodeLayout,
};

/// Captures the current frame's [`InputActions`] into the [`RecordingBuffer`].
///
/// Skips frames with no actions. If `RecordingConfig::level_filter` is set,
/// also skips frames where the active layout name does not match.
pub(crate) fn capture_frame(
    config: Res<RecordingConfig>,
    actions: Res<InputActions>,
    layout: Option<Res<ActiveNodeLayout>>,
    mut buffer: ResMut<RecordingBuffer>,
    mut frame: ResMut<RecordingFrame>,
) {
    frame.0 += 1;

    if !config.enabled {
        return;
    }

    // Level filter: skip if layout name doesn't match
    if let Some(filter) = &config.level_filter {
        let layout_name = layout.as_ref().map_or("", |l| l.0.name.as_str());
        if layout_name != filter {
            return;
        }
    }

    if actions.0.is_empty() {
        return;
    }

    buffer.0.push(RecordedFrame {
        frame: frame.0,
        actions: actions.0.clone(),
    });
}

#[cfg(test)]
mod tests {
    use bevy::{app::App, ecs::schedule::ScheduleLabel};

    use super::*;
    use crate::{
        input::resources::GameAction,
        state::run::node::{ActiveNodeLayout, NodeLayout, definition::NodePool},
    };

    #[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
    struct TestSchedule;

    fn test_app_with_config(enabled: bool, filter: Option<&str>) -> App {
        let mut app = App::new();
        app.insert_resource(RecordingConfig {
            enabled,
            level_filter: filter.map(str::to_owned),
        })
        .insert_resource(InputActions::default())
        .init_resource::<RecordingBuffer>()
        .init_resource::<RecordingFrame>()
        .add_systems(TestSchedule, capture_frame);
        app
    }

    fn run_with_actions(app: &mut App, actions: Vec<GameAction>) {
        app.world_mut().resource_mut::<InputActions>().0 = actions;
        app.world_mut().run_schedule(TestSchedule);
    }

    #[test]
    fn capture_frame_records_non_empty_actions_when_enabled() {
        let mut app = test_app_with_config(true, None);
        run_with_actions(&mut app, vec![GameAction::MoveLeft, GameAction::Bump]);

        let buffer = app.world().resource::<RecordingBuffer>();
        assert_eq!(buffer.0.len(), 1);
        assert_eq!(
            buffer.0[0].actions,
            vec![GameAction::MoveLeft, GameAction::Bump]
        );
    }

    #[test]
    fn capture_frame_skips_empty_action_frames() {
        let mut app = test_app_with_config(true, None);
        run_with_actions(&mut app, vec![]);

        let buffer = app.world().resource::<RecordingBuffer>();
        assert!(buffer.0.is_empty());
    }

    #[test]
    fn capture_frame_does_nothing_when_disabled() {
        let mut app = test_app_with_config(false, None);
        run_with_actions(&mut app, vec![GameAction::Bump]);

        let buffer = app.world().resource::<RecordingBuffer>();
        assert!(buffer.0.is_empty());
    }

    #[test]
    fn capture_frame_skips_when_level_filter_does_not_match() {
        let mut app = test_app_with_config(true, Some("corridor"));
        // Insert wrong layout
        app.world_mut()
            .insert_resource(ActiveNodeLayout(NodeLayout {
                name: "open".to_owned(),
                timer_secs: 60.0,
                cols: 10,
                rows: 5,
                grid_top_offset: 0.0,
                grid: vec![],
                pool: NodePool::default(),
                entity_scale: 1.0,
            }));
        run_with_actions(&mut app, vec![GameAction::MoveRight]);

        let buffer = app.world().resource::<RecordingBuffer>();
        assert!(buffer.0.is_empty());
    }

    #[test]
    fn capture_frame_records_when_level_filter_matches() {
        let mut app = test_app_with_config(true, Some("corridor"));
        app.world_mut()
            .insert_resource(ActiveNodeLayout(NodeLayout {
                name: "corridor".to_owned(),
                timer_secs: 60.0,
                cols: 10,
                rows: 5,
                grid_top_offset: 0.0,
                grid: vec![],
                pool: NodePool::default(),
                entity_scale: 1.0,
            }));
        run_with_actions(&mut app, vec![GameAction::DashLeft]);

        let buffer = app.world().resource::<RecordingBuffer>();
        assert_eq!(buffer.0.len(), 1);
    }

    #[test]
    fn capture_frame_increments_frame_counter_regardless_of_filter() {
        let mut app = test_app_with_config(false, None);
        run_with_actions(&mut app, vec![]);
        run_with_actions(&mut app, vec![]);
        run_with_actions(&mut app, vec![]);

        let frame = app.world().resource::<RecordingFrame>();
        assert_eq!(frame.0, 3);
    }

    #[test]
    fn recorded_frame_index_reflects_frame_counter() {
        let mut app = test_app_with_config(true, None);
        run_with_actions(&mut app, vec![]); // frame 1 — empty, skipped
        run_with_actions(&mut app, vec![]); // frame 2 — empty, skipped
        run_with_actions(&mut app, vec![GameAction::Bump]); // frame 3 — recorded

        let buffer = app.world().resource::<RecordingBuffer>();
        assert_eq!(buffer.0.len(), 1);
        assert_eq!(buffer.0[0].frame, 3);
    }
}
