//! Slide transition effects.
//!
//! The primary API is [`Slide`] with [`SlideDirection`], a unified
//! `OneShotTransition` that slides the camera in any of four directions.

use bevy::prelude::*;

use super::shared::{ScreenSize, SlideStartEnd, TransitionProgress};
#[cfg(test)]
use crate::transition::resources::{EndingTransition, RunningTransition};
use crate::transition::{
    messages::{TransitionOver, TransitionReady, TransitionRunComplete},
    resources::StartingTransition,
    traits::{OneShotTransition, Transition},
};

// ---------------------------------------------------------------------------
// SlideDirection
// ---------------------------------------------------------------------------

/// Direction for slide transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SlideDirection {
    /// Slide content to the left (camera moves left, negative X).
    #[default]
    Left,
    /// Slide content to the right (camera moves right, positive X).
    Right,
    /// Slide content upward (camera moves up, positive Y).
    Up,
    /// Slide content downward (camera moves down, negative Y).
    Down,
}

// ---------------------------------------------------------------------------
// Slide (unified effect)
// ---------------------------------------------------------------------------

/// Slide content in the specified direction.
///
/// An `OneShotTransition` that animates the camera position by one screen
/// dimension in the given [`SlideDirection`].
pub struct Slide {
    /// Duration in seconds.
    pub duration: f32,
    /// Direction of the slide.
    pub direction: SlideDirection,
}

impl Default for Slide {
    fn default() -> Self {
        Self {
            duration: 0.3,
            direction: SlideDirection::Left,
        }
    }
}

impl Transition for Slide {
    fn insert_starting(&self, world: &mut World) {
        world.insert_resource(StartingTransition::<Self>::new());
        world.insert_resource(SlideConfig {
            duration: self.duration,
            direction: self.direction,
        });
    }
}
impl OneShotTransition for Slide {}

// ---------------------------------------------------------------------------
// SlideConfig (unified config)
// ---------------------------------------------------------------------------

/// Configuration resource inserted by [`Slide::insert_starting`].
#[derive(Resource)]
pub struct SlideConfig {
    /// Duration in seconds.
    pub duration: f32,
    /// Direction of the slide.
    pub direction: SlideDirection,
}

// ---------------------------------------------------------------------------
// Unified Slide systems
// ---------------------------------------------------------------------------

/// Start system for [`Slide`] -- records camera position and sends
/// `TransitionReady`.
pub(crate) fn slide_start(
    config: Res<SlideConfig>,
    screen: Res<ScreenSize>,
    cameras: Query<&Transform, With<Camera2d>>,
    mut writer: MessageWriter<TransitionReady>,
    mut commands: Commands,
) {
    let camera_pos = cameras
        .iter()
        .next()
        .map_or(Vec2::ZERO, |t| Vec2::new(t.translation.x, t.translation.y));

    let target = match config.direction {
        SlideDirection::Left => Vec2::new(camera_pos.x - screen.0.x, camera_pos.y),
        SlideDirection::Right => Vec2::new(camera_pos.x + screen.0.x, camera_pos.y),
        SlideDirection::Up => Vec2::new(camera_pos.x, camera_pos.y + screen.0.y),
        SlideDirection::Down => Vec2::new(camera_pos.x, camera_pos.y - screen.0.y),
    };

    commands.insert_resource(SlideStartEnd {
        start: camera_pos,
        target,
    });
    commands.insert_resource(TransitionProgress {
        elapsed: 0.0,
        duration: config.duration,
        completed: false,
    });
    commands.remove_resource::<SlideConfig>();
    writer.write(TransitionReady);
}

/// Run system for [`Slide`] -- lerps camera toward target.
pub(crate) fn slide_run(
    mut cameras: Query<&mut Transform, With<Camera2d>>,
    slide: Res<SlideStartEnd>,
    mut progress: ResMut<TransitionProgress>,
    time: Res<Time<Real>>,
    mut writer: MessageWriter<TransitionRunComplete>,
) {
    if progress.completed {
        return;
    }

    progress.elapsed += time.delta_secs();

    let t = if progress.duration > 0.0 {
        (progress.elapsed / progress.duration).clamp(0.0, 1.0)
    } else {
        1.0
    };

    let pos = slide.start.lerp(slide.target, t);
    for mut transform in &mut cameras {
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
    }

    if t >= 1.0 {
        progress.completed = true;
        writer.write(TransitionRunComplete);
    }
}

/// End system for [`Slide`] -- removes resources and sends `TransitionOver`.
pub(crate) fn slide_end(mut writer: MessageWriter<TransitionOver>, mut commands: Commands) {
    commands.remove_resource::<SlideStartEnd>();
    commands.remove_resource::<TransitionProgress>();
    writer.write(TransitionOver);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn effect_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<TransitionReady>();
        app.add_message::<TransitionRunComplete>();
        app.add_message::<TransitionOver>();
        app.insert_resource(ScreenSize::default());
        app
    }

    // =======================================================================
    // Section: Slide (unified)
    // =======================================================================

    // --- SlideDirection ---

    #[test]
    fn slide_direction_default_is_left() {
        assert_eq!(SlideDirection::default(), SlideDirection::Left);
    }

    #[test]
    fn slide_direction_has_all_four_variants() {
        let left = SlideDirection::Left;
        let _ = &left;
        let right = SlideDirection::Right;
        let _ = &right;
        let up = SlideDirection::Up;
        let _ = &up;
        let down = SlideDirection::Down;
        let _ = &down;
    }

    // --- Slide default ---

    #[test]
    fn slide_default_duration_is_0_3() {
        let effect = Slide::default();
        assert!((effect.duration - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn slide_default_direction_is_left() {
        let effect = Slide::default();
        assert_eq!(effect.direction, SlideDirection::Left);
    }

    // --- Slide implements Transition and OneShotTransition ---

    #[test]
    fn slide_satisfies_transition_and_oneshot_transition() {
        let _effect: Box<dyn OneShotTransition> = Box::new(Slide {
            duration: 0.4,
            direction: SlideDirection::Left,
        });
    }

    // --- Slide insert_starting ---

    #[test]
    fn slide_insert_starting_inserts_marker_and_config() {
        let mut world = World::new();
        let effect = Slide {
            duration: 0.4,
            direction: SlideDirection::Right,
        };
        effect.insert_starting(&mut world);

        assert!(world.contains_resource::<StartingTransition<Slide>>());
        assert!(
            world.contains_resource::<SlideConfig>(),
            "SlideConfig should be inserted by insert_starting"
        );
        let config = world.resource::<SlideConfig>();
        assert!((config.duration - 0.4).abs() < f32::EPSILON);
        assert_eq!(config.direction, SlideDirection::Right);
    }

    // --- Slide start (Left direction) ---

    #[test]
    fn slide_start_left_inserts_slide_start_end_resource() {
        let mut app = effect_test_app();
        app.insert_resource(ScreenSize(Vec2::new(1280.0, 720.0)));
        app.insert_resource(SlideConfig {
            duration: 0.4,
            direction: SlideDirection::Left,
        });
        app.insert_resource(StartingTransition::<Slide>::new());
        app.world_mut().spawn(Camera2d);
        app.add_systems(Update, slide_start);
        app.update();

        assert!(
            app.world().contains_resource::<SlideStartEnd>(),
            "SlideStartEnd should be inserted"
        );
        let sse = app.world().resource::<SlideStartEnd>();
        assert!(sse.start.x.abs() < f32::EPSILON, "start x should be 0.0");
        assert!(
            (sse.target.x - (-1280.0)).abs() < f32::EPSILON,
            "target x should be -1280.0 (negative screen width)"
        );
        assert!(sse.target.y.abs() < f32::EPSILON, "target y should be 0.0");
    }

    #[test]
    fn slide_start_left_sends_transition_ready() {
        let mut app = effect_test_app();
        app.insert_resource(ScreenSize(Vec2::new(1280.0, 720.0)));
        app.insert_resource(SlideConfig {
            duration: 0.4,
            direction: SlideDirection::Left,
        });
        app.insert_resource(StartingTransition::<Slide>::new());
        app.world_mut().spawn(Camera2d);
        app.add_systems(Update, slide_start);
        app.update();

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionReady>>();
        assert_eq!(msgs.iter_current_update_messages().count(), 1);
    }

    #[test]
    fn slide_start_left_inserts_transition_progress() {
        let mut app = effect_test_app();
        app.insert_resource(ScreenSize(Vec2::new(1280.0, 720.0)));
        app.insert_resource(SlideConfig {
            duration: 0.4,
            direction: SlideDirection::Left,
        });
        app.insert_resource(StartingTransition::<Slide>::new());
        app.world_mut().spawn(Camera2d);
        app.add_systems(Update, slide_start);
        app.update();

        assert!(app.world().contains_resource::<TransitionProgress>());
        let progress = app.world().resource::<TransitionProgress>();
        assert!(progress.elapsed.abs() < f32::EPSILON);
        assert!((progress.duration - 0.4).abs() < f32::EPSILON);
        assert!(!progress.completed);
    }

    // --- Slide start (Right direction) ---

    #[test]
    fn slide_start_right_targets_positive_x() {
        let mut app = effect_test_app();
        app.insert_resource(ScreenSize(Vec2::new(1280.0, 720.0)));
        app.insert_resource(SlideConfig {
            duration: 0.4,
            direction: SlideDirection::Right,
        });
        app.insert_resource(StartingTransition::<Slide>::new());
        app.world_mut().spawn(Camera2d);
        app.add_systems(Update, slide_start);
        app.update();

        let sse = app.world().resource::<SlideStartEnd>();
        assert!(
            (sse.target.x - 1280.0).abs() < f32::EPSILON,
            "target x should be +1280.0 (positive screen width)"
        );
        assert!(sse.target.y.abs() < f32::EPSILON, "target y should be 0.0");
    }

    // --- Slide start (Up direction) ---

    #[test]
    fn slide_start_up_targets_positive_y() {
        let mut app = effect_test_app();
        app.insert_resource(ScreenSize(Vec2::new(1280.0, 720.0)));
        app.insert_resource(SlideConfig {
            duration: 0.4,
            direction: SlideDirection::Up,
        });
        app.insert_resource(StartingTransition::<Slide>::new());
        app.world_mut().spawn(Camera2d);
        app.add_systems(Update, slide_start);
        app.update();

        let sse = app.world().resource::<SlideStartEnd>();
        assert!(sse.target.x.abs() < f32::EPSILON, "target x should be 0.0");
        assert!(
            (sse.target.y - 720.0).abs() < f32::EPSILON,
            "target y should be +720.0 (positive screen height)"
        );
    }

    // --- Slide start (Down direction) ---

    #[test]
    fn slide_start_down_targets_negative_y() {
        let mut app = effect_test_app();
        app.insert_resource(ScreenSize(Vec2::new(1280.0, 720.0)));
        app.insert_resource(SlideConfig {
            duration: 0.4,
            direction: SlideDirection::Down,
        });
        app.insert_resource(StartingTransition::<Slide>::new());
        app.world_mut().spawn(Camera2d);
        app.add_systems(Update, slide_start);
        app.update();

        let sse = app.world().resource::<SlideStartEnd>();
        assert!(sse.target.x.abs() < f32::EPSILON, "target x should be 0.0");
        assert!(
            (sse.target.y - (-720.0)).abs() < f32::EPSILON,
            "target y should be -720.0 (negative screen height)"
        );
    }

    // --- Slide start with offset camera ---

    #[test]
    fn slide_start_with_offset_camera_targets_relative_position() {
        let mut app = effect_test_app();
        app.insert_resource(ScreenSize(Vec2::new(1280.0, 720.0)));
        app.insert_resource(SlideConfig {
            duration: 0.4,
            direction: SlideDirection::Left,
        });
        app.insert_resource(StartingTransition::<Slide>::new());
        app.world_mut()
            .spawn((Camera2d, Transform::from_xyz(100.0, 50.0, 0.0)));
        app.add_systems(Update, slide_start);
        app.update();

        let sse = app.world().resource::<SlideStartEnd>();
        assert!(
            (sse.start.x - 100.0).abs() < f32::EPSILON,
            "start x should be camera's current x"
        );
        assert!(
            (sse.target.x - (100.0 - 1280.0)).abs() < f32::EPSILON,
            "target x should be camera x - screen width = -1180.0"
        );
        assert!(
            (sse.target.y - 50.0).abs() < f32::EPSILON,
            "target y should match camera y"
        );
    }

    // --- Slide run ---

    #[test]
    fn slide_run_lerps_camera_position() {
        let mut app = effect_test_app();
        app.insert_resource(RunningTransition::<Slide>::new());
        app.insert_resource(SlideStartEnd {
            start: Vec2::new(0.0, 0.0),
            target: Vec2::new(-1000.0, 0.0),
        });
        app.insert_resource(TransitionProgress {
            elapsed: 0.25,
            duration: 1.0,
            completed: false,
        });
        app.world_mut()
            .spawn((Camera2d, Transform::from_xyz(0.0, 0.0, 0.0)));
        app.add_systems(Update, slide_run);
        app.update();

        let cameras: Vec<&Transform> = app
            .world_mut()
            .query_filtered::<&Transform, With<Camera2d>>()
            .iter(app.world())
            .collect();
        assert!(
            (cameras[0].translation.x - (-250.0)).abs() < 1.0,
            "camera x should be ~-250.0 at 25% progress, got {}",
            cameras[0].translation.x
        );
    }

    #[test]
    fn slide_run_sends_complete_at_full_progress() {
        let mut app = effect_test_app();
        app.insert_resource(RunningTransition::<Slide>::new());
        app.insert_resource(SlideStartEnd {
            start: Vec2::new(0.0, 0.0),
            target: Vec2::new(-1000.0, 0.0),
        });
        app.insert_resource(TransitionProgress {
            elapsed: 1.0,
            duration: 1.0,
            completed: false,
        });
        app.world_mut()
            .spawn((Camera2d, Transform::from_xyz(0.0, 0.0, 0.0)));
        app.add_systems(Update, slide_run);
        app.update();

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
        assert_eq!(msgs.iter_current_update_messages().count(), 1);
    }

    #[test]
    fn slide_run_does_not_double_send_when_already_completed() {
        let mut app = effect_test_app();
        app.insert_resource(RunningTransition::<Slide>::new());
        app.insert_resource(SlideStartEnd {
            start: Vec2::new(0.0, 0.0),
            target: Vec2::new(-1000.0, 0.0),
        });
        app.insert_resource(TransitionProgress {
            elapsed: 1.0,
            duration: 1.0,
            completed: true,
        });
        app.world_mut()
            .spawn((Camera2d, Transform::from_xyz(-1000.0, 0.0, 0.0)));
        app.add_systems(Update, slide_run);
        app.update();

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
        assert_eq!(
            msgs.iter_current_update_messages().count(),
            0,
            "should not double-send when already completed"
        );
    }

    // --- Slide end ---

    #[test]
    fn slide_end_sends_transition_over() {
        let mut app = effect_test_app();
        app.insert_resource(EndingTransition::<Slide>::new());
        app.insert_resource(SlideStartEnd {
            start: Vec2::ZERO,
            target: Vec2::new(-1000.0, 0.0),
        });
        app.insert_resource(TransitionProgress {
            elapsed: 0.5,
            duration: 0.5,
            completed: true,
        });
        app.add_systems(Update, slide_end);
        app.update();

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionOver>>();
        assert_eq!(msgs.iter_current_update_messages().count(), 1);
    }

    #[test]
    fn slide_end_removes_slide_start_end_and_progress() {
        let mut app = effect_test_app();
        app.insert_resource(EndingTransition::<Slide>::new());
        app.insert_resource(SlideStartEnd {
            start: Vec2::ZERO,
            target: Vec2::new(-1000.0, 0.0),
        });
        app.insert_resource(TransitionProgress {
            elapsed: 0.5,
            duration: 0.5,
            completed: true,
        });
        app.add_systems(Update, slide_end);
        app.update();

        assert!(
            !app.world().contains_resource::<SlideStartEnd>(),
            "SlideStartEnd should be removed"
        );
        assert!(
            !app.world().contains_resource::<TransitionProgress>(),
            "TransitionProgress should be removed"
        );
    }
}
