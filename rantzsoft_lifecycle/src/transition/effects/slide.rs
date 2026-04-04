//! Slide transition effects — `SlideLeft` and `SlideRight` (`OneShotTransition`).
//!
//! Slide effects animate the camera position rather than spawning an overlay
//! entity.

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
// Effect structs
// ---------------------------------------------------------------------------

/// Slide content to the left.
pub struct SlideLeft {
    /// Duration in seconds.
    pub duration: f32,
}

impl Default for SlideLeft {
    fn default() -> Self {
        Self { duration: 0.3 }
    }
}

impl Transition for SlideLeft {
    fn insert_starting(&self, world: &mut World) {
        world.insert_resource(StartingTransition::<Self>::new());
        world.insert_resource(SlideLeftConfig {
            duration: self.duration,
        });
    }
}
impl OneShotTransition for SlideLeft {}

/// Slide content to the right.
pub struct SlideRight {
    /// Duration in seconds.
    pub duration: f32,
}

impl Default for SlideRight {
    fn default() -> Self {
        Self { duration: 0.3 }
    }
}

impl Transition for SlideRight {
    fn insert_starting(&self, world: &mut World) {
        world.insert_resource(StartingTransition::<Self>::new());
        world.insert_resource(SlideRightConfig {
            duration: self.duration,
        });
    }
}
impl OneShotTransition for SlideRight {}

// ---------------------------------------------------------------------------
// Config resources
// ---------------------------------------------------------------------------

/// Configuration resource inserted by `SlideLeft::insert_starting`.
#[derive(Resource)]
pub struct SlideLeftConfig {
    /// Duration in seconds.
    pub duration: f32,
}

/// Configuration resource inserted by `SlideRight::insert_starting`.
#[derive(Resource)]
pub struct SlideRightConfig {
    /// Duration in seconds.
    pub duration: f32,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Start system for `SlideLeft` — records camera position and sends
/// `TransitionReady`.
pub(crate) fn slide_left_start(
    config: Res<SlideLeftConfig>,
    screen: Res<ScreenSize>,
    cameras: Query<&Transform, With<Camera2d>>,
    mut writer: MessageWriter<TransitionReady>,
    mut commands: Commands,
) {
    let camera_pos = cameras
        .iter()
        .next()
        .map_or(Vec2::ZERO, |t| Vec2::new(t.translation.x, t.translation.y));

    commands.insert_resource(SlideStartEnd {
        start: camera_pos,
        target: Vec2::new(camera_pos.x - screen.0.x, camera_pos.y),
    });
    commands.insert_resource(TransitionProgress {
        elapsed: 0.0,
        duration: config.duration,
        completed: false,
    });
    commands.remove_resource::<SlideLeftConfig>();
    writer.write(TransitionReady);
}

/// Run system for `SlideLeft` — lerps camera toward target.
pub(crate) fn slide_left_run(
    mut cameras: Query<&mut Transform, With<Camera2d>>,
    slide: Res<SlideStartEnd>,
    mut progress: ResMut<TransitionProgress>,
    mut writer: MessageWriter<TransitionRunComplete>,
) {
    if progress.completed {
        return;
    }

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

/// End system for `SlideLeft` — removes resources and sends `TransitionOver`.
pub(crate) fn slide_left_end(mut writer: MessageWriter<TransitionOver>, mut commands: Commands) {
    commands.remove_resource::<SlideStartEnd>();
    commands.remove_resource::<TransitionProgress>();
    writer.write(TransitionOver);
}

/// Start system for `SlideRight` — records camera position and sends
/// `TransitionReady`.
pub(crate) fn slide_right_start(
    config: Res<SlideRightConfig>,
    screen: Res<ScreenSize>,
    cameras: Query<&Transform, With<Camera2d>>,
    mut writer: MessageWriter<TransitionReady>,
    mut commands: Commands,
) {
    let camera_pos = cameras
        .iter()
        .next()
        .map_or(Vec2::ZERO, |t| Vec2::new(t.translation.x, t.translation.y));

    commands.insert_resource(SlideStartEnd {
        start: camera_pos,
        target: Vec2::new(camera_pos.x + screen.0.x, camera_pos.y),
    });
    commands.insert_resource(TransitionProgress {
        elapsed: 0.0,
        duration: config.duration,
        completed: false,
    });
    commands.remove_resource::<SlideRightConfig>();
    writer.write(TransitionReady);
}

/// Run system for `SlideRight` — lerps camera toward target.
pub(crate) fn slide_right_run(
    mut cameras: Query<&mut Transform, With<Camera2d>>,
    slide: Res<SlideStartEnd>,
    mut progress: ResMut<TransitionProgress>,
    mut writer: MessageWriter<TransitionRunComplete>,
) {
    if progress.completed {
        return;
    }

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

/// End system for `SlideRight` — removes resources and sends `TransitionOver`.
pub(crate) fn slide_right_end(mut writer: MessageWriter<TransitionOver>, mut commands: Commands) {
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
    // Section 3: SlideLeft
    // =======================================================================

    // --- Behavior 13: SlideLeft implements Transition and OneShotTransition ---

    #[test]
    fn slide_left_satisfies_transition_and_oneshot_transition() {
        let _effect: Box<dyn OneShotTransition> = Box::new(SlideLeft { duration: 0.4 });
    }

    // --- Behavior 14: SlideLeft start records camera position ---

    #[test]
    fn slide_left_start_inserts_slide_start_end_resource() {
        let mut app = effect_test_app();
        app.insert_resource(ScreenSize(Vec2::new(1280.0, 720.0)));
        app.insert_resource(SlideLeftConfig { duration: 0.4 });
        app.insert_resource(StartingTransition::<SlideLeft>::new());
        app.world_mut().spawn(Camera2d);
        app.add_systems(Update, slide_left_start);
        app.update();

        assert!(
            app.world().contains_resource::<SlideStartEnd>(),
            "SlideStartEnd should be inserted"
        );
        let sse = app.world().resource::<SlideStartEnd>();
        assert!(sse.start.x.abs() < f32::EPSILON, "start x should be 0.0");
        assert!(sse.start.y.abs() < f32::EPSILON, "start y should be 0.0");
        assert!(
            (sse.target.x - (-1280.0)).abs() < f32::EPSILON,
            "target x should be -1280.0 (negative screen width)"
        );
        assert!(sse.target.y.abs() < f32::EPSILON, "target y should be 0.0");
    }

    #[test]
    fn slide_left_start_sends_transition_ready() {
        let mut app = effect_test_app();
        app.insert_resource(ScreenSize(Vec2::new(1280.0, 720.0)));
        app.insert_resource(SlideLeftConfig { duration: 0.4 });
        app.insert_resource(StartingTransition::<SlideLeft>::new());
        app.world_mut().spawn(Camera2d);
        app.add_systems(Update, slide_left_start);
        app.update();

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionReady>>();
        assert_eq!(msgs.iter_current_update_messages().count(), 1);
    }

    #[test]
    fn slide_left_start_inserts_transition_progress() {
        let mut app = effect_test_app();
        app.insert_resource(ScreenSize(Vec2::new(1280.0, 720.0)));
        app.insert_resource(SlideLeftConfig { duration: 0.4 });
        app.insert_resource(StartingTransition::<SlideLeft>::new());
        app.world_mut().spawn(Camera2d);
        app.add_systems(Update, slide_left_start);
        app.update();

        assert!(app.world().contains_resource::<TransitionProgress>());
        let progress = app.world().resource::<TransitionProgress>();
        assert!(progress.elapsed.abs() < f32::EPSILON);
        assert!((progress.duration - 0.4).abs() < f32::EPSILON);
        assert!(!progress.completed);
    }

    #[test]
    fn slide_left_start_with_offset_camera_targets_relative_position() {
        let mut app = effect_test_app();
        app.insert_resource(ScreenSize(Vec2::new(1280.0, 720.0)));
        app.insert_resource(SlideLeftConfig { duration: 0.4 });
        app.insert_resource(StartingTransition::<SlideLeft>::new());
        app.world_mut()
            .spawn((Camera2d, Transform::from_xyz(100.0, 50.0, 0.0)));
        app.add_systems(Update, slide_left_start);
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

    // --- Behavior 15: SlideLeft run lerps camera toward target ---

    #[test]
    fn slide_left_run_lerps_camera_position() {
        let mut app = effect_test_app();
        app.insert_resource(RunningTransition::<SlideLeft>::new());
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
        app.add_systems(Update, slide_left_run);
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
    fn slide_left_run_does_not_send_complete_when_in_progress() {
        let mut app = effect_test_app();
        app.insert_resource(RunningTransition::<SlideLeft>::new());
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
        app.add_systems(Update, slide_left_run);
        app.update();

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
        assert_eq!(msgs.iter_current_update_messages().count(), 0);
    }

    // --- Behavior 16: SlideLeft run sends complete when done ---

    #[test]
    fn slide_left_run_sends_complete_at_full_progress() {
        let mut app = effect_test_app();
        app.insert_resource(RunningTransition::<SlideLeft>::new());
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
        app.add_systems(Update, slide_left_run);
        app.update();

        let cameras: Vec<&Transform> = app
            .world_mut()
            .query_filtered::<&Transform, With<Camera2d>>()
            .iter(app.world())
            .collect();
        assert!(
            (cameras[0].translation.x - (-1000.0)).abs() < 1.0,
            "camera should be at target x"
        );

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
        assert_eq!(msgs.iter_current_update_messages().count(), 1);
    }

    #[test]
    fn slide_left_run_clamps_to_target_on_overshoot() {
        let mut app = effect_test_app();
        app.insert_resource(RunningTransition::<SlideLeft>::new());
        app.insert_resource(SlideStartEnd {
            start: Vec2::new(0.0, 0.0),
            target: Vec2::new(-500.0, 0.0),
        });
        app.insert_resource(TransitionProgress {
            elapsed: 0.6,
            duration: 0.5,
            completed: false,
        });
        app.world_mut()
            .spawn((Camera2d, Transform::from_xyz(0.0, 0.0, 0.0)));
        app.add_systems(Update, slide_left_run);
        app.update();

        let cameras: Vec<&Transform> = app
            .world_mut()
            .query_filtered::<&Transform, With<Camera2d>>()
            .iter(app.world())
            .collect();
        assert!(
            (cameras[0].translation.x - (-500.0)).abs() < 1.0,
            "camera should be clamped to target, not past it"
        );
    }

    #[test]
    fn slide_left_run_sets_completed_flag() {
        let mut app = effect_test_app();
        app.insert_resource(RunningTransition::<SlideLeft>::new());
        app.insert_resource(SlideStartEnd {
            start: Vec2::new(0.0, 0.0),
            target: Vec2::new(-1000.0, 0.0),
        });
        app.insert_resource(TransitionProgress {
            elapsed: 0.5,
            duration: 0.5,
            completed: false,
        });
        app.world_mut()
            .spawn((Camera2d, Transform::from_xyz(0.0, 0.0, 0.0)));
        app.add_systems(Update, slide_left_run);
        app.update();

        let progress = app.world().resource::<TransitionProgress>();
        assert!(progress.completed);
    }

    #[test]
    fn slide_left_run_does_not_double_send_when_already_completed() {
        let mut app = effect_test_app();
        app.insert_resource(RunningTransition::<SlideLeft>::new());
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
        app.add_systems(Update, slide_left_run);
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

    // --- Behavior 17: SlideLeft end removes resources and sends TransitionOver ---

    #[test]
    fn slide_left_end_sends_transition_over() {
        let mut app = effect_test_app();
        app.insert_resource(EndingTransition::<SlideLeft>::new());
        app.insert_resource(SlideStartEnd {
            start: Vec2::ZERO,
            target: Vec2::new(-1000.0, 0.0),
        });
        app.insert_resource(TransitionProgress {
            elapsed: 0.5,
            duration: 0.5,
            completed: true,
        });
        app.add_systems(Update, slide_left_end);
        app.update();

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionOver>>();
        assert_eq!(msgs.iter_current_update_messages().count(), 1);
    }

    #[test]
    fn slide_left_end_removes_slide_start_end_and_progress() {
        let mut app = effect_test_app();
        app.insert_resource(EndingTransition::<SlideLeft>::new());
        app.insert_resource(SlideStartEnd {
            start: Vec2::ZERO,
            target: Vec2::new(-1000.0, 0.0),
        });
        app.insert_resource(TransitionProgress {
            elapsed: 0.5,
            duration: 0.5,
            completed: true,
        });
        app.add_systems(Update, slide_left_end);
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

    // --- Behavior 18: SlideLeft default configuration ---

    #[test]
    fn slide_left_default_duration_is_0_3() {
        let effect = SlideLeft::default();
        assert!((effect.duration - 0.3).abs() < f32::EPSILON);
    }

    // =======================================================================
    // Section 4: SlideRight
    // =======================================================================

    // --- Behavior 19: SlideRight implements Transition and OneShotTransition ---

    #[test]
    fn slide_right_satisfies_transition_and_oneshot_transition() {
        let _effect: Box<dyn OneShotTransition> = Box::new(SlideRight { duration: 0.4 });
    }

    // --- Behavior 20: SlideRight start records camera position targeting rightward ---

    #[test]
    fn slide_right_start_inserts_slide_start_end_targeting_rightward() {
        let mut app = effect_test_app();
        app.insert_resource(ScreenSize(Vec2::new(1280.0, 720.0)));
        app.insert_resource(SlideRightConfig { duration: 0.4 });
        app.insert_resource(StartingTransition::<SlideRight>::new());
        app.world_mut().spawn(Camera2d);
        app.add_systems(Update, slide_right_start);
        app.update();

        assert!(app.world().contains_resource::<SlideStartEnd>());
        let sse = app.world().resource::<SlideStartEnd>();
        assert!(sse.start.x.abs() < f32::EPSILON);
        assert!(
            (sse.target.x - 1280.0).abs() < f32::EPSILON,
            "target x should be +1280.0 (positive screen width)"
        );
    }

    #[test]
    fn slide_right_start_with_offset_camera() {
        let mut app = effect_test_app();
        app.insert_resource(ScreenSize(Vec2::new(1280.0, 720.0)));
        app.insert_resource(SlideRightConfig { duration: 0.4 });
        app.insert_resource(StartingTransition::<SlideRight>::new());
        app.world_mut()
            .spawn((Camera2d, Transform::from_xyz(-500.0, 100.0, 0.0)));
        app.add_systems(Update, slide_right_start);
        app.update();

        let sse = app.world().resource::<SlideStartEnd>();
        assert!(
            (sse.target.x - (-500.0 + 1280.0)).abs() < f32::EPSILON,
            "target x should be camera x + screen width = 780.0"
        );
        assert!(
            (sse.target.y - 100.0).abs() < f32::EPSILON,
            "target y should match camera y"
        );
    }

    #[test]
    fn slide_right_start_sends_transition_ready_and_progress() {
        let mut app = effect_test_app();
        app.insert_resource(ScreenSize(Vec2::new(1280.0, 720.0)));
        app.insert_resource(SlideRightConfig { duration: 0.4 });
        app.insert_resource(StartingTransition::<SlideRight>::new());
        app.world_mut().spawn(Camera2d);
        app.add_systems(Update, slide_right_start);
        app.update();

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionReady>>();
        assert_eq!(msgs.iter_current_update_messages().count(), 1);
        assert!(app.world().contains_resource::<TransitionProgress>());
        let progress = app.world().resource::<TransitionProgress>();
        assert!((progress.duration - 0.4).abs() < f32::EPSILON);
    }

    // --- Behavior 21: SlideRight run lerps camera rightward ---

    #[test]
    fn slide_right_run_lerps_camera_rightward() {
        let mut app = effect_test_app();
        app.insert_resource(RunningTransition::<SlideRight>::new());
        app.insert_resource(SlideStartEnd {
            start: Vec2::new(0.0, 0.0),
            target: Vec2::new(1000.0, 0.0),
        });
        app.insert_resource(TransitionProgress {
            elapsed: 0.5,
            duration: 1.0,
            completed: false,
        });
        app.world_mut()
            .spawn((Camera2d, Transform::from_xyz(0.0, 0.0, 0.0)));
        app.add_systems(Update, slide_right_run);
        app.update();

        let cameras: Vec<&Transform> = app
            .world_mut()
            .query_filtered::<&Transform, With<Camera2d>>()
            .iter(app.world())
            .collect();
        assert!(
            (cameras[0].translation.x - 500.0).abs() < 1.0,
            "camera x should be ~500.0 at 50% progress, got {}",
            cameras[0].translation.x
        );
    }

    #[test]
    fn slide_right_run_sends_complete_at_full_progress() {
        let mut app = effect_test_app();
        app.insert_resource(RunningTransition::<SlideRight>::new());
        app.insert_resource(SlideStartEnd {
            start: Vec2::new(0.0, 0.0),
            target: Vec2::new(1000.0, 0.0),
        });
        app.insert_resource(TransitionProgress {
            elapsed: 1.0,
            duration: 1.0,
            completed: false,
        });
        app.world_mut()
            .spawn((Camera2d, Transform::from_xyz(0.0, 0.0, 0.0)));
        app.add_systems(Update, slide_right_run);
        app.update();

        let cameras: Vec<&Transform> = app
            .world_mut()
            .query_filtered::<&Transform, With<Camera2d>>()
            .iter(app.world())
            .collect();
        assert!(
            (cameras[0].translation.x - 1000.0).abs() < 1.0,
            "camera should be at target"
        );

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
        assert_eq!(msgs.iter_current_update_messages().count(), 1);
    }

    #[test]
    fn slide_right_run_does_not_double_send_when_already_completed() {
        let mut app = effect_test_app();
        app.insert_resource(RunningTransition::<SlideRight>::new());
        app.insert_resource(SlideStartEnd {
            start: Vec2::new(0.0, 0.0),
            target: Vec2::new(1000.0, 0.0),
        });
        app.insert_resource(TransitionProgress {
            elapsed: 1.0,
            duration: 1.0,
            completed: true,
        });
        app.world_mut()
            .spawn((Camera2d, Transform::from_xyz(1000.0, 0.0, 0.0)));
        app.add_systems(Update, slide_right_run);
        app.update();

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionRunComplete>>();
        assert_eq!(msgs.iter_current_update_messages().count(), 0);
    }

    // --- Behavior 22: SlideRight end ---

    #[test]
    fn slide_right_end_sends_transition_over_and_removes_resources() {
        let mut app = effect_test_app();
        app.insert_resource(EndingTransition::<SlideRight>::new());
        app.insert_resource(SlideStartEnd {
            start: Vec2::ZERO,
            target: Vec2::new(1000.0, 0.0),
        });
        app.insert_resource(TransitionProgress {
            elapsed: 0.5,
            duration: 0.5,
            completed: true,
        });
        app.add_systems(Update, slide_right_end);
        app.update();

        let msgs = app
            .world()
            .resource::<bevy::ecs::message::Messages<TransitionOver>>();
        assert_eq!(msgs.iter_current_update_messages().count(), 1);
        assert!(!app.world().contains_resource::<SlideStartEnd>());
        assert!(!app.world().contains_resource::<TransitionProgress>());
    }

    // --- Behavior 23: SlideRight default ---

    #[test]
    fn slide_right_default_duration_is_0_3() {
        let effect = SlideRight::default();
        assert!((effect.duration - 0.3).abs() < f32::EPSILON);
    }

    // =======================================================================
    // Section 13: insert_starting overrides (behaviors 59-60)
    // =======================================================================

    #[test]
    fn slide_left_insert_starting_inserts_marker_and_config() {
        let mut world = World::new();
        let effect = SlideLeft { duration: 0.4 };
        effect.insert_starting(&mut world);

        assert!(world.contains_resource::<StartingTransition<SlideLeft>>());
        assert!(
            world.contains_resource::<SlideLeftConfig>(),
            "SlideLeftConfig should be inserted by insert_starting"
        );
    }

    #[test]
    fn slide_right_insert_starting_inserts_marker_and_config() {
        let mut world = World::new();
        let effect = SlideRight { duration: 0.4 };
        effect.insert_starting(&mut world);

        assert!(world.contains_resource::<StartingTransition<SlideRight>>());
        assert!(
            world.contains_resource::<SlideRightConfig>(),
            "SlideRightConfig should be inserted by insert_starting"
        );
    }
}
